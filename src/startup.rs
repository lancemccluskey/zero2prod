use std::{net::TcpListener, ops::Deref, sync::Arc};

use axum::{
    extract::FromRef,
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use axum_extra::extract::cookie::Key;
use http::Request;
use hyper::{server::conn::AddrIncoming, Body};
use reqwest::Url;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::*,
};

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

#[derive(Clone)]
pub struct AppState(Arc<InnerState>);

impl Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct InnerState {
    pub pool: PgPool,
    pub email_client: EmailClient,
    pub application_base_url: String,
    pub key: Key,
}

// this impl tells `SignedCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

pub async fn build(
    configuration: Settings,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    // Setup db connection pool
    let connection_pool = get_connection_pool(&configuration.database);

    // Setup `EmailClient` for sending emails after subscribing
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        Url::parse(configuration.email_client.base_url.as_str())
            .expect("Unable to parse config base_url"),
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    // Get `TcpListener` for setting up as server
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind port");

    // Get the server
    run(
        listener,
        connection_pool,
        email_client,
        configuration.application.base_url,
        HmacSecret(configuration.application.hmac_secret),
    )
}

pub fn get_connection_pool(database_settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(database_settings.with_db())
}

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: HmacSecret,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    // Initialize application state
    let app_state = AppState(Arc::new(InnerState {
        pool,
        email_client,
        application_base_url: base_url,
        key: Key::from(hmac_secret.0.expose_secret().as_bytes()),
    }));

    // Setup tracing for the application
    // Rust yells at me when I don't include the request in the `new_make_span` closure. No idea why
    let svc = ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
        |_request: &Request<Body>| {
            tracing::info_span!("request", request_id = Uuid::new_v4().to_string())
        },
    ));

    // TODO: Need to test it out
    // * Issue was I was using Arc<AppState> in all my handlers before
    // * I should be okay since im still using Arc but its wrapped in a new type
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .route("/", get(home))
        .route("/login", get(login_form).post(login))
        .layer(svc)
        .with_state(app_state);

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    Ok(server)
}
