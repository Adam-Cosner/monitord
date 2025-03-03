use thiserror::Error;

use crate::error::PlatformError;

#[derive(Error, Debug)]
pub enum ServiceError {
    // todo
    #[error("platform error: {0}")]
    PlatformError(PlatformError),
}
