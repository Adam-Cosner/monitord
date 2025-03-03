use prost::Message;

use monitord_protocols::monitord::{CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo};

use super::{config::CommunicationConfig, error::CommunicationError};

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

    pub fn publish_system_info(&self, info: SystemInfo) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_system_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }

    pub fn publish_cpu_info(&self, info: CpuInfo) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_cpu_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }

    pub fn publish_memory_info(&self, info: MemoryInfo) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_memory_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }

    pub fn publish_gpu_info(&self, info: &[GpuInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_gpu_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }

    pub fn publish_network_info(&self, info: &[NetworkInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_net_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }

    pub fn publish_storage_info(&self, info: &[StorageInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_storage_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }

    pub fn publish_process_info(&self, info: &[ProcessInfo]) -> Result<(), CommunicationError> {
        if let Some(iceoryx) = &self.iceoryx {
            iceoryx.send_process_info(info)?;
        }

        // Add gRPC implementation when available
        
        Ok(())
    }
}
