use crate::config::{
    CommunicationConfig, CpuCollectorConfig, GpuCollectorConfig, IceoryxConfig,
    MemoryCollectorConfig, NetworkCollectorConfig, ProcessCollectorConfig, StorageCollectorConfig,
    SubscriptionConfig, SystemCollectorConfig,
};
use crate::{config::CollectionConfig, config::PlatformConfig};

#[derive(Debug, Clone, Default)]
pub struct ServiceConfig {
    pub collection_config: CollectionConfig,
    pub communication_config: CommunicationConfig,
    pub platform_config: PlatformConfig,
}

impl ServiceConfig {
    pub(crate) fn load_from_env_or_file() -> Self {
        // TODO: Read from env or config file
        Self::default()
    }
}
