use crate::collectors::CollectorManager;
use crate::communication::CommunicationManager;
use crate::config::ServiceConfig;
use crate::error::ServiceError;

pub struct ServiceManager {
    communication_manager: CommunicationManager,
    collector_manager: CollectorManager,
}

impl ServiceManager {
    pub async fn run(config: ServiceConfig) -> Result<(), ServiceError> {
        todo!()
    }
}
