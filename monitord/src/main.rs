mod config;
mod metrics;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Monitord daemon executed, does nothing for now");

    // Load config from file
    let config = config::load_config_from_file("/etc/monitord.toml");

    // Run the server
    let mut server = server::Server::new(&config);

    server.run().await?;

    Ok(())
}
