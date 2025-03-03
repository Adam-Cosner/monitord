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
    let service_config = ServiceConfig::load_from_env_or_file();
    service::ServiceManager::run(service_config).await?;
    Ok(())
}
