use std::net::{SocketAddr, TcpListener};

use zero2prod::run;

#[tokio::main]
async fn main() {
    let listener =
        TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000))).expect("Failed to bind port");

    run(listener).unwrap().await.unwrap();
}
