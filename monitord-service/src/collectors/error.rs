use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("generic error: {0}")]
    Generic(String),
    #[error("collector disabled")]
    Disabled,
    #[error("channel error: {0}")]
    Channel(String),
    #[error("process error: {0}")]
    Process(String),
    #[error("read error: {0}")]
    Read(String),
}
