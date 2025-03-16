//! Collector manager for coordinating all system monitoring collectors
//!
//! The CollectorManager initializes and manages the lifecycle of all collector
//! instances, handling their configuration and data distribution.

use super::Collector;
use super::{
    config::CollectionConfig, cpu::CpuCollector, error::CollectionError, gpu::GpuCollector,
    memory::MemoryCollector, network::NetworkCollector, process::ProcessCollector,
    storage::StorageCollector, system::SystemCollector,
};
use monitord_protocols::monitord::*;
use tokio::sync::broadcast::Sender;

/// Create a collector task that can be run in a tokio::select! statement
///
/// Follows a common pattern:
/// 1. Check if the collector is enabled
/// 2. Collect data
/// 3. Send to channel
/// 4. Sleep for the configured interval
///
/// Returns a future that can be used in a tokio::select! statement
macro_rules! collector_task {
    ($collector:expr, $tx:expr) => {
        async {
            loop {
                if !$collector.config().enabled {
                    return Err::<(), CollectionError>(CollectionError::Disabled);
                }
                let collected_data = $collector.collect()?;
                let _ = $tx.send(collected_data);
                tokio::time::sleep($collector.config().interval.to_std().unwrap()).await;
            }
        }
    };
}

/// Manager for all system monitoring collectors
///
/// Coordinates the initialization, configuration, and operation of all collector
/// instances in the system, providing broadcast channels for distributing collected data.
pub struct CollectorManager {
    cpu_collector: CpuCollector,
    pub cpu_tx: Sender<CpuInfo>,

    memory_collector: MemoryCollector,
    pub memory_tx: Sender<MemoryInfo>,

    gpu_collector: GpuCollector,
    pub gpu_tx: Sender<Vec<GpuInfo>>,

    network_collector: NetworkCollector,
    pub network_tx: Sender<Vec<NetworkInfo>>,

    process_collector: ProcessCollector,
    pub process_tx: Sender<Vec<ProcessInfo>>,

    storage_collector: StorageCollector,
    pub storage_tx: Sender<Vec<StorageInfo>>,

    system_collector: SystemCollector,
    pub system_tx: Sender<SystemInfo>,
}

impl CollectorManager {
    /// Initialize a new collector manager with the provided configuration
    ///
    /// Creates all collector instances and their associated broadcast channels
    /// for distributing collected data to subscribers.
    pub fn init(config: CollectionConfig) -> Result<Self, CollectionError> {
        let (cpu_tx, _) = tokio::sync::broadcast::channel(1);
        let (memory_tx, _) = tokio::sync::broadcast::channel(1);
        let (gpu_tx, _) = tokio::sync::broadcast::channel(1);
        let (network_tx, _) = tokio::sync::broadcast::channel(1);
        let (process_tx, _) = tokio::sync::broadcast::channel(1);
        let (storage_tx, _) = tokio::sync::broadcast::channel(1);
        let (system_tx, _) = tokio::sync::broadcast::channel(1);
        Ok(Self {
            cpu_collector: CpuCollector::new(config.cpu_config)?,
            cpu_tx,
            memory_collector: MemoryCollector::new(config.memory_config)?,
            memory_tx,
            gpu_collector: GpuCollector::new(config.gpu_config)?,
            gpu_tx,
            network_collector: NetworkCollector::new(config.net_config)?,
            network_tx,
            process_collector: ProcessCollector::new(config.proc_config)?,
            process_tx,
            storage_collector: StorageCollector::new(config.disk_config)?,
            storage_tx,
            system_collector: SystemCollector::new(config.sys_config)?,
            system_tx,
        })
    }

    /// Run all enabled collectors in parallel
    ///
    /// Each collector runs in its own async task, collecting data at the configured
    /// interval and broadcasting it through its associated channel.
    pub async fn run(&mut self) -> Result<(), CollectionError> {
        tokio::select! {
            res = collector_task!(&mut self.cpu_collector, &self.cpu_tx) => {
                res?;
            }
            res = collector_task!(&mut self.memory_collector, &self.memory_tx) => {
                res?;
            }
            res = collector_task!(&mut self.gpu_collector, &self.gpu_tx) => {
                res?;
            }
            res = collector_task!(&mut self.network_collector, &self.network_tx) => {
                res?;
            }
            res = collector_task!(&mut self.process_collector, &self.process_tx) => {
                res?;
            }
            res = collector_task!(&mut self.storage_collector, &self.storage_tx) => {
                res?;
            }
            res = collector_task!(&mut self.system_collector, &self.system_tx) => {
                res?;
            }
        }
        Ok(())
    }
}
