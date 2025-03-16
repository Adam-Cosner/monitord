use thiserror::Error;
pub use crate::communication::error::CommunicationError;

#[derive(Error, Debug)]
pub enum ClientError {
    // todo
    #[error("Communication Error: {0}")]
    Communication(#[from] CommunicationError)
}