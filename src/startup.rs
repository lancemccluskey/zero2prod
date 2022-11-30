use std::{net::TcpListener, sync::Arc};

use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use http::Request;
use hyper::{server::conn::AddrIncoming, Body};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::routes::*;

pub fn run(
    listener: TcpListener,
    connection: PgPool,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let connection_state = Arc::new(connection);

    // TODO: Prob move this out at some point
    // Also, Rust yells at me when I dont include the request in the makespan closure. No idea why
    let svc = ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
        |_request: &Request<Body>| {
            tracing::info_span!("request", request_id = Uuid::new_v4().to_string())
        },
    ));

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(svc)
        .with_state(connection_state);

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    Ok(server)
}
