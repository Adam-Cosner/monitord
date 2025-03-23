use thiserror::Error;
use tonic::{Status, transport};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to connect to monitord service: {0}")]
    ConnectionError(String),
    
    #[error("gRPC error: {0}")]
    GrpcError(#[from] Status),
    
    #[error("Stream closed unexpectedly")]
    StreamClosed,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<transport::Error> for ClientError {
    fn from(error: transport::Error) -> Self {
        ClientError::ConnectionError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ClientError>;