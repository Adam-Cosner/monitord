#[derive(Debug, Clone)]
pub struct SystemCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}

impl Default for SystemCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
        }
    }
}
