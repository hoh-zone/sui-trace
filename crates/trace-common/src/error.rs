use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("http error: {0}")]
    Http(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("indexer error: {0}")]
    Indexer(String),

    #[error("security scan error: {0}")]
    Security(String),

    #[error("alert delivery error: {0}")]
    Alert(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("rate limited")]
    RateLimited,

    #[error("internal: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Error::Internal(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Internal(value.to_string())
    }
}
