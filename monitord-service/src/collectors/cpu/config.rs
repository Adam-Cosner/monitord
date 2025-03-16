#[derive(Debug, Clone)]
pub struct CpuCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
}

impl Default for CpuCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
        }
    }
}
