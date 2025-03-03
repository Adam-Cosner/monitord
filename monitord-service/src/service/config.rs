use crate::{config::CollectionConfig, platform::config::PlatformConfig};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub collection_config: CollectionConfig,
    pub platform_config: PlatformConfig,
}
impl ServiceConfig {
    pub(crate) fn load_from_env_or_file() -> Self {
        todo!()
    }
}
