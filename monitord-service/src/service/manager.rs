use crate::collectors::CollectorManager;
use crate::communication::CommunicationManager;
use crate::config::ServiceConfig;
use crate::error::ServiceError;
use std::sync::Arc;

pub struct ServiceManager {
    communication_manager: CommunicationManager,
    collector_manager: CollectorManager,
}

impl ServiceManager {
    pub fn init(config: ServiceConfig) -> Result<Self, ServiceError> {
        let communication_manager = match CommunicationManager::new(
            config.communication_config,
            Arc::new(crate::communication::handlers::ProtobufHandler {}),
        ) {
            Ok(manager) => manager,
            Err(e) => return Err(ServiceError::Communication(e)),
        };
        let collector_manager = match CollectorManager::init(config.collection_config) {
            Ok(manager) => manager,
            Err(e) => return Err(ServiceError::Collection(e)),
        };
        match crate::platform::native::register_service(config.platform_config) {
            Ok(_) => {}
            Err(e) => return Err(ServiceError::Platform(e)),
        }
        Ok(Self {
            communication_manager,
            collector_manager,
        })
    }

    pub async fn run(mut self) -> Result<(), ServiceError> {
        let cpu_rx = self.collector_manager.cpu_tx.subscribe();
        let memory_rx = self.collector_manager.memory_tx.subscribe();
        let gpu_rx = self.collector_manager.gpu_tx.subscribe();
        let net_rx = self.collector_manager.network_tx.subscribe();
        let proc_rx = self.collector_manager.process_tx.subscribe();
        let storage_rx = self.collector_manager.storage_tx.subscribe();
        let system_rx = self.collector_manager.system_tx.subscribe();
        tokio::select! {
            res = self.collector_manager.run() => match res {
                Ok(_) => {}
                Err(e) => return Err(ServiceError::Collection(e)),
            },
            res = self.communication_manager.run(cpu_rx, memory_rx, gpu_rx, net_rx, proc_rx, storage_rx, system_rx) => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(ServiceError::Communication(e)),
                }
            }
        }
        Ok(())
    }
}
