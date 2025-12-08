use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    // todo: add error variants
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("NVIDIA error: {0}")]
    Nvidia(#[from] nvml_wrapper::error::NvmlError),

    #[error("Not implemented")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, Error>;
