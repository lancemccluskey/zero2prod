use std::net::SocketAddr;

use axum::{extract::Path, response::IntoResponse, routing::get, Router};

async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello {name}!")
}

async fn hello_world() -> &'static str {
    "Hello World!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/:name", get(greet))
        .route("/", get(hello_world))
        .route("/health_check", get(|| async {}));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
