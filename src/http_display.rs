use std::collections::HashMap;
use thiserror::Error;
use url::Url;

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

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] wreq::Error),
    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),
    #[error("client error: {0}")]
    Client(#[from] crate::client::ClientError),
}

pub async fn retrieve_page(
    url: &Url,
    client: &crate::client::PageClient,
) -> Result<HttpTransaction, HttpError> {
    let start = std::time::Instant::now();

    let response = client.get_raw(url).await?;

    let duration_ms = start.elapsed().as_millis() as u64;

    let status = response.status();
    let resp_headers = response.headers().clone();
    let body = response.text().await?;
    let builder = HttpTransactionBuilder::new("GET", url.as_str())
        .request_headers_from_map(&HashMap::new());

    Ok(builder.finish_with_parts(status, resp_headers, body, duration_ms))
}

pub struct HttpTransactionBuilder {
    method: String,
    url: String,
    request_headers: HashMap<String, String>,
    request_body: Option<String>,
}

impl HttpTransactionBuilder {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            request_headers: HashMap::new(),
            request_body: None,
        }
    }

    pub fn request_headers_from_map(
        mut self,
        headers: &HashMap<String, String>,
    ) -> Self {
        self.request_headers = headers.clone();
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
        duration_ms: u64,
    ) -> HttpTransaction {
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
            duration_ms,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transaction() -> HttpTransaction {
        let mut req_headers = HashMap::new();
        req_headers.insert("host".to_string(), "example.com".to_string());
        let mut resp_headers = HashMap::new();
        resp_headers.insert("content-type".to_string(), "text/html".to_string());

        HttpTransaction {
            request: HttpRequestInfo {
                method: "GET".to_string(),
                url: "https://example.com/".to_string(),
                headers: req_headers,
                body: None,
            },
            response: HttpResponseInfo {
                status: 200,
                headers: resp_headers,
                body: "<html></html>".to_string(),
                body_length: 13,
            },
            duration_ms: 42,
        }
    }

    #[test]
    fn format_for_llm_includes_all_sections() {
        let tx = sample_transaction();
        let out = tx.format_for_llm();

        assert!(out.contains("=== HTTP TRANSACTION ==="));
        assert!(out.contains("REQUEST:"));
        assert!(out.contains("Method: GET"));
        assert!(out.contains("https://example.com/"));
        assert!(out.contains("RESPONSE:"));
        assert!(out.contains("Status: 200"));
        assert!(out.contains("42ms"));
        assert!(out.contains("<html></html>"));
        assert!(out.contains("========================"));
    }

    #[test]
    fn format_for_llm_shows_header_count() {
        let tx = sample_transaction();
        let out = tx.format_for_llm();

        assert!(out.contains("1 headers)"));
    }

    #[test]
    fn format_for_llm_empty_body() {
        let tx = HttpTransaction {
            request: HttpRequestInfo {
                method: "GET".to_string(),
                url: "https://example.com/".to_string(),
                headers: HashMap::new(),
                body: None,
            },
            response: HttpResponseInfo {
                status: 200,
                headers: HashMap::new(),
                body: String::new(),
                body_length: 0,
            },
            duration_ms: 0,
        };
        let out = tx.format_for_llm();

        assert!(out.contains("(empty)"));
        assert!(out.contains("(no headers)"));
    }

    #[test]
    fn builder_finish_constructs_transaction() {
        let mut hdrs = wreq::header::HeaderMap::new();
        hdrs.insert("content-type", "text/plain".parse().unwrap());

        let tx = HttpTransactionBuilder::new("GET", "https://example.com/")
            .request_headers_from_map(&HashMap::new())
            .finish_with_parts(wreq::StatusCode::OK, hdrs, "hello".to_string(), 10);

        assert_eq!(tx.request.method, "GET");
        assert_eq!(tx.response.status, 200);
        assert_eq!(tx.response.body, "hello");
        assert_eq!(tx.response.body_length, 5);
        assert_eq!(tx.duration_ms, 10);
        assert_eq!(
            tx.response.headers.get("content-type").unwrap(),
            "text/plain"
        );
    }

    #[test]
    fn builder_with_request_body() {
        let tx = HttpTransactionBuilder::new("POST", "https://example.com/")
            .request_body(Some("data".to_string()))
            .finish_with_parts(
                wreq::StatusCode::OK,
                wreq::header::HeaderMap::new(),
                String::new(),
                0,
            );

        assert_eq!(tx.request.body.as_deref(), Some("data"));
    }

    #[test]
    fn format_headers_empty() {
        let tx = sample_transaction();
        let empty = HashMap::new();
        let out = tx.format_headers(&empty);
        assert_eq!(out, "    (no headers)");
    }

    #[test]
    fn format_headers_present() {
        let tx = sample_transaction();
        let mut h = HashMap::new();
        h.insert("x-test".to_string(), "value".to_string());
        let out = tx.format_headers(&h);
        assert!(out.contains("x-test: value"));
    }
}
