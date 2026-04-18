use std::collections::HashMap;

use thiserror::Error;
use url::Url;

use crate::cache::CachedPage;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("fetch failed for {url}: HTTP {status}")]
    Fetch { url: String, status: u16 },
    #[error("request error for {url}: {reason}")]
    Request { url: String, reason: String },
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("invalid proxy URL: {0}")]
    InvalidProxy(String),
    #[error("unknown browser name: {0}")]
    UnknownBrowser(String),
    #[error("all {attempts} attempts failed for {url}")]
    AllAttemptsFailed { url: String, attempts: usize },
}

#[derive(Debug, Clone)]
pub struct PageClient {
    proxy_url: Option<String>,
    browser: Option<wreq_util::Emulation>,
    fallback_browsers: Vec<wreq_util::Emulation>,
    max_retries: usize,
}

impl Default for PageClient {
    fn default() -> Self {
        Self {
            proxy_url: None,
            browser: None,
            fallback_browsers: vec![
                wreq_util::Emulation::Chrome136,
                wreq_util::Emulation::Firefox139,
                wreq_util::Emulation::Safari18_5,
            ],
            max_retries: 3,
        }
    }
}

impl PageClient {
    pub fn builder() -> PageClientBuilder {
        PageClientBuilder::new()
    }

    pub async fn fetch(&self, url: &str) -> Result<CachedPage, ClientError> {
        let parsed =
            Url::parse(url).map_err(|e| ClientError::InvalidUrl(e.to_string()))?;

        let mut attempts = 0;
        let mut last_err = None;
        let mut browsers_to_try: Vec<Option<wreq_util::Emulation>> =
            vec![self.browser];
        for fb in &self.fallback_browsers {
            browsers_to_try.push(Some(*fb));
        }

        for browser_opt in browsers_to_try {
            if attempts >= self.max_retries {
                break;
            }
            attempts += 1;

            let client = self.build_wreq_client(browser_opt)?;
            match self.do_fetch(&client, &parsed).await {
                Ok(page) => return Ok(page),
                Err(e) if is_retryable(&e) => {
                    last_err = Some(e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        if attempts > 0 {
            Err(last_err.unwrap_or(ClientError::AllAttemptsFailed {
                url: url.to_string(),
                attempts,
            }))
        } else {
            Err(ClientError::AllAttemptsFailed {
                url: url.to_string(),
                attempts: 0,
            })
        }
    }

    pub async fn get_raw(&self, url: &Url) -> Result<wreq::Response, ClientError> {
        let client = self.build_wreq_client(self.browser)?;
        client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.to_string(),
                reason: e.to_string(),
            })
    }

    fn build_wreq_client(
        &self,
        browser: Option<wreq_util::Emulation>,
    ) -> Result<wreq::Client, ClientError> {
        let mut builder = wreq::ClientBuilder::new();

        if let Some(emulation) = browser {
            let opt = wreq_util::EmulationOption::builder()
                .emulation(emulation)
                .build();
            builder = builder.emulation(opt);
        }

        if let Some(ref proxy_str) = self.proxy_url {
            let proxy = wreq::Proxy::all(proxy_str).map_err(|e| {
                ClientError::InvalidProxy(format!("{proxy_str}: {e}"))
            })?;
            builder = builder.proxy(proxy);
        }

        builder.build().map_err(|e| ClientError::Request {
            url: String::new(),
            reason: e.to_string(),
        })
    }

    async fn do_fetch(
        &self,
        client: &wreq::Client,
        url: &Url,
    ) -> Result<CachedPage, ClientError> {
        let response = client.get(url.clone()).send().await.map_err(|e| {
            ClientError::Request {
                url: url.to_string(),
                reason: e.to_string(),
            }
        })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            return Err(ClientError::Fetch {
                url: url.to_string(),
                status,
            });
        }

        let final_url = response.url().to_string();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (k.to_string(), v.to_str().unwrap_or("<invalid>").to_string())
            })
            .collect();

