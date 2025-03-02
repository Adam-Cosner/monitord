#[derive(Debug, Clone)]
pub struct CollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}
