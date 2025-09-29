#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Monitord daemon executed, does nothing for now");

    std::future::pending::<()>().await;
}
