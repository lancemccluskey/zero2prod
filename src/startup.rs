use std::net::TcpListener;

use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use hyper::server::conn::AddrIncoming;

use crate::routes::*;

pub fn run(
    listener: TcpListener,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    println!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    Ok(server)
}
