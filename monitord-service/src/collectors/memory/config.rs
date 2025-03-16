#[derive(Debug, Clone)]
pub struct MemoryCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}

impl Default for MemoryCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
        }
    }
}
