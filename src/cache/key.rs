use sha2::{Digest, Sha256};
use url::Url;

use crate::cache::CacheError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    pub normalized_final_url: String,
    pub hash: String,
}

impl CacheKey {
    pub fn new(url: &str) -> Result<Self, CacheError> {
        let normalized_final_url = normalize_url(url)?;
        let hash = hash_url(&normalized_final_url);
        Ok(Self {
            normalized_final_url,
            hash,
        })
    }
}

pub fn normalize_url(url: &str) -> Result<String, CacheError> {
    let mut parsed =
        Url::parse(url).map_err(|e| CacheError::InvalidUrl(e.to_string()))?;

    parsed.set_fragment(None);

    if matches!(
        (parsed.scheme(), parsed.port()),
        ("http", Some(80)) | ("https", Some(443))
    ) {
        let _ = parsed.set_port(None);
    }

    Ok(parsed.to_string())
}

fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let bytes = hasher.finalize();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_removes_fragment_and_default_port() {
        let normalized =
            normalize_url("https://Example.com:443/path?q=1#frag").unwrap();
        assert_eq!(normalized, "https://example.com/path?q=1");
    }
}
