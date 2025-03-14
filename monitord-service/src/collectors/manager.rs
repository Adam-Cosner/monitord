use super::Collector;
use super::{
    config::CollectionConfig, cpu::CpuCollector, error::CollectionError, memory::MemoryCollector,
};
use crate::collectors::gpu::GpuCollector;
use crate::collectors::network::NetworkCollector;
use crate::collectors::process::ProcessCollector;
use monitord_protocols::monitord::*;
use tokio::sync::broadcast::Sender;
use crate::collectors::storage::StorageCollector;
use crate::collectors::system::SystemCollector;

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

    pub async fn run(&mut self) -> Result<(), CollectionError> {
        tokio::select! {
            // CPU Collector
            res = async {
                loop {
                    if !self.cpu_collector.config().enabled {
                        return Err::<(), CollectionError>(CollectionError::Disabled);
                    }
                    let collected_data = self.cpu_collector.collect()?;
                    self.cpu_tx.send(collected_data).unwrap();
                    tokio::time::sleep(self.cpu_collector.config().interval.to_std().unwrap()).await;
                }
            } => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            // Memory collector
            res = async {
                loop {
                    if !self.memory_collector.config().enabled {
                        return Err::<(), CollectionError>(CollectionError::Disabled);
                    }
                    let collected_data = self.memory_collector.collect()?;
                    self.memory_tx.send(collected_data).unwrap();
                    tokio::time::sleep(self.memory_collector.config().interval.to_std().unwrap()).await;
                }
            } => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            // GPU Collector
            res = async {
                loop {
                    if !self.gpu_collector.config().enabled {
                        return Err::<(), CollectionError>(CollectionError::Disabled);
                    }
                    let collected_data = self.gpu_collector.collect()?;
                    self.gpu_tx.send(collected_data).unwrap();
                    tokio::time::sleep(self.gpu_collector.config().interval.to_std().unwrap()).await;
                }
            } => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            // Net Collector
            res = async {
                loop {
                    if !self.network_collector.config().enabled {
                        return Err::<(), CollectionError>(CollectionError::Disabled);
                    }
                    let collected_data = self.network_collector.collect()?;
                    self.network_tx.send(collected_data).unwrap();
                    tokio::time::sleep(self.network_collector.config().interval.to_std().unwrap()).await;
                }
            } => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            // Process Collector
            res = async {
                loop {
                    if !self.process_collector.config().enabled {
                        return Err::<(), CollectionError>(CollectionError::Disabled);
                    }
                    let collected_data = self.process_collector.collect()?;
                    self.process_tx.send(collected_data).unwrap();
                    tokio::time::sleep(self.process_collector.config().interval.to_std().unwrap()).await;
                }
            } => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            // Storage Collector
            res = async {
                loop {
                    if !self.storage_collector.config().enabled {
                        return Err::<(), CollectionError>(CollectionError::Disabled);
                    }
                    let collected_data = self.storage_collector.collect()?;
                    self.storage_tx.send(collected_data).unwrap();
                    tokio::time::sleep(self.storage_collector.config().interval.to_std().unwrap()).await;
                }
            } => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }

        }
        Ok(())
    }
}
