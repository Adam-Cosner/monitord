use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("server startup error: {0}")]
    ServerStartup(String),

    #[error("task join error: {0}")]
    TaskJoin(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

impl From<String> for CommunicationError {
    fn from(s: String) -> Self {
        Self::Unknown(s)
    }
}
