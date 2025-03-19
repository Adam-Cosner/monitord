#[derive(Debug, Clone)]
pub struct GpuCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
    pub amd_enabled: bool,
    pub nvidia_enabled: bool,
    pub intel_enabled: bool,
}

impl Default for GpuCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
            amd_enabled: true,
            nvidia_enabled: true,
            intel_enabled: true,
        }
    }
}
