use pageinfo_rs::httpreq::{load_config, execute_request, HttpReqError};
use std::path::Path;

#[tokio::test]
async fn test_load_config_get_request() {
    let config_path = Path::new("tests/data/test_request.toml");
    let config = load_config(config_path).await.unwrap();
    
    assert_eq!(config.request.url, "https://httpbin.org/get");
    assert_eq!(config.request.method, "GET");
    assert_eq!(config.headers.len(), 2);
    assert_eq!(config.headers.get("Accept"), Some(&"application/json".to_string()));
    assert_eq!(config.headers.get("Custom-Header"), Some(&"test-value".to_string()));
    assert!(config.body.is_none());
}

#[tokio::test]
async fn test_load_config_post_form() {
    let config_path = Path::new("tests/data/test_post.toml");
    let config = load_config(config_path).await.unwrap();
    
    assert_eq!(config.request.url, "https://httpbin.org/post");
    assert_eq!(config.request.method, "POST");
    assert_eq!(config.headers.len(), 1);
    assert_eq!(config.headers.get("Content-Type"), Some(&"application/json".to_string()));
    assert!(config.body.is_some());
}

#[tokio::test]
async fn test_load_config_post_json() {
    let config_path = Path::new("tests/data/test_json.toml");
    let config = load_config(config_path).await.unwrap();
    
    assert_eq!(config.request.url, "https://httpbin.org/post");
    assert_eq!(config.request.method, "POST");
    assert!(config.body.is_some());
}

#[tokio::test]
async fn test_load_config_nonexistent_file() {
    let config_path = Path::new("tests/data/nonexistent.toml");
    let result = load_config(config_path).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), HttpReqError::FileRead(_)));
}

#[tokio::test]
async fn test_execute_get_request() {
    let config_path = Path::new("tests/data/test_request.toml");
    let config = load_config(config_path).await.unwrap();
    let transaction = execute_request(config).await.unwrap();
    
    assert_eq!(transaction.request.method, "GET");
    assert_eq!(transaction.request.url, "https://httpbin.org/get");
    assert_eq!(transaction.response.status, 200);
    assert!(transaction.response.body.contains("httpbin.org"));
    assert!(transaction.duration_ms > 0);
}

#[tokio::test]
async fn test_execute_post_json_request() {
    let config_path = Path::new("tests/data/test_json.toml");
    let config = load_config(config_path).await.unwrap();
    let transaction = execute_request(config).await.unwrap();
    
    assert_eq!(transaction.request.method, "POST");
    assert_eq!(transaction.request.url, "https://httpbin.org/post");
    assert_eq!(transaction.response.status, 200);
    assert!(transaction.response.body.contains("Hello from pageinfo-rs!"));
    assert!(transaction.response.body.contains("\"json\": {"));
}

#[tokio::test]
async fn test_execute_post_form_request() {
    let config_path = Path::new("tests/data/test_post.toml");
    let config = load_config(config_path).await.unwrap();
    let transaction = execute_request(config).await.unwrap();
    
    assert_eq!(transaction.request.method, "POST");
    assert_eq!(transaction.response.status, 200);
    assert!(transaction.response.body.contains("key1=value1"));
    assert!(transaction.response.body.contains("key2=value2"));
}

#[tokio::test]
async fn test_invalid_url() {
    let config_path = Path::new("tests/data/test_request.toml");
    let mut config = load_config(config_path).await.unwrap();
    config.request.url = "invalid-url".to_string();
    
    let result = execute_request(config).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), HttpReqError::Url(_)));
}