#[derive(Debug, Clone)]
pub struct ProcessCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}
