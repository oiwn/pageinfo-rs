use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("render engine error: {0}")]
    Engine(String),
    #[error("navigation failed for {url}: {reason}")]
    Navigation { url: String, reason: String },
    #[error("selector not found: {selector}")]
    SelectorNotFound { selector: String },
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub proxy: Option<String>,
    pub selector: Option<String>,
    pub eval: Option<String>,
    pub settle_ms: u64,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            proxy: None,
            selector: None,
            eval: None,
            settle_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RenderedOutput {
    pub url: String,
    pub final_url: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_result: Option<serde_json::Value>,
}

impl RenderedOutput {
    fn render_value(&self) -> serde_json::Value {
        serde_json::json!({
            "url": &self.url,
            "final_url": &self.final_url,
            "content": &self.content,
            "content_length": self.content.len(),
            "eval_result": &self.eval_result,
        })
    }
}

impl crate::output::RenderOutput for RenderedOutput {
    fn render_text(&self) -> String {
        self.content.clone()
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.render_value()).unwrap_or_default()
    }

    fn render_toon(&self) -> String {
        toon_format::encode_default(&self.render_value()).unwrap_or_default()
    }
}

/// Render a page using the obscura headless browser engine.
///
/// Handles JavaScript rendering, async data loading, and stealth mode.
/// Returns fully rendered HTML that a real browser would produce.
#[cfg(feature = "render")]
pub async fn render_page(
    url: &str,
    opts: &RenderOptions,
) -> Result<RenderedOutput, RenderError> {
    let mut builder = obscura::Browser::builder().stealth(true);

    if let Some(ref proxy) = opts.proxy {
        builder = builder.proxy(proxy);
    }

    let browser = builder
        .build()
        .map_err(|e| RenderError::Engine(e.to_string()))?;

    let mut page = browser
        .new_page()
        .await
        .map_err(|e| RenderError::Engine(e.to_string()))?;

    page.goto(url).await.map_err(|e| RenderError::Navigation {
        url: url.to_string(),
        reason: e.to_string(),
    })?;

    page.settle(opts.settle_ms).await;

    let mut content = page.content();
    let final_url = page.url().to_string();

    if let Some(ref selector) = opts.selector {
        let css = dom_content_extraction::scraper::Selector::parse(selector)
            .map_err(|e| {
                RenderError::Engine(format!("invalid selector '{selector}': {e}"))
            })?;
        let document =
            dom_content_extraction::scraper::Html::parse_document(&content);
        let matches: Vec<_> = document.select(&css).collect();
        if matches.is_empty() {
            return Err(RenderError::SelectorNotFound {
                selector: selector.clone(),
            });
        }
        content = matches
            .iter()
            .map(|el| el.html())
            .collect::<Vec<_>>()
            .join("\n");
    }

    let eval_result = opts.eval.as_ref().map(|js| page.evaluate(js));

    Ok(RenderedOutput {
        url: url.to_string(),
        final_url,
        content,
        eval_result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::RenderOutput;

    #[test]
    fn render_options_defaults() {
        let opts = RenderOptions::default();
        assert!(opts.proxy.is_none());
        assert!(opts.selector.is_none());
        assert!(opts.eval.is_none());
        assert_eq!(opts.settle_ms, 2000);
    }

    #[test]
    fn render_error_engine_display() {
        let err = RenderError::Engine("V8 init failed".into());
        assert!(err.to_string().contains("V8 init failed"));
    }

    #[test]
    fn render_error_navigation_display() {
        let err = RenderError::Navigation {
            url: "https://example.com".into(),
            reason: "timeout".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("example.com"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn render_error_selector_not_found_display() {
        let err = RenderError::SelectorNotFound {
            selector: ".missing".into(),
        };
        assert!(err.to_string().contains(".missing"));
    }

    #[test]
    fn rendered_output_render_json_structure() {
        let out = RenderedOutput {
            url: "https://example.com".into(),
            final_url: "https://example.com/final".into(),
            content: "<html></html>".into(),
            eval_result: None,
        };
        let json = out.render_json();
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"content_length\""));
        assert!(json.contains("\"content_length\": 13"));
    }

    #[test]
    fn rendered_output_render_text_is_content() {
        let out = RenderedOutput {
            url: "https://example.com".into(),
            final_url: "https://example.com".into(),
            content: "<p>hello</p>".into(),
            eval_result: None,
        };
        assert_eq!(out.render_text(), "<p>hello</p>");
    }
}
