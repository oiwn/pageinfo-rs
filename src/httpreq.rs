use crate::http::{HttpTransaction, HttpTransactionBuilder, HttpError};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct HttpRequestConfig {
    pub request: RequestSection,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<BodySection>,
}

#[derive(Debug, Deserialize)]
pub struct RequestSection {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BodySection {
    Text { text: String },
    Form(HashMap<String, String>),
    Plain(String),
}

fn default_method() -> String {
    "GET".to_string()
}

#[derive(Debug, Error)]
pub enum HttpReqError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),
    #[error("Failed to parse TOML config: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("HTTP request failed: {0}")]
    Http(#[from] HttpError),
    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),
    #[error("Unsupported HTTP method: {0}")]
    UnsupportedMethod(String),
}

pub async fn load_config(path: &Path) -> Result<HttpRequestConfig, HttpReqError> {
    let content = tokio::fs::read_to_string(path).await?;
    let config: HttpRequestConfig = toml::from_str(&content)?;
    Ok(config)
}

pub async fn execute_request(config: HttpRequestConfig) -> Result<HttpTransaction, HttpReqError> {
    let url = Url::parse(&config.request.url)?;
    
    // Build wreq client with default user agent
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.63 Safari/537.36";
    let client_builder = wreq::Client::builder().user_agent(user_agent);
    
    let client = client_builder.build().map_err(HttpError::Request)?;
    
    // Start building the transaction
    let mut builder = HttpTransactionBuilder::new(&config.request.method, url.as_str());
    
    // Build request based on method
    let mut request_builder = match config.request.method.to_uppercase().as_str() {
        "GET" => client.get(url.clone()),
        "POST" => client.post(url.clone()),
        "PUT" => client.put(url.clone()),
        "DELETE" => client.delete(url.clone()),
        "PATCH" => client.patch(url.clone()),
        "HEAD" => client.head(url.clone()),
        method => {
            return Err(HttpReqError::UnsupportedMethod(method.to_string()));
        }
    };
    
    // Add headers
    for (key, value) in &config.headers {
        request_builder = request_builder.header(key, value);
    }
    
    // Add body if present
    if let Some(body) = &config.body {
        request_builder = match body {
            BodySection::Text { text } => request_builder.body(text.clone()),
            BodySection::Form(form_data) => request_builder.form(form_data),
            BodySection::Plain(text) => request_builder.body(text.clone()),
        };
    }
    
    // Build and execute request
    let request = request_builder.build().map_err(HttpError::Request)?;
    builder = builder.request_headers(request.headers());
    
    let response = client.execute(request).await.map_err(HttpError::Request)?;
    
    // Extract response data
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.map_err(HttpError::Request)?;
    
    // Finish transaction
    let transaction = builder.finish_with_parts(status, headers, body);
    Ok(transaction)
}

pub async fn run_from_file(config_path: &Path) -> Result<(), HttpReqError> {
    let config = load_config(config_path).await?;
    let transaction = execute_request(config).await?;
    
    println!("{}", transaction.format_for_llm());
    Ok(())
}