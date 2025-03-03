use super::{config::CommunicationConfig, error::CommunicationError};
use monitord_protocols::monitord::{
    CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo,
};
use prost::Message;
use tokio::sync::broadcast::Receiver;
use tracing::debug;

pub struct CommunicationManager {
    iceoryx: Option<super::iceoryx::IceoryxManager>,
    grpc: Option<super::grpc::GrpcService>,
}

impl CommunicationManager {
    pub fn init(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        let iceoryx = if let Some(iceoryx_config) = config.iceoryx {
            Some(super::iceoryx::IceoryxManager::init(iceoryx_config)?)
        } else {
            None
        };

        let grpc = if let Some(_grpc_config) = config.grpc {
            // Implementation for gRPC would go here
            None
        } else {
            None
        };

        Ok(Self { iceoryx, grpc })
    }

    pub async fn run(
        &self,
        mut cpu_rx: Receiver<CpuInfo>,
        mut memory_rx: Receiver<MemoryInfo>,
    ) -> Result<(), CommunicationError> {
        tokio::select! {
            cpu_info = async move {
                loop {
                    match cpu_rx.recv().await {
                        Ok(info) => self.publish_cpu_info(info)?,
                        Err(e) => return Err::<(), CommunicationError>(CommunicationError::ReceiveError(e.to_string())),
                    }
                }
            } => {
                match cpu_info {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            memory_info = async move {
                loop {
                    match memory_rx.recv().await {
                        Ok(info) => self.publish_memory_info(info)?,
                        Err(e) => return Err::<(), CommunicationError>(CommunicationError::ReceiveError(e.to_string())),
                    }
                }
            } => {
                match memory_info {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    fn publish_system_info(&self, info: SystemInfo) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_system_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }

    fn publish_cpu_info(&self, info: CpuInfo) -> Result<(), CommunicationError> {
        debug!("CPU info: {:?}", info);

        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_cpu_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }

    fn publish_memory_info(&self, info: MemoryInfo) -> Result<(), CommunicationError> {
        debug!("Memory info: {:?}", info);

        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_memory_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }

    fn publish_gpu_info(&self, info: &[GpuInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_gpu_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }

    fn publish_network_info(&self, info: &[NetworkInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_net_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }

    fn publish_storage_info(&self, info: &[StorageInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_storage_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }

    fn publish_process_info(&self, info: &[ProcessInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_process_info(info)?;
        }

        // Add gRPC implementation when available

        Ok(())
    }
}
