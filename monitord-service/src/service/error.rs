use crate::error::{CommunicationError, PlatformError};
use monitord_collectors::error::CollectorError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("communication error: {0}")]
    Communication(#[from] CommunicationError),

    #[error("collection error: {0}")]
    Collection(#[from] CollectorError),

    #[error("task error: {0}")]
    TaskError(String),

    #[error("configuration error: {0}")]
    ConfigError(String),
}

impl From<config::ConfigError> for ServiceError {
    fn from(e: config::ConfigError) -> Self {
        Self::ConfigError(e.to_string())
    }
}
