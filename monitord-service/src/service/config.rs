use crate::config::{
    CommunicationConfig, CpuCollectorConfig, GpuCollectorConfig, IceoryxConfig,
    MemoryCollectorConfig, NetworkCollectorConfig, ProcessCollectorConfig, StorageCollectorConfig,
    SubscriptionConfig, SystemCollectorConfig,
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
                gpu_config: GpuCollectorConfig::default(),
                disk_config: StorageCollectorConfig {
                    enabled: true,
                    interval: chrono::Duration::seconds(1),
                },
                net_config: NetworkCollectorConfig {
                    enabled: true,
                    interval: chrono::Duration::seconds(1),
                },
                proc_config: ProcessCollectorConfig {
                    enabled: true,
                    interval: chrono::Duration::seconds(1),
                },
            },
            communication_config: CommunicationConfig::default(),
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
