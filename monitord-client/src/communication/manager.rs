use crate::communication::transport::TransportVariant;
use crate::config::CommunicationConfig;
use crate::error::CommunicationError;

pub struct CommunicationManager {
    transport: TransportVariant
}

impl CommunicationManager {
    pub fn new(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        todo!()
    }
}