use serde::{Deserialize, Serialize};

use crate::output::RenderOutput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOutput {
    pub url: String,
    pub content: String,
}

impl TextOutput {
    fn render_value(&self) -> serde_json::Value {
        serde_json::json!({
            "url": &self.url,
            "content": &self.content,
            "content_length": self.content.len(),
        })
    }
}

impl RenderOutput for TextOutput {
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
