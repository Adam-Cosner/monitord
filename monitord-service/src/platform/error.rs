use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Failed to detect init system")]
    InitSystemDetectionFailed,
    
    #[error("Unsupported init system: {0}")]
    UnsupportedInitSystem(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Service file generation failed: {0}")]
    ServiceFileGenerationFailed(String),
}
