use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("generic error: {0}")]
    Generic(String),
    #[error("collector disabled")]
    Disabled,
    #[error("process error: {0}")]
    Process(String),
}
