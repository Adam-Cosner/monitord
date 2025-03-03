use thiserror::Error;

use crate::error::{CollectionError, CommunicationError, PlatformError};

#[derive(Error, Debug)]
pub enum ServiceError {
    // todo
    #[error("platform error: {0}")]
    PlatformError(PlatformError),
    #[error("communication error: {0}")]
    CommunicationError(CommunicationError),
    #[error("collection error: {0}")]
    CollectionError(CollectionError)
}
