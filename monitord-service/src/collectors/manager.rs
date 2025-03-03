use super::Collector;
use super::{
    config::CollectionConfig, cpu::CpuCollector, error::CollectionError, memory::MemoryCollector,
};
use monitord_protocols::monitord::*;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::debug;

pub struct CollectorManager {
    cpu_collector: CpuCollector,
    pub cpu_tx: Sender<CpuInfo>,

    memory_collector: MemoryCollector,
    pub memory_tx: Sender<MemoryInfo>,
}

impl CollectorManager {
    pub fn init(config: CollectionConfig) -> Result<Self, CollectionError> {
        let (cpu_tx, _) = tokio::sync::broadcast::channel(1);
        let (memory_tx, _) = tokio::sync::broadcast::channel(1);
        Ok(Self {
            cpu_collector: CpuCollector::new(config.cpu_config)?,
            cpu_tx,
            memory_collector: MemoryCollector::new(config.memory_config)?,
            memory_tx,
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

        }
        Ok(())
    }
}
