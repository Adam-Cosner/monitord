use crate::config::{CommunicationConfig, PlatformConfig};
use crate::error::ServiceError;
use monitord_collectors::config::CollectorsConfig;
use tracing::error;

#[derive(Debug, Clone, Default)]
pub struct ServiceConfig {
    pub collection_config: CollectorsConfig,
    pub communication_config: CommunicationConfig,
    pub platform_config: PlatformConfig,
}

impl ServiceConfig {
    pub(crate) fn load_from_env_or_file() -> Result<Self, ServiceError> {
        // TODO: Read from env or config file
        let mut builder = config::Config::builder()
            .set_default("service.update_interval_ms", 1000)?
            .set_default("service.max_clients", 10)?
            .set_default("service.enable_logging", true)?
            .set_default("service.log_level", "INFO")?
            // Collection config defaults
            // Communication config defaults
            .set_default("transport.type", "nng")?;

        // Add configuration from a file if specified via environment variable
        if let Ok(config_path) = std::env::var("MONITORD_CONFIG") {
            builder = builder.add_source(config::File::with_name(&config_path));
        } else {
            // Check standard config locations if no env var is set
            builder = builder
                .add_source(config::File::with_name("/etc/monitord/config").required(false))
                .add_source(config::File::with_name("~/.config/monitord/config").required(false))
                .add_source(config::File::with_name("config").required(false));
        }

        // Add environment variable source
        // Environment variables should be prefixed with MONITORD_
        // e.g., MONITORD_SERVICE_MAX_CLIENTS=20
        builder = builder.add_source(config::Environment::with_prefix("MONITORD").separator("_"));

        // Try to build the config
        let config_result = builder.build();
        let config = match config_result {
            Ok(config) => config,
            Err(e) => {
                error!("Error loading configuration: {}. Using defaults", e);
                // Return default config on error
                return Ok(Self::default());
            }
        };

        // Convert durations from milliseconds to chrono::Duration
        let system_interval = chrono::Duration::milliseconds(
            config
                .get_int("collectors.system.interval_ms")
                .unwrap_or(1000),
        );
        let cpu_interval = chrono::Duration::milliseconds(
            config.get_int("collectors.cpu.interval_ms").unwrap_or(1000),
        );
        let memory_interval = chrono::Duration::milliseconds(
            config
                .get_int("collectors.memory.interval_ms")
                .unwrap_or(1000),
        );
        let gpu_interval = chrono::Duration::milliseconds(
            config.get_int("collectors.gpu.interval_ms").unwrap_or(1000),
        );
        let network_interval = chrono::Duration::milliseconds(
            config
                .get_int("collectors.network.interval_ms")
                .unwrap_or(1000),
        );
        let process_interval = chrono::Duration::milliseconds(
            config
                .get_int("collectors.process.interval_ms")
                .unwrap_or(1000),
        );
        let storage_interval = chrono::Duration::milliseconds(
            config
                .get_int("collectors.storage.interval_ms")
                .unwrap_or(1000),
        );

        // Build collector configs
        let system_config = monitord_collectors::config::SystemCollectorConfig {
            enabled: config.get_bool("collectors.system.enabled").unwrap_or(true),
            interval_ms: 1000,
            collect_load_avg: true,
            collect_open_files: true,
            collect_thread_count: true,
        };

        let cpu_config = monitord_collectors::config::CpuCollectorConfig {
            enabled: config.get_bool("collectors.cpu.enabled").unwrap_or(true),
            interval_ms: 1000,
            collect_per_core: true,
            collect_cache_info: false,
            collect_temperature: false,
            collect_frequency: false,
        };

        let memory_config = monitord_collectors::config::MemoryCollectorConfig {
            enabled: config.get_bool("collectors.memory.enabled").unwrap_or(true),
            interval_ms: 1000,
            collect_dram_info: true,
            collect_swap_info: true,
        };

        let gpu_config = monitord_collectors::config::GpuCollectorConfig {
            enabled: config.get_bool("collectors.gpu.enabled").unwrap_or(true),
            interval_ms: 1000,
            collect_nvidia: true,
            collect_amd: true,
            collect_intel: true,
            collect_processes: true,
        };

        let network_config = monitord_collectors::config::NetworkCollectorConfig {
            enabled: config
                .get_bool("collectors.network.enabled")
                .unwrap_or(true),
            interval_ms: 1000,
            collect_packets: true,
            collect_errors: true,
        };

        let process_config = monitord_collectors::config::ProcessCollectorConfig {
            enabled: config
                .get_bool("collectors.process.enabled")
                .unwrap_or(true),
            interval_ms: 1000,
            max_processes: 1000000,
            collect_command_line: false,
            collect_environment: false,
            collect_io_stats: false,
        };

        let storage_config = monitord_collectors::config::StorageCollectorConfig {
            enabled: config
                .get_bool("collectors.storage.enabled")
                .unwrap_or(true),
            interval_ms: 1000,
            collect_smart: false,
            collect_io_stats: true,
        };

        // Combine all collector configs
        let collection_config = CollectorsConfig {
            system: system_config,
            cpu: cpu_config,
            memory: memory_config,
            gpu: gpu_config,
            storage: storage_config,
            network: network_config,
            process: process_config,
        };

        // Build transport config based on type
        let transport_type = config
            .get_string("transport.type")
            .unwrap_or_else(|_| "intra".to_string());
        let transport_config = match transport_type.as_str() {
            "nng" => {
                let nng_config = monitord_transport::config::NngConfig {
                    transport: config
                        .get_string("transport.nng.transport")
                        .unwrap_or_else(|_| "ipc".to_string()),
                    url: config
                        .get_string("transport.nng.url")
                        .unwrap_or_else(|_| "/tmp/monitord".to_string()),
                    timeout_ms: config.get_int("transport.nng.timeout_ms").unwrap_or(1000) as u32,
                };
                monitord_transport::config::TransportType::Nng(nng_config)
            }
            "iceoryx" => {
                let iceoryx_config = monitord_transport::config::IceoryxConfig {
                    service_name: config
                        .get_string("transport.iceoryx.service_name")
                        .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
                    buffer_size: config
                        .get_int("transport.iceoryx.buffer_size")
                        .unwrap_or(1024 * 1024) as usize,
                };
                monitord_transport::config::TransportType::Iceoryx(iceoryx_config)
            }
            "grpc" => monitord_transport::config::TransportType::Grpc,
            _ => monitord_transport::config::TransportType::Intra,
        };

        let communication_config = CommunicationConfig {
            transport_config: monitord_transport::config::TransportConfig { transport_config },
        };

        // Build platform config
        // Currently empty, but can be extended later
        let platform_config = crate::platform::config::PlatformConfig::default();

        Ok(Self {
            collection_config,
            communication_config,
            platform_config,
        })
    }
}
