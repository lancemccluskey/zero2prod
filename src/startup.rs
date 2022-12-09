use std::{net::TcpListener, sync::Arc};

use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use http::Request;
use hyper::{server::conn::AddrIncoming, Body};
use reqwest::Url;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::*,
};

pub struct AppState {
    pub pool: PgPool,
    pub email_client: EmailClient,
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
    run(listener, connection_pool, email_client)
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
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    // Initialize application state
    let app_state = Arc::new(AppState { pool, email_client });

    // Setup tracing for the application
    // Rust yells at me when I don't include the request in the `new_make_span` closure. No idea why
    let svc = ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
        |_request: &Request<Body>| {
            tracing::info_span!("request", request_id = Uuid::new_v4().to_string())
        },
    ));

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(svc)
        .with_state(app_state);

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    Ok(server)
}
