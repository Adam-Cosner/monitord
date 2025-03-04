use crate::collectors::error::CollectionError;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

pub use super::subscription::error::*;

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("Failed to initialize communication layer: {0}")]
    InitError(String),

    #[error("Failed to send message: {0}")]
    SendError(String),

    #[error("Failed to receive message: {0}")]
    ReceiveError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid subscription: {0}")]
    InvalidSubscription(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Subscription error: {0}")]
    SubscriptionError(#[from] SubscriptionError),

    #[error("Collector error: {0}")]
    CollectorError(#[from] CollectionError),

    #[error("GRPC error: {0}")]
    GrpcError(String),

    #[error("Iceoryx error: {0}")]
    IceoryxError(String),
}

impl<T> From<SendError<T>> for CommunicationError {
    fn from(err: SendError<T>) -> Self {
        CommunicationError::SendError(err.to_string())
    }
}
