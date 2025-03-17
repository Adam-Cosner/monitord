use crate::config::{CollectionConfig, PlatformConfig, CommunicationConfig};

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
