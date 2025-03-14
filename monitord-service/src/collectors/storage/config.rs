#[derive(Debug, Clone)]
pub struct StorageCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}
