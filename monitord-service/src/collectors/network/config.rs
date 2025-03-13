#[derive(Debug, Clone)]
pub struct NetworkCollectorConfig {
    pub(crate) enabled: bool,
    pub(crate) interval: chrono::Duration,
    // todo
}
