use std::path::Path;

use super::{config::CommunicationConfig, error::CommunicationError};

pub struct CommunicationManager {
    iceoryx: Option<super::iceoryx::IceoryxManager>,
    grpc: Option<super::grpc::GrpcService>,
}

impl CommunicationManager {
    pub fn init(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        todo!()
    }

    pub fn publish(&self, data: &[u8]) -> Result<(), CommunicationError> {
        todo!()
    }
}
