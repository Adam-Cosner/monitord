use thiserror::Error;

use crate::error::{CollectionError, CommunicationError, PlatformError};

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("platform error: {0}")]
    Platform(PlatformError),
    #[error("communication error: {0}")]
    Communication(CommunicationError),
    #[error("collection error: {0}")]
    Collection(CollectionError),
}
