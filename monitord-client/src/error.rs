//! Error types for the monitord client library

use thiserror::Error;

/// Errors that can occur when using the monitord client
#[derive(Error, Debug)]
pub enum ClientError {
    /// Failed to connect to the monitord service
    #[error("Failed to connect to monitord service: {0}")]
    ConnectionError(String),

    /// Error related to subscription operations
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Error communicating with the service
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Service responded with an error
    #[error("Service error: {0}")]
    ServiceError(String),

    /// Error with the transport layer
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Timeout occurred while waiting for a response
    #[error("Timeout occurred while waiting for response")]
    Timeout,

    /// The operation is not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}
