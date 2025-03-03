use std::sync::{Arc, Mutex};

use super::{
    config::CollectionConfig, cpu::CpuCollector, error::CollectionError, memory::MemoryCollector,
};

pub struct CollectorManager {
    cpu_collector: Arc<Mutex<CpuCollector>>,
    memory_collector: Arc<Mutex<MemoryCollector>>,
}

impl CollectorManager {
    pub fn init(config: CollectionConfig) -> Result<Self, CollectionError> {
        
        todo!()
    }

    pub async fn run(&mut self) -> Result<(), CollectionError> {
        todo!()
    }
}
