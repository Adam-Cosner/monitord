use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectorError {
    #[error("Failed to initialize collector: {0}")]
    InitializationError(String),
    
    #[error("Failed to collect data: {0}")]
    CollectionError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigurationError(String),
    
    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("GPU error: {0}")]
    GpuError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Process error: {0}")]
    ProcessError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, CollectorError>;