use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to detect init system")]
    InitSystemDetectionFailed,
}
