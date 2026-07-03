#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("commitment mismatch")]
    CommitmentMismatch,

    #[error("decode error: {0}")]
    Decode(String),
}

pub type Result<T> = std::result::Result<T, Error>;
