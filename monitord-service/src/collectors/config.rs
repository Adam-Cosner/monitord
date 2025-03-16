pub use super::{
    cpu::config::CpuCollectorConfig, gpu::config::GpuCollectorConfig,
    memory::config::MemoryCollectorConfig, network::config::NetworkCollectorConfig,
    process::config::ProcessCollectorConfig, storage::config::StorageCollectorConfig,
    system::config::SystemCollectorConfig,
};

#[derive(Debug, Clone, Default)]
pub struct CollectionConfig {
    pub sys_config: SystemCollectorConfig,
    pub cpu_config: CpuCollectorConfig,
    pub memory_config: MemoryCollectorConfig,
    pub gpu_config: GpuCollectorConfig,
    pub disk_config: StorageCollectorConfig,
    pub net_config: NetworkCollectorConfig,
    pub proc_config: ProcessCollectorConfig,
}
