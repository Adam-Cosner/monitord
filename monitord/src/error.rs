#[derive(Clone, Debug)]
pub enum Error {
    // todo: add error variants
}

pub type Result<T> = std::result::Result<T, Error>;
