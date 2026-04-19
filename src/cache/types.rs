use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const CACHE_DIR: &str = ".pginf";

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub root_dir: PathBuf,
    pub enabled: bool,
    pub refresh: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from(CACHE_DIR),
            enabled: true,
            refresh: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFetch {
    pub input_url: String,
    pub final_url: String,
    pub normalized_final_url: String,
    pub status: u16,
    pub fetched_at: String,
}

#[derive(Debug, Clone)]
pub struct CachedPage {
    pub fetch: CachedFetch,
    pub headers: HashMap<String, String>,
    pub html: String,
}
