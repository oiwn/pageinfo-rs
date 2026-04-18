mod error;
mod key;
mod store;
mod types;

pub use error::CacheError;
pub use store::{Cache, FileCache};
pub use types::{CacheConfig, CachedPage};
#[cfg(test)]
pub use types::CachedFetch;
