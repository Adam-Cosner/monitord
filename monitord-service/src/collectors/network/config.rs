#[derive(Debug, Clone)]
pub struct NetworkCollectorConfig {
    pub(crate) enabled: bool,
    pub(crate) interval: chrono::Duration,
}

impl Default for NetworkCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
        }
    }
}
