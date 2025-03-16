//! Transport implementations for different communication protocols

mod common;
pub(crate) mod grpc;
pub(crate) mod iceoryx;

use crate::communication::config::CommunicationConfig;
use crate::communication::core::traits::Transport;
use crate::communication::error::CommunicationError;
use std::sync::Arc;
use tracing::{info, warn};

/// Create all configured transports
///
/// Initializes and returns transport implementations based on the provided configuration.
/// Returns an error if no transports could be configured or initialization failed.
pub fn create_transports(
    config: &CommunicationConfig,
) -> Result<Vec<Arc<dyn Transport>>, CommunicationError> {
    let mut transports = Vec::new();

    // Initialize Iceoryx transport if configured
    if let Some(iceoryx_config) = &config.iceoryx {
        let mut iceoryx = iceoryx::IceoryxTransport::new(iceoryx_config.clone())?;
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { iceoryx.initialize().await })
        }) {
            Ok(_) => {
                info!("Initialized transport: {}", iceoryx.name());
            }
            Err(e) => {
                warn!("Failed to initialize transport {}: {}", iceoryx.name(), e);
            }
        }
        transports.push(Arc::new(iceoryx) as Arc<dyn Transport>);
    }

    // Initialize gRPC transport if configured
    if let Some(grpc_config) = &config.grpc {
        let mut grpc = grpc::GrpcTransport::new(grpc_config.clone())?;
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { grpc.initialize().await })
        }) {
            Ok(_) => {
                info!("Initialized transport: {}", grpc.name());
            }
            Err(e) => {
                warn!("Failed to initialize transport {}: {}", grpc.name(), e);
            }
        }
        transports.push(Arc::new(grpc) as Arc<dyn Transport>);
    }

    if transports.is_empty() {
        return Err(CommunicationError::InvalidConfiguration(
            "No transport mechanisms configured or all transports failed to initialize".to_string(),
        ));
    }

    info!("All transports initialized");

    Ok(transports)
}
