use crate::error::CollectionError;
use monitord_protocols::monitord::StorageInfo;

pub mod config;

pub struct StorageCollector {
    config: config::StorageCollectorConfig,
}

impl StorageCollector {
    pub fn new(config: config::StorageCollectorConfig) -> Result<Self, CollectionError> {
        Ok(Self { config })
    }
}

impl super::Collector for StorageCollector {
    type CollectedData = Vec<StorageInfo>;
    type CollectorConfig = config::StorageCollectorConfig;

    fn name(&self) -> &'static str {
        "storage"
    }

    fn config(&self) -> &Self::CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        Ok(vec![])
    }
}
