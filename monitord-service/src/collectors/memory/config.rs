#[derive(Debug, Clone)]
pub struct MemoryCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}
