use config::ServiceConfig;

mod collectors;
mod communication;
mod config;
mod error;
mod service;

#[cfg(target_os = "linux")]
mod platform {
    pub mod config;
    pub mod error;

    pub mod linux;
    pub use linux as native;
}

#[cfg(target_os = "macos")]
mod platform {
    pub mod config;
    pub mod error;

    pub mod macos;
    pub use macos as native;
}

#[cfg(target_os = "windows")]
mod platform {
    pub mod config;
    pub mod error;

    pub mod windows;
    pub use windows as native;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let service_config = ServiceConfig::load_from_env_or_file();
    let service_manager = service::ServiceManager::init(service_config)?;

    service_manager.run().await?;
    Ok(())
}
