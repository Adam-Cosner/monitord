#[derive(Debug, Clone)]
pub struct StorageCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}

impl Default for StorageCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
        }
    }
}
