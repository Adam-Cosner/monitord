use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} Collector Error: {1}")]
    Collector(String, String),
}
