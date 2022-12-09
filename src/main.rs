use zero2prod::{
    configuration::get_configuration,
    startup::build,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup tracing layers and initialize the tracing subscriber
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    // Read configuration settings from config files and then build and start the server
    let configuration = get_configuration().expect("Failed to read configuration.");
    let server = build(configuration).await?;
    server.await.unwrap();
    Ok(())
}
