use super::Collector;
use super::{
    config::CollectionConfig, cpu::CpuCollector, error::CollectionError, memory::MemoryCollector,
};
use crate::collectors::gpu::GpuCollector;
use crate::collectors::network::NetworkCollector;
use monitord_protocols::monitord::*;
use tokio::sync::broadcast::Sender;
use tracing::info;

pub struct CollectorManager {
    cpu_collector: CpuCollector,
    pub cpu_tx: Sender<CpuInfo>,

    memory_collector: MemoryCollector,
    pub memory_tx: Sender<MemoryInfo>,

    gpu_collector: GpuCollector,
    pub gpu_tx: Sender<Vec<GpuInfo>>,

    network_collector: NetworkCollector,
    pub network_tx: Sender<Vec<NetworkInfo>>,
}

impl CollectorManager {
    pub fn init(config: CollectionConfig) -> Result<Self, CollectionError> {
        let (cpu_tx, _) = tokio::sync::broadcast::channel(1);
        let (memory_tx, _) = tokio::sync::broadcast::channel(1);
        let (gpu_tx, _) = tokio::sync::broadcast::channel(1);
        let (network_tx, _) = tokio::sync::broadcast::channel(1);
        Ok(Self {
            cpu_collector: CpuCollector::new(config.cpu_config)?,
            cpu_tx,
            memory_collector: MemoryCollector::new(config.memory_config)?,
            memory_tx,
            gpu_collector: GpuCollector::new(config.gpu_config)?,
            gpu_tx,
            network_collector: NetworkCollector::new(config.net_config)?,
            network_tx,
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

        }
        Ok(())
    }
}
