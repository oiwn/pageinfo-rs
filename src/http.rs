use std::collections::HashMap;
use thiserror::Error;
use url::Url;
// use wreq::Response;
// use wreq::header::{HeaderName, HeaderValue};

#[derive(Debug, Clone)]
pub struct HttpRequestInfo {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HttpResponseInfo {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub body_length: usize,
}

#[derive(Debug, Clone)]
pub struct HttpTransaction {
    pub request: HttpRequestInfo,
    pub response: HttpResponseInfo,
    pub duration_ms: u64,
}

impl HttpTransaction {
    pub fn format_for_llm(&self) -> String {
        format!(
            r#"=== HTTP TRANSACTION ===
REQUEST:
  Method: {}
  URL: {}
  Headers: ({} headers)
{}
  Body: {}

RESPONSE:
  Status: {}
  Headers: ({} headers)
{}
  Body Length: {} bytes
  Duration: {}ms

RESPONSE BODY:
{}
========================"#,
            self.request.method,
            self.request.url,
            self.request.headers.len(),
            self.format_headers(&self.request.headers),
            self.request.body.as_deref().unwrap_or("(empty)"),
            self.response.status,
            self.response.headers.len(),
            self.format_headers(&self.response.headers),
            self.response.body_length,
            self.duration_ms,
            self.response.body
        )
    }

    fn format_headers(&self, headers: &HashMap<String, String>) -> String {
        if headers.is_empty() {
            return "    (no headers)".to_string();
        }

        headers
            .iter()
            .map(|(k, v)| format!("    {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub struct HttpTransactionBuilder {
    method: String,
    url: String,
    request_headers: HashMap<String, String>,
    request_body: Option<String>,
    start_time: std::time::Instant,
}

impl HttpTransactionBuilder {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            request_headers: HashMap::new(),
            request_body: None,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn request_headers(mut self, headers: &wreq::header::HeaderMap) -> Self {
        self.request_headers = headers_to_hashmap(headers);
        self
    }

    #[allow(dead_code)]
    pub fn request_body(mut self, body: Option<String>) -> Self {
        self.request_body = body;
        self
    }

    pub fn finish_with_parts(
        self,
        status: wreq::StatusCode,
        headers: wreq::header::HeaderMap,
        body: String,
    ) -> HttpTransaction {
        let duration = self.start_time.elapsed().as_millis() as u64;

        HttpTransaction {
            request: HttpRequestInfo {
                method: self.method,
                url: self.url,
                headers: self.request_headers,
                body: self.request_body,
            },
            response: HttpResponseInfo {
                status: status.as_u16(),
                headers: headers_to_hashmap(&headers),
                body_length: body.len(),
                body,
            },
            duration_ms: duration,
        }
    }
}

fn headers_to_hashmap(
    headers: &wreq::header::HeaderMap,
) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| {
            (k.to_string(), v.to_str().unwrap_or("<invalid>").to_string())
        })
        .collect()
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] wreq::Error),
    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),
}

pub async fn retrieve_page(url: &Url) -> Result<HttpTransaction, HttpError> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.63 Safari/537.36";

    let client = wreq::Client::builder().user_agent(user_agent).build()?;

    let mut builder = HttpTransactionBuilder::new("GET", url.as_str());

    let request = client.get(url.clone()).build()?;

    builder = builder.request_headers(request.headers());

    let response = client.execute(request).await?;

    // Extract what we need from response before consuming it
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await?;

    Ok(builder.finish_with_parts(status, headers, body))
}
