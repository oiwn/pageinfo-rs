use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cache::error::CacheError;
use crate::cache::key::CacheKey;
use crate::cache::types::{CacheConfig, CachedFetch, CachedPage};

const CACHE_VERSION: u32 = 1;

pub trait Cache {
    fn init(&self) -> Result<(), CacheError>;
    fn key_for_final_url(&self, final_url: &str) -> Result<CacheKey, CacheError>;
    fn load(&self, key: &CacheKey) -> Result<Option<CachedPage>, CacheError>;
    fn store(&self, page: CachedPage) -> Result<CacheKey, CacheError>;
    #[allow(dead_code)]
    fn delete(&self, key: &CacheKey) -> Result<(), CacheError>;
}

#[derive(Debug, Clone)]
pub struct FileCache {
    config: CacheConfig,
}

impl FileCache {
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn should_refresh(&self) -> bool {
        self.config.refresh
    }

    fn version_path(&self) -> PathBuf {
        self.config.root_dir.join("VERSION")
    }

    fn pages_dir(&self) -> PathBuf {
        self.config.root_dir.join("pages")
    }

    fn entry_dir(&self, key: &CacheKey) -> PathBuf {
        self.pages_dir().join(&key.hash)
    }

    fn fetch_path(&self, key: &CacheKey) -> PathBuf {
        self.entry_dir(key).join("fetch.json")
    }

    fn headers_path(&self, key: &CacheKey) -> PathBuf {
        self.entry_dir(key).join("headers.json")
    }

    fn html_path(&self, key: &CacheKey) -> PathBuf {
        self.entry_dir(key).join("page.html")
    }

    fn read_version(&self) -> Result<Option<String>, CacheError> {
        let path = self.version_path();
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(fs::read_to_string(path)?.trim().to_string()))
    }

    fn write_json<T: serde::Serialize>(
        &self,
        path: &Path,
        value: &T,
    ) -> Result<(), CacheError> {
        let bytes = serde_json::to_vec_pretty(value)?;
        fs::write(path, bytes)?;
        Ok(())
    }
}

impl Cache for FileCache {
    fn init(&self) -> Result<(), CacheError> {
        if !self.is_enabled() {
            return Ok(());
        }

        fs::create_dir_all(self.pages_dir())?;

        match self.read_version()? {
            Some(found) if found != CACHE_VERSION.to_string() => {
                Err(CacheError::VersionMismatch {
                    expected: CACHE_VERSION,
                    found,
                })
            }
            Some(_) => Ok(()),
            None => {
                fs::write(self.version_path(), CACHE_VERSION.to_string())?;
                Ok(())
            }
        }
    }

    fn key_for_final_url(&self, final_url: &str) -> Result<CacheKey, CacheError> {
        CacheKey::new(final_url)
    }

    fn load(&self, key: &CacheKey) -> Result<Option<CachedPage>, CacheError> {
        if !self.is_enabled() {
            return Ok(None);
        }

        let entry_dir = self.entry_dir(key);
        if !entry_dir.exists() {
            return Ok(None);
        }

        let fetch_path = self.fetch_path(key);
        let headers_path = self.headers_path(key);
        let html_path = self.html_path(key);

        if !fetch_path.exists() || !headers_path.exists() || !html_path.exists() {
            return Ok(None);
        }

        let fetch: CachedFetch = serde_json::from_slice(&fs::read(fetch_path)?)?;
        let headers: HashMap<String, String> =
            serde_json::from_slice(&fs::read(headers_path)?)?;
        let html = fs::read_to_string(html_path)?;

        Ok(Some(CachedPage {
            fetch,
            headers,
            html,
        }))
    }

    fn store(&self, page: CachedPage) -> Result<CacheKey, CacheError> {
        let key = self.key_for_final_url(&page.fetch.final_url)?;
        let entry_dir = self.entry_dir(&key);
        fs::create_dir_all(&entry_dir)?;
        self.write_json(&self.fetch_path(&key), &page.fetch)?;
        self.write_json(&self.headers_path(&key), &page.headers)?;
        fs::write(self.html_path(&key), &page.html)?;
        Ok(key)
    }

    fn delete(&self, key: &CacheKey) -> Result<(), CacheError> {
        let entry_dir = self.entry_dir(key);
        if entry_dir.exists() {
            fs::remove_dir_all(entry_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("pageinfo-cache-test-{nanos}"))
    }

    fn make_cached_page(
        input_url: &str,
        final_url: &str,
        status: u16,
        headers: HashMap<String, String>,
        html: &str,
    ) -> CachedPage {
        let key = CacheKey::new(final_url).unwrap();
        CachedPage {
            fetch: CachedFetch {
                input_url: input_url.to_string(),
                final_url: final_url.to_string(),
                normalized_final_url: key.normalized_final_url,
                status,
                fetched_at: "0".to_string(),
            },
            headers,
            html: html.to_string(),
        }
    }

    #[test]
    fn round_trip_store_and_load() {
        let root_dir = temp_root();
        let cache = FileCache::new(CacheConfig {
            root_dir: root_dir.clone(),
            enabled: true,
            refresh: false,
        });

        cache.init().unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());

        let page = make_cached_page(
            "https://example.com",
            "https://example.com/news",
            200,
            headers,
            "<html></html>",
        );

        let key = cache.store(page).unwrap();
        let loaded = cache.load(&key).unwrap().unwrap();

        assert_eq!(loaded.fetch.final_url, "https://example.com/news");
        assert_eq!(loaded.fetch.status, 200);
        assert_eq!(loaded.html, "<html></html>");

        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn delete_removes_cached_entry() {
        let root_dir = temp_root();
        let cache = FileCache::new(CacheConfig {
            root_dir: root_dir.clone(),
            enabled: true,
            refresh: false,
        });

        cache.init().unwrap();

        let page = make_cached_page(
            "https://example.com",
            "https://example.com/news",
            200,
            HashMap::new(),
            "<html></html>",
        );

        let key = cache.store(page).unwrap();
        assert!(cache.load(&key).unwrap().is_some());

        cache.delete(&key).unwrap();
        assert!(cache.load(&key).unwrap().is_none());

        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn init_fails_on_version_mismatch() {
        let root_dir = temp_root();
        fs::create_dir_all(root_dir.join("pages")).unwrap();
        fs::write(root_dir.join("VERSION"), "999").unwrap();

        let cache = FileCache::new(CacheConfig {
            root_dir: root_dir.clone(),
            enabled: true,
            refresh: false,
        });

        let result = cache.init();
        assert!(matches!(
            result,
            Err(CacheError::VersionMismatch {
                expected: 1,
                found
            }) if found == "999"
        ));

        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn disabled_cache_skips_init_and_load() {
        let root_dir = temp_root();
        let cache = FileCache::new(CacheConfig {
            root_dir: root_dir.clone(),
            enabled: false,
            refresh: false,
        });

        cache.init().unwrap();
        assert!(!root_dir.exists());

        let key = cache.key_for_final_url("https://example.com/news").unwrap();
        assert!(cache.load(&key).unwrap().is_none());
    }
}
