use crate::collectors::error::CollectionError;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

pub use super::subscription::error::*;

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("Failed to initialize communication layer: {0}")]
    Init(String),

    #[error("Failed to send message: {0}")]
    Send(String),

    #[error("Failed to receive message: {0}")]
    Receive(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid subscription: {0}")]
    InvalidSubscription(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Subscription error: {0}")]
    Subscription(#[from] SubscriptionError),

    #[error("Collector error: {0}")]
    Collector(#[from] CollectionError),

    #[error("GRPC error: {0}")]
    Grpc(String),

    #[error("Iceoryx error: {0}")]
    Iceoryx(String),
}

impl<T> From<SendError<T>> for CommunicationError {
    fn from(err: SendError<T>) -> Self {
        CommunicationError::Send(err.to_string())
    }
}
