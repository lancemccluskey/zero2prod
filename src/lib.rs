use std::net::SocketAddr;

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

pub fn run() -> Result<Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let app = Router::new()
        .route("/:name", get(greet))
        .route("/", get(hello_world))
        .route("/health_check", get(|| async {}));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    Ok(server)
}