        let body = response.text().await.map_err(|e| ClientError::Request {
            url: url.to_string(),
            reason: e.to_string(),
        })?;

        let cache =
            crate::cache::FileCache::new(crate::cache::CacheConfig::default());
        cache
            .cache_page(url.as_str(), &final_url, status, headers, body)
            .map_err(|e| ClientError::Request {
                url: url.to_string(),
                reason: e.to_string(),
            })
    }
}

fn is_retryable(err: &ClientError) -> bool {
    match err {
        ClientError::Fetch { status, .. } => {
            matches!(status, 403 | 429 | 503)
        }
        ClientError::Request { .. } => true,
        _ => false,
    }
}

pub fn parse_browser(name: &str) -> Result<wreq_util::Emulation, ClientError> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "chrome" | "chrome137" => Ok(wreq_util::Emulation::Chrome137),
        "chrome136" => Ok(wreq_util::Emulation::Chrome136),
        "chrome135" => Ok(wreq_util::Emulation::Chrome135),
        "chrome134" => Ok(wreq_util::Emulation::Chrome134),
        "chrome133" => Ok(wreq_util::Emulation::Chrome133),
        "chrome132" => Ok(wreq_util::Emulation::Chrome132),
        "chrome131" => Ok(wreq_util::Emulation::Chrome131),
        "chrome130" => Ok(wreq_util::Emulation::Chrome130),
        "chrome129" => Ok(wreq_util::Emulation::Chrome129),
        "chrome128" => Ok(wreq_util::Emulation::Chrome128),
        "chrome127" => Ok(wreq_util::Emulation::Chrome127),
        "chrome126" => Ok(wreq_util::Emulation::Chrome126),
        "chrome124" => Ok(wreq_util::Emulation::Chrome124),
        "chrome123" => Ok(wreq_util::Emulation::Chrome123),
        "chrome120" => Ok(wreq_util::Emulation::Chrome120),
        "chrome119" => Ok(wreq_util::Emulation::Chrome119),
        "chrome118" => Ok(wreq_util::Emulation::Chrome118),
        "chrome117" => Ok(wreq_util::Emulation::Chrome117),
        "chrome116" => Ok(wreq_util::Emulation::Chrome116),
        "chrome114" => Ok(wreq_util::Emulation::Chrome114),
        "chrome110" => Ok(wreq_util::Emulation::Chrome110),
        "chrome109" => Ok(wreq_util::Emulation::Chrome109),
        "chrome108" => Ok(wreq_util::Emulation::Chrome108),
        "chrome107" => Ok(wreq_util::Emulation::Chrome107),
        "chrome106" => Ok(wreq_util::Emulation::Chrome106),
        "chrome105" => Ok(wreq_util::Emulation::Chrome105),
        "chrome104" => Ok(wreq_util::Emulation::Chrome104),
        "chrome101" => Ok(wreq_util::Emulation::Chrome101),
        "chrome100" => Ok(wreq_util::Emulation::Chrome100),
        "firefox" => Ok(wreq_util::Emulation::Firefox139),
        "safari" => Ok(wreq_util::Emulation::Safari18_5),
        "edge" => Ok(wreq_util::Emulation::Edge134),
        "okhttp" => Ok(wreq_util::Emulation::OkHttp5),
        _ => Err(ClientError::UnknownBrowser(name.to_string())),
    }
}

#[derive(Debug, Clone)]
pub struct PageClientBuilder {
    proxy_url: Option<String>,
    browser: Option<wreq_util::Emulation>,
    fallback_browsers: Vec<wreq_util::Emulation>,
    max_retries: usize,
}

impl PageClientBuilder {
    pub fn new() -> Self {
        Self {
            proxy_url: None,
            browser: None,
            fallback_browsers: vec![
                wreq_util::Emulation::Chrome136,
                wreq_util::Emulation::Firefox139,
                wreq_util::Emulation::Safari18_5,
            ],
            max_retries: 3,
        }
    }

