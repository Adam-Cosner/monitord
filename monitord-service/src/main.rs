use std::env;

use config::ServiceConfig;
use platform::config::PlatformConfig;

mod communication;
mod config;
mod error;
mod service;
mod platform;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Process command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check for --register-service flag
    if args.len() > 1 && args[1] == "--register-service" {
        println!("Registering monitord as a system service...");
        
        let mut platform_config = PlatformConfig::default();
        
        // Override defaults with command-line arguments if provided
        for i in 2..args.len() {
            let arg = &args[i];
            if let Some((key, value)) = arg.split_once('=') {
                match key {
                    "--name" => platform_config.service_name = value.to_string(),
                    "--description" => platform_config.description = value.to_string(),
                    "--path" => platform_config.executable_path = value.to_string(),
                    "--user" => platform_config.user = Some(value.to_string()),
                    "--group" => platform_config.group = Some(value.to_string()),
                    "--workdir" => platform_config.working_directory = Some(value.to_string()),
                    "--init" => platform_config.init_system = match value.to_lowercase().as_str() {
                        "systemd" => Some(platform::config::InitSystem::SystemD),
                        "sysvinit" => Some(platform::config::InitSystem::SysVInit),
                        "openrc" => Some(platform::config::InitSystem::OpenRC),
                        "runit" => Some(platform::config::InitSystem::Runit),
                        "auto" => Some(platform::config::InitSystem::Auto),
                        _ => {
                            eprintln!("Unknown init system: {}. Using auto detection.", value);
                            Some(platform::config::InitSystem::Auto)
                        }
                    },
                    _ => eprintln!("Unknown option: {}", key),
                }
            }
        }
        
        // Register the service
        #[cfg(target_os = "linux")]
        {
            match platform::linux::register_service(platform_config) {
                Ok(_) => println!("Service registration complete."),
                Err(e) => {
                    eprintln!("Service registration failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            println!("Service registration not implemented for this platform.");
        }
        
        return Ok(());
    }
    
    // Normal service startup
    let service_config = ServiceConfig::load_from_env_or_file()?;
    let service_manager = service::ServiceManager::init(service_config)?;

    service_manager.run().await?;
    Ok(())
}