//! Error types for communication module

use thiserror::Error;
use tokio::sync::broadcast::error::SendError;
use crate::communication::subscription::error::SubscriptionError;
use crate::collectors::error::CollectionError;

/// Error types for the communication module
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

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Task error: {0}")]
    Task(String),

    #[error("Registry error: {0}")]
    Registry(String),
}

impl<T> From<SendError<T>> for CommunicationError {
    fn from(err: SendError<T>) -> Self {
        CommunicationError::Send(err.to_string())
    }
}

impl From<tokio::task::JoinError> for CommunicationError {
    fn from(err: tokio::task::JoinError) -> Self {
        CommunicationError::Task(err.to_string())
    }
}