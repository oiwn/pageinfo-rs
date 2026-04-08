use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("cache version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: String },
}
