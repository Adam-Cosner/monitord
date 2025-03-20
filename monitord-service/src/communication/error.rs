use thiserror::Error;
use monitord_transport::error::TransportError;

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("transport layer error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("serialization error: {0}")]
    Serialization(String),
    
    #[error("task join error: {0}")]
    TaskJoin(String),
    
    #[error("channel closed")]
    ChannelClosed,
    
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl From<String> for CommunicationError {
    fn from(s: String) -> Self {
        Self::Unknown(s)
    }
}