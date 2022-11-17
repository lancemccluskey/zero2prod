use std::{net::TcpListener, sync::Arc};

use axum::{
    routing::{get, post, IntoMakeService},
    Router, RouterService, Server,
};
use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;

use crate::routes::*;

pub fn run(
    listener: TcpListener,
    connection: PgPool,
) -> Result<Server<AddrIncoming, IntoMakeService<RouterService>>, std::io::Error> {
    let connection_state = Arc::new(connection);

    let app = Router::with_state(connection_state)
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    println!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    Ok(server)
}
