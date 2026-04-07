use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleOptions {
    pub max_pages: usize,
    pub concurrency: usize,
}

impl Default for SampleOptions {
    fn default() -> Self {
        Self {
            max_pages: 5,
            concurrency: 3,
        }
    }
}
