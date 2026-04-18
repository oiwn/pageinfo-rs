mod error;
mod key;
mod store;
mod types;

pub use error::CacheError;
pub use store::{Cache, FileCache};
#[cfg(test)]
pub use types::CachedFetch;
pub use types::{CacheConfig, CachedPage};
