use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("serialization error: {0}")]
    Serialize(String),

    #[error("initialize error: {0}")]
    Initialize(String),

    #[error("publish error: {0}")]
    Publish(String),

    #[error("receive error: {0}")]
    Receive(String),
}

