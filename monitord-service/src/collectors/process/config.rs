#[derive(Debug, Clone)]
pub struct ProcessCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}

impl Default for ProcessCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
        }
    }
}
