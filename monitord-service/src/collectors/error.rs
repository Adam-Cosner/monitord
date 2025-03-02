use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("generic error: {0}")]
    Generic(String),
    #[error("collector disabled")]
    Disabled,
    #[error("channel error: {0}")]
    ChannelError(String),
}
