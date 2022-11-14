use std::net::{SocketAddr, TcpListener};

use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = SocketAddr::from(([127, 0, 0, 1], configuration.application_port));
    let listener = TcpListener::bind(address).expect("Failed to bind port");

    run(listener).unwrap().await.unwrap();
}
