use crate::config::CommunicationConfig;
use crate::error::ServiceError;
use monitord_collectors::config::CollectorsConfig;
use tracing::error;

#[derive(Debug, Clone, Default)]
pub struct ServiceConfig {
    pub collection_config: CollectorsConfig,
    pub communication_config: CommunicationConfig,
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
            .set_default("grpc.server_address", "127.0.0.1:50051")?
            .set_default("grpc.timeout_ms", 3000)?;

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
        let system_interval = config
            .get_int("collectors.system.interval_ms")
            .unwrap_or(1000) as u64;
        let cpu_interval = config.get_int("collectors.cpu.interval_ms").unwrap_or(1000) as u64;
        let memory_interval = config
            .get_int("collectors.memory.interval_ms")
            .unwrap_or(1000) as u64;
        let gpu_interval = config.get_int("collectors.gpu.interval_ms").unwrap_or(1000) as u64;
        let network_interval = config
            .get_int("collectors.network.interval_ms")
            .unwrap_or(1000) as u64;
        let process_interval = config
            .get_int("collectors.process.interval_ms")
            .unwrap_or(1000) as u64;
        let storage_interval = config
            .get_int("collectors.storage.interval_ms")
            .unwrap_or(1000) as u64;

        // Build collector configs
        let system_config = monitord_collectors::config::SystemCollectorConfig {
            enabled: config.get_bool("collectors.system.enabled").unwrap_or(true),
            interval_ms: system_interval,
            collect_load_avg: true,
            collect_open_files: true,
            collect_thread_count: true,
        };

        let cpu_config = monitord_collectors::config::CpuCollectorConfig {
            enabled: config.get_bool("collectors.cpu.enabled").unwrap_or(true),
            interval_ms: cpu_interval,
            collect_per_core: true,
            collect_cache_info: false,
            collect_temperature: false,
            collect_frequency: false,
        };

        let memory_config = monitord_collectors::config::MemoryCollectorConfig {
            enabled: config.get_bool("collectors.memory.enabled").unwrap_or(true),
            interval_ms: memory_interval,
            collect_dram_info: true,
            collect_swap_info: true,
        };

        let gpu_config = monitord_collectors::config::GpuCollectorConfig {
            enabled: config.get_bool("collectors.gpu.enabled").unwrap_or(true),
            interval_ms: gpu_interval,
            collect_nvidia: true,
            collect_amd: true,
            collect_intel: true,
            collect_processes: true,
        };

        let network_config = monitord_collectors::config::NetworkCollectorConfig {
            enabled: config
                .get_bool("collectors.network.enabled")
                .unwrap_or(true),
            interval_ms: network_interval,
            collect_packets: true,
            collect_errors: true,
        };

        let process_config = monitord_collectors::config::ProcessCollectorConfig {
            enabled: config
                .get_bool("collectors.process.enabled")
                .unwrap_or(true),
            interval_ms: process_interval,
            max_processes: 1000000,
            collect_command_line: false,
            collect_environment: false,
            collect_io_stats: false,
        };

        let storage_config = monitord_collectors::config::StorageCollectorConfig {
            enabled: config
                .get_bool("collectors.storage.enabled")
                .unwrap_or(true),
            interval_ms: storage_interval,
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

        // Configure gRPC
        let grpc_config = crate::communication::config::GrpcConfig {
            server_address: config
                .get_string("grpc.server_address")
                .unwrap_or_else(|_| "127.0.0.1:50051".to_string()),
        };

        let communication_config = CommunicationConfig { grpc_config };

        Ok(Self {
            collection_config,
            communication_config,
        })
    }
}
