//! Transport implementations for the monitord client

pub mod config;
pub(crate) mod grpc;
pub(crate) mod iceoryx;

use self::config::TransportType;
use crate::Result;

/// Common trait for all transport implementations
#[async_trait::async_trait]
pub(crate) trait TransportTrait: Send + Sync {
    /// Sends a request and receives a response
    async fn send_request(&self, req_type: &str, req_data: Vec<u8>) -> Result<Vec<u8>>;

    /// Closes the transport connection
    async fn close(&mut self) -> Result<()>;

    /// Checks if the transport is connected
    fn is_connected(&self) -> bool;
}

/// Enum of available transport implementations
#[derive(Debug)]
pub(crate) enum TransportLayer {
    Grpc(grpc::GrpcTransport),
    Iceoryx(iceoryx::IceoryxTransport),
}

impl TransportLayer {
    /// Creates a new transport instance
    pub async fn new(transport_type: TransportType, address: &str, port: u16) -> Result<Self> {
        match transport_type {
            TransportType::Grpc => {
                let transport = grpc::GrpcTransport::new(address, port).await?;
                Ok(TransportLayer::Grpc(transport))
            }
            TransportType::Iceoryx => {
                let transport = iceoryx::IceoryxTransport::new(address).await?;
                Ok(TransportLayer::Iceoryx(transport))
            }
        }
    }

    /// Sends a request and receives a response
    pub async fn send_request(&self, req_type: &str, req_data: Vec<u8>) -> Result<Vec<u8>> {
        match self {
            TransportLayer::Grpc(t) => t.send_request(req_type, req_data).await,
            TransportLayer::Iceoryx(t) => t.send_request(req_type, req_data).await,
        }
    }

    /// Closes the transport connection
    pub async fn close(&mut self) -> Result<()> {
        match self {
            TransportLayer::Grpc(t) => t.close().await,
            TransportLayer::Iceoryx(t) => t.close().await,
        }
    }

    /// Checks if the transport is connected
    pub fn is_connected(&self) -> bool {
        match self {
            TransportLayer::Grpc(t) => t.is_connected(),
            TransportLayer::Iceoryx(t) => t.is_connected(),
        }
    }
}
