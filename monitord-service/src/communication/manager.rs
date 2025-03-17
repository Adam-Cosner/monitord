use crate::config::CommunicationConfig;
use crate::error::CommunicationError;
use monitord_protocols::monitord::*;
use monitord_transport::TransportManager;
use tokio::sync::broadcast::Receiver;

pub struct CommunicationManager {
    transport: TransportManager,
}

impl CommunicationManager {
    pub fn new(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        todo!()
    }

    pub async fn run(
        &self,
        cpu_rx: Receiver<CpuInfo>,
        memory_rx: Receiver<MemoryInfo>,
        gpu_rx: Receiver<Vec<GpuInfo>>,
        net_rx: Receiver<Vec<NetworkInfo>>,
        proc_rx: Receiver<Vec<ProcessInfo>>,
        storage_rx: Receiver<Vec<StorageInfo>>,
        system_rx: Receiver<SystemInfo>,
    ) -> Result<(), CommunicationError> {
        todo!()
    }
}