    pub fn proxy(mut self, url: &str) -> Result<Self, ClientError> {
        wreq::Proxy::all(url)
            .map_err(|e| ClientError::InvalidProxy(format!("{url}: {e}")))?;
        self.proxy_url = Some(url.to_string());
        Ok(self)
    }

    pub fn proxy_from_env(mut self) -> Self {
        for var in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
            if let Ok(val) = std::env::var(var) {
                if !val.is_empty() {
                    self.proxy_url = Some(val);
                    return self;
                }
            }
        }
        self
    }

    pub fn browser(mut self, emulation: wreq_util::Emulation) -> Self {
        self.browser = Some(emulation);
        self
    }

    #[allow(dead_code)]
    pub fn fallback_browsers(
        mut self,
        browsers: Vec<wreq_util::Emulation>,
    ) -> Self {
        self.fallback_browsers = browsers;
        self
    }

    #[allow(dead_code)]
    pub fn max_retries(mut self, n: usize) -> Self {
        self.max_retries = n;
        self
    }

    pub fn build(self) -> PageClient {
        PageClient {
            proxy_url: self.proxy_url,
            browser: self.browser,
            fallback_browsers: self.fallback_browsers,
            max_retries: self.max_retries,
        }
    }
}

impl Default for PageClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults() {
        let client = PageClient::builder().build();
        assert!(client.proxy_url.is_none());
        assert!(client.browser.is_none());
        assert_eq!(client.fallback_browsers.len(), 3);
        assert_eq!(client.max_retries, 3);
    }

    #[test]
    fn proxy_rejects_invalid_url() {
        let result = PageClient::builder().proxy("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn proxy_accepts_valid_url() {
        let result = PageClient::builder().proxy("http://127.0.0.1:8080");
        assert!(result.is_ok());
        let client = result.unwrap().build();
        assert_eq!(client.proxy_url.unwrap(), "http://127.0.0.1:8080");
    }

    #[test]
    fn parse_browser_known_names() {
        assert!(parse_browser("chrome131").is_ok());
        assert!(parse_browser("firefox").is_ok());
        assert!(parse_browser("safari").is_ok());
        assert!(parse_browser("edge").is_ok());
        assert!(parse_browser("okhttp").is_ok());
    }

    #[test]
    fn parse_browser_case_insensitive() {
        assert!(parse_browser("Chrome131").is_ok());
        assert!(parse_browser("FIREFOX").is_ok());
    }

    #[test]
    fn parse_browser_unknown_name() {
        assert!(matches!(
            parse_browser("netscape"),
            Err(ClientError::UnknownBrowser(_))
        ));
    }

    #[test]
    fn builder_with_browser() {
        let client = PageClient::builder()
            .browser(wreq_util::Emulation::Chrome131)
            .build();
        assert!(client.browser.is_some());
    }

    #[test]
    fn builder_custom_fallbacks() {
        let client = PageClient::builder()
            .fallback_browsers(vec![wreq_util::Emulation::Firefox139])
            .build();
        assert_eq!(client.fallback_browsers.len(), 1);
    }

    #[test]
    fn builder_custom_max_retries() {
        let client = PageClient::builder().max_retries(5).build();
        assert_eq!(client.max_retries, 5);
    }

    #[test]
    fn is_retryable_on_403() {
        let err = ClientError::Fetch {
            url: "http://x".into(),
            status: 403,
        };
        assert!(is_retryable(&err));
    }

    #[test]
    fn is_retryable_on_429() {
        let err = ClientError::Fetch {
            url: "http://x".into(),
            status: 429,
        };
        assert!(is_retryable(&err));
    }

    #[test]
    fn is_not_retryable_on_404() {
        let err = ClientError::Fetch {
            url: "http://x".into(),
            status: 404,
        };
        assert!(!is_retryable(&err));
    }
}
