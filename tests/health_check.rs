use std::net::{SocketAddr, TcpListener};

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr).expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).unwrap();

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
