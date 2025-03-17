use thiserror::Error;
use monitord_transport::error::TransportError;

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("transport layer error: {0}")]
    Transport(#[from] TransportError),
}