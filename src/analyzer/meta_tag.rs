use dom_content_extraction::scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::output::RenderOutput;

static META_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    pub name: Option<String>,
    pub content: Option<String>,
    pub source: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetaOutput {
    pub url: String,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub verbosity: MetaVerbosity,
    pub tags: Vec<MetaTag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaVerbosity {
    Main,
    Extended,
    All,
}

impl MetaVerbosity {
    /// Parses a CLI verbosity value into a metadata verbosity level.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "main" => Some(Self::Main),
            "extended" => Some(Self::Extended),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// Returns the stable CLI/JSON spelling for this verbosity level.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Extended => "extended",
            Self::All => "all",
        }
    }
}

/// Extracts all `<meta>` tags from a parsed HTML document in document order.
pub fn extract_meta(document: &Html) -> Vec<MetaTag> {
    document
        .select(&META_SELECTOR)
        .filter_map(|el| {
            let attrs = el.value();
            let source = ["name", "property", "http-equiv", "itemprop", "charset"]
                .into_iter()
                .find(|attr| attrs.attr(attr).is_some());
            let name = source.map(|attr| {
                if attr == "charset" {
                    "charset".to_string()
                } else {
                    attrs.attr(attr).unwrap_or_default().to_string()
                }
            });
            let content = attrs
                .attr("content")
                .or_else(|| attrs.attr("charset"))
                .map(String::from);
            let source = source.map(String::from);
            let id = attrs.attr("id").map(String::from);

            if name.is_some() || content.is_some() {
                Some(MetaTag {
                    name,
                    content,
                    source,
                    id,
                })
            } else {
                None
            }
        })
        .collect()
}

impl MetaOutput {
    fn render_value(&self) -> serde_json::Value {
        serde_json::json!({
            "url": &self.url,
            "title": &self.title,
            "lang": &self.lang,
            "verbosity": self.verbosity.as_str(),
            "tags": &self.tags,
        })
    }
}

impl RenderOutput for MetaOutput {
    fn render_text(&self) -> String {
        if self.tags.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        out.push_str("## Metadata\n");
        out.push_str(&format!("URL: {}\n", self.url));
        if let Some(title) = &self.title {
            out.push_str(&format!("Title: {title}\n"));
        }
        if let Some(lang) = &self.lang {
            out.push_str(&format!("Lang: {lang}\n"));
        }
        out.push_str(&format!("Verbosity: {}\n", self.verbosity.as_str()));
        out.push_str("Source\tProperty\tContent\n");
        for tag in &self.tags {
            let source = tag.source.as_deref().unwrap_or("");
            let name = tag.name.as_deref().unwrap_or("(unnamed)");
            let content = tag.content.as_deref().unwrap_or("");
            out.push_str(&format!("{source}\t{name}\t{content}\n"));
        }
        out
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.render_value()).unwrap_or_default()
    }

    fn render_toon(&self) -> String {
        toon_format::encode_default(&self.render_value()).unwrap_or_default()
    }
}

/// Selects metadata tags for the requested verbosity while preserving order.
pub fn select_meta(meta: &[MetaTag], verbosity: MetaVerbosity) -> Vec<MetaTag> {
    if verbosity == MetaVerbosity::All {
        return meta.to_vec();
    }

    let mut selected = Vec::new();
    for tag in meta {
        let Some(name) = tag.name.as_deref() else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        let keep = match verbosity {
            MetaVerbosity::Main => is_main_meta(&lower),
            MetaVerbosity::Extended => is_extended_meta(&lower),
            MetaVerbosity::All => true,
        };
        if keep {
            selected.push(tag.clone());
        }
    }
    selected
}

/// Returns true for high-signal page and article metadata.
fn is_main_meta(name: &str) -> bool {
    name.starts_with("article:")
        || matches!(
            name,
            "author"
                | "description"
                | "news_keywords"
                | "og:description"
                | "og:image"
                | "og:site_name"
                | "og:title"
                | "og:type"
                | "og:url"
        )
}

/// Returns true for main metadata plus crawler and card protocol metadata.
fn is_extended_meta(name: &str) -> bool {
    is_main_meta(name)
        || name.starts_with("og:")
        || name.starts_with("twitter:")
        || matches!(
            name,
            "charset"
                | "content-language"
                | "content-type"
                | "keywords"
                | "language"
                | "page-category"
                | "robots"
                | "section"
                | "category"
        )
}
