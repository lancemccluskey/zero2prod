use std::net::TcpListener;

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, IntoMakeService},
    Router, Server,
};
use hyper::server::conn::AddrIncoming;

async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello {name}!")
}

async fn hello_world() -> &'static str {
    "Hello World!"
}

pub fn run(
    listener: TcpListener,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let app = Router::new()
        .route("/:name", get(greet))
        .route("/", get(hello_world))
        .route("/health_check", get(|| async {}));

    println!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    Ok(server)
}
