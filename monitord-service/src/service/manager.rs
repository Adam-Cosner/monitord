use crate::collectors::CollectorManager;
use crate::communication::CommunicationManager;
use crate::config::ServiceConfig;
use crate::error::ServiceError;

pub struct ServiceManager {
    communication_manager: CommunicationManager,
    collector_manager: CollectorManager,
}

impl ServiceManager {
    pub fn init(config: ServiceConfig) -> Result<Self, ServiceError> {
        let communication_manager = match CommunicationManager::init(config.communication_config) {
            Ok(manager) => manager,
            Err(e) => return Err(ServiceError::CommunicationError(e)),
        };
        let collector_manager = match CollectorManager::init(config.collection_config) {
            Ok(manager) => manager,
            Err(e) => return Err(ServiceError::CollectionError(e)),
        };
        match crate::platform::native::register_service(config.platform_config) {
            Ok(_) => {}
            Err(e) => return Err(ServiceError::PlatformError(e)),
        }
        Ok(Self {
            communication_manager,
            collector_manager,
        })
    }

    pub async fn run(mut self) -> Result<(), ServiceError> {
        let cpu_rx = self.collector_manager.cpu_tx.subscribe();
        let memory_rx = self.collector_manager.memory_tx.subscribe();
        let iceoryx_subscription_rx = self
            .communication_manager
            .iceoryx_subscription_tx
            .subscribe();
        tokio::select! {
            res = self.collector_manager.run() => match res {
                Ok(_) => {}
                Err(e) => return Err(ServiceError::CollectionError(e)),
            },
            res = self.communication_manager.run(iceoryx_subscription_rx, cpu_rx, memory_rx) => {
                match res {
                    Ok(_) => {}
                    Err(e) => return Err(ServiceError::CommunicationError(e)),
                }
            }
        }
        Ok(())
    }
}
