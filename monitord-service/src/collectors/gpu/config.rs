#[derive(Debug, Clone)]
pub struct GpuCollectorConfig {
    pub enabled: bool,
    pub interval: chrono::Duration,
    pub amd_enabled: bool,
    pub nvidia_enabled: bool,
    pub intel_enabled: bool,
    pub collect_processes: bool,
    pub max_processes_per_gpu: usize,
}

impl Default for GpuCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: chrono::Duration::seconds(1),
            amd_enabled: true,
            nvidia_enabled: true,
            intel_enabled: true,
            collect_processes: true,
            max_processes_per_gpu: 10,
        }
    }
}
