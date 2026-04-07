use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("fetch failed for {url}: HTTP {status}")]
    Fetch { url: String, status: u16 },

    #[error("parse error for {url}: {reason}")]
    Parse { url: String, reason: String },

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
