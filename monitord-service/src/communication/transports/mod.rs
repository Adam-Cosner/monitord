//! Transport implementations for different communication protocols

pub(crate) mod iceoryx;
pub(crate) mod grpc;
mod common;


use crate::communication::config::CommunicationConfig;
use crate::communication::core::traits::Transport;
use crate::communication::error::CommunicationError;

/// Create all configured transports
pub fn create_transports(config: &CommunicationConfig) -> Result<Vec<Box<dyn Transport>>, CommunicationError> {
    let mut transports = Vec::new();

    // Initialize Iceoryx transport if configured
    if let Some(iceoryx_config) = &config.iceoryx {
        let iceoryx = iceoryx::IceoryxTransport::new(iceoryx_config.clone())?;
        transports.push(Box::new(iceoryx) as Box<dyn Transport>);
    }

    // Initialize gRPC transport if configured
    if let Some(grpc_config) = &config.grpc {
        let grpc = grpc::GrpcTransport::new(grpc_config.clone())?;
        transports.push(Box::new(grpc) as Box<dyn Transport>);
    }

    if transports.is_empty() {
        return Err(CommunicationError::InvalidConfiguration(
            "No transport mechanisms configured".into()
        ));
    }

    Ok(transports)
}