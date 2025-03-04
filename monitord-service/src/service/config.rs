use crate::config::{
    CommunicationConfig, CpuCollectorConfig, GpuCollectorConfig, IceoryxConfig,
    MemoryCollectorConfig, NetworkCollectorConfig, ProcessCollectorConfig, StorageCollectorConfig,
    SystemCollectorConfig,
};
use crate::{config::CollectionConfig, config::PlatformConfig};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub collection_config: CollectionConfig,
    pub communication_config: CommunicationConfig,
    pub platform_config: PlatformConfig,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        ServiceConfig {
            collection_config: CollectionConfig {
                sys_config: SystemCollectorConfig {},
                cpu_config: CpuCollectorConfig {
                    enabled: true,
                    interval: chrono::Duration::seconds(1),
                },
                memory_config: MemoryCollectorConfig {
                    enabled: true,
                    interval: chrono::Duration::seconds(1),
                },
                gpu_config: GpuCollectorConfig {
                    amd_enabled: false,
                    nvidia_enabled: false,
                    intel_enabled: false,
                },
                disk_config: StorageCollectorConfig {},
                net_config: NetworkCollectorConfig {},
                proc_config: ProcessCollectorConfig {},
            },
            communication_config: CommunicationConfig {
                iceoryx: Some(IceoryxConfig {
                    service_name: "monitord".to_string(),
                    buffer_size: 1024 * 1024,
                }),
                grpc: None,
            },
            platform_config: PlatformConfig {},
        }
    }
}
impl ServiceConfig {
    pub(crate) fn load_from_env_or_file() -> Self {
        // TODO: Read from env or config file
        Self::default()
    }
}
