use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use dom_content_extraction::scraper::{Html, Selector};
use url::Url;

use crate::analyzer::error::AnalyzerError;
use crate::analyzer::headings::{
    self, Headings, HeadingsOutput, HeadingsVerbosity,
};
use crate::analyzer::link::{self, Link, LinkFilter, LinkGroup, LinksOutput};
use crate::analyzer::meta_tag::{
    MetaOutput, MetaTag, MetaVerbosity, extract_meta, select_meta,
};
use crate::analyzer::text::TextOutput;
use crate::analyzer::url_facts::UrlFacts;
use crate::cache::CachedPage;
use crate::client::{ClientError, FetchResult};
use crate::output::RenderOutput;

#[derive(Debug, Clone)]
pub struct StructuredDataSummary {
    pub json_ld_count: usize,
    pub kinds: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub url: String,
    pub final_url: String,
    pub domain: String,
    pub status: u16,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub meta: Vec<MetaTag>,
    pub links: Vec<Link>,
    pub url_facts: UrlFacts,
    pub feeds: Vec<String>,
    pub structured_data: StructuredDataSummary,
    pub headings: Headings,
    pub text_content: Option<String>,
}

impl PageInfo {
    #[allow(dead_code)]
    pub async fn fetch_raw(
        url: &str,
        client: &crate::client::PageClient,
    ) -> Result<FetchResult, AnalyzerError> {
        client.fetch(url).await.map_err(|e| match e {
            ClientError::Fetch { url, status } => {
                AnalyzerError::Fetch { url, status }
            }
            ClientError::Request { url, reason } => {
                AnalyzerError::Parse { url, reason }
            }
            ClientError::InvalidUrl(msg) => AnalyzerError::InvalidUrl(msg),
            ClientError::InvalidProxy(msg) => AnalyzerError::InvalidUrl(msg),
            ClientError::UnknownBrowser(msg) => AnalyzerError::InvalidUrl(msg),
            ClientError::AllAttemptsFailed { url, .. } => {
                AnalyzerError::Fetch { url, status: 0 }
            }
        })
    }

    pub fn from_fetch_result(result: &FetchResult) -> Result<Self, AnalyzerError> {
        Self::from_raw_html(
            &result.input_url,
            &result.final_url,
            result.status,
            result.body.clone(),
        )
    }

    #[allow(dead_code)]
    pub fn from_cached_page(cached: &CachedPage) -> Result<Self, AnalyzerError> {
        Self::from_raw_html(
            &cached.fetch.input_url,
            &cached.fetch.final_url,
            cached.fetch.status,
            cached.html.clone(),
        )
    }

    fn from_raw_html(
        input_url: &str,
        final_url: &str,
        status: u16,
        body: String,
    ) -> Result<Self, AnalyzerError> {
        let parsed = Url::parse(final_url)
            .map_err(|e| AnalyzerError::InvalidUrl(e.to_string()))?;
        let domain = link::extract_registered_domain(&parsed)
            .unwrap_or_else(|| parsed.host_str().unwrap_or("unknown").to_string());
        let document = Html::parse_document(&body);
        let title = extract_title(&document);
        let lang = extract_lang(&document);
        let meta = extract_meta(&document);
        let links = link::extract_links(&document, &parsed);
        let url_facts = UrlFacts::from_links(&links, &domain);
        let feeds = detect_feeds(&links);
        let structured_data = detect_structured_data(&document);
        let headings = headings::extract_headings(&document);
        let text_content = dom_content_extraction::get_content(&document).ok();
        Ok(PageInfo {
            url: input_url.to_string(),
            final_url: final_url.to_string(),
            domain,
            status,
            title,
            lang,
            meta,
            links,
            url_facts,
            feeds,
            structured_data,
            headings,
            text_content,
        })
    }

    #[allow(dead_code)]
    pub fn format_for_llm(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("# Page Analysis: {}\n\n", self.domain));
        out.push_str(&self.format_header());
        out.push_str(&self.format_summary());
        out.push_str(&self.meta_output(MetaVerbosity::Main).render_text());
        out.push_str(&self.format_links_for_llm());
        out.push_str(&self.format_json_for_llm());
        out.push_str(&self.format_content_for_llm());
        out
    }

    #[allow(dead_code)]
    pub fn format_links_for_llm(&self) -> String {
        self.links_output(LinkFilter::All).render_text()
    }

    pub fn meta_tags(&self, verbosity: MetaVerbosity) -> Vec<MetaTag> {
        select_meta(&self.meta, verbosity)
    }

    pub fn meta_output(&self, verbosity: MetaVerbosity) -> MetaOutput {
        MetaOutput {
            url: self.final_url.clone(),
            title: self.title.clone(),
            lang: self.lang.clone(),
            verbosity,
            tags: self.meta_tags(verbosity),
        }
    }

    pub fn links_output(&self, filter: LinkFilter) -> LinksOutput {
        let facts = &self.url_facts;
        let links = self
            .links
            .iter()
            .filter(|link| match filter {
                LinkFilter::All => true,
                LinkFilter::Internal => link.is_internal,
                LinkFilter::External => !link.is_internal,
            })
            .cloned()
            .collect();
        let groups = facts
            .top_first_segments
            .iter()
            .map(|(section, count)| {
                let samples = facts
                    .url_samples_by_section
                    .get(section)
                    .cloned()
                    .unwrap_or_default();
                LinkGroup {
                    section: section.clone(),
                    count: *count,
                    samples,
                }
            })
            .collect();
        LinksOutput {
            url: self.final_url.clone(),
            filter,
            total_internal: facts.total_internal,
            total_external: facts.total_external,
            links,
            groups,
            depth_distribution: facts
                .depth_distribution
                .iter()
                .map(|(depth, count)| (*depth, *count))
                .collect(),
            utility_urls: facts.likely_utility_urls.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn format_meta_for_llm(&self, verbosity: MetaVerbosity) -> String {
        self.meta_output(verbosity).render_text()
    }

    pub fn format_json_for_llm(&self) -> String {
        let mut out = String::new();
        out.push_str("\n## Structured Data\n");
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.load_preset(UTF8_FULL_CONDENSED);
        table.add_row(vec![
            Cell::new("JSON-LD").add_attribute(Attribute::Bold),
            Cell::new(self.structured_data.json_ld_count),
        ]);
        table.add_row(vec![
            Cell::new("Detected").add_attribute(Attribute::Bold),
            Cell::new(if self.structured_data.kinds.is_empty() {
                "none".to_string()
            } else {
                self.structured_data.kinds.join(", ")
            }),
        ]);
        out.push_str(&table.to_string());
        out.push('\n');
        out
    }

    #[allow(dead_code)]
    fn format_header(&self) -> String {
        let mut header = Table::new();
        header.set_content_arrangement(ContentArrangement::Dynamic);
        header.load_preset(UTF8_FULL_CONDENSED);
        header.add_row(vec![
            Cell::new("URL").add_attribute(Attribute::Bold),
            Cell::new(&self.url),
        ]);
        header.add_row(vec![
            Cell::new("Final URL").add_attribute(Attribute::Bold),
            Cell::new(&self.final_url),
        ]);
        header.add_row(vec![
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new(self.status),
        ]);
        header.add_row(vec![
            Cell::new("Domain").add_attribute(Attribute::Bold),
            Cell::new(&self.domain),
        ]);
        if let Some(ref title) = self.title {
            header.add_row(vec![
                Cell::new("Title").add_attribute(Attribute::Bold),
                Cell::new(title),
            ]);
        }
        if let Some(ref lang) = self.lang {
            header.add_row(vec![
                Cell::new("Lang").add_attribute(Attribute::Bold),
                Cell::new(lang),
            ]);
        }
        let mut out = header.to_string();
        out.push('\n');
        out
    }

    #[allow(dead_code)]
    fn format_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("\n## Summary\n");
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.load_preset(UTF8_FULL_CONDENSED);
        table.add_row(vec![
            Cell::new("Internal links").add_attribute(Attribute::Bold),
            Cell::new(self.url_facts.total_internal),
        ]);
        table.add_row(vec![
            Cell::new("External links").add_attribute(Attribute::Bold),
            Cell::new(self.url_facts.total_external),
        ]);
        table.add_row(vec![
            Cell::new("Sections").add_attribute(Attribute::Bold),
            Cell::new(self.url_facts.top_first_segments.len()),
        ]);
        table.add_row(vec![
            Cell::new("Feeds").add_attribute(Attribute::Bold),
            Cell::new(if self.feeds.is_empty() { "no" } else { "yes" }),
        ]);
        table.add_row(vec![
            Cell::new("Structured data").add_attribute(Attribute::Bold),
            Cell::new(if self.structured_data.kinds.is_empty() {
                "no"
            } else {
                "yes"
            }),
        ]);
        out.push_str(&table.to_string());
        out.push('\n');
        out
    }

    #[allow(dead_code)]
    fn format_content_for_llm(&self) -> String {
        let mut out = String::new();
        if let Some(ref text) = self.text_content {
            out.push_str("\n## Extracted Content\n");
            out.push_str(text);
            out.push('\n');
        }
        out
    }

    #[allow(dead_code)]
    pub fn meta_json(&self, verbosity: MetaVerbosity) -> String {
        self.meta_output(verbosity).render_json()
    }

    pub fn json_data_json(&self) -> String {
        let obj = serde_json::json!({
            "url": self.final_url,
            "json_ld_count": self.structured_data.json_ld_count,
            "kinds": self.structured_data.kinds,
        });
        serde_json::to_string_pretty(&obj).unwrap_or_default()
    }

    pub fn headings_output(&self, verbosity: HeadingsVerbosity) -> HeadingsOutput {
        HeadingsOutput {
            url: self.final_url.clone(),
            verbosity,
            headings: headings::select_headings(&self.headings, verbosity),
        }
    }

    pub fn text_output(&self) -> TextOutput {
        TextOutput {
            url: self.final_url.clone(),
            content: self
                .text_content
                .clone()
                .unwrap_or_else(|| "(no content extracted)".to_string()),
        }
    }
}

fn extract_title(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(""))
}

fn extract_lang(document: &Html) -> Option<String> {
    let selector = Selector::parse("html").ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("lang").map(String::from))
}

fn detect_feeds(links: &[link::Link]) -> Vec<String> {
    let mut feeds = std::collections::BTreeSet::new();
    for link in links {
        let lower = link.url.as_str().to_ascii_lowercase();
        if lower.contains("/rss")
            || lower.contains("rss/")
            || lower.contains("/feed")
            || lower.contains("feed/")
            || lower.contains("atom")
        {
            feeds.insert(link.url.to_string());
        }
    }
    feeds.into_iter().collect()
}

fn detect_structured_data(document: &Html) -> StructuredDataSummary {
    let mut json_ld_count = 0;
    let mut kinds = std::collections::BTreeSet::new();
    let selector = match Selector::parse("script") {
        Ok(selector) => selector,
        Err(_) => {
            return StructuredDataSummary {
                json_ld_count,
                kinds: Vec::new(),
            };
        }
    };

    for script in document.select(&selector) {
        if let Some(script_type) = script.value().attr("type")
            && script_type.eq_ignore_ascii_case("application/ld+json")
        {
            json_ld_count += 1;
            kinds.insert("json-ld".to_string());
            continue;
        }

        if let Some(id) = script.value().attr("id")
            && id.eq_ignore_ascii_case("__NEXT_DATA__")
        {
            kinds.insert("next-data".to_string());
            continue;
        }

        let text = script.inner_html();
        let trimmed = text.trim();
        if trimmed.len() > 100
            && ((trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        {
            kinds.insert("inline-json".to_string());
        }
    }

    StructuredDataSummary {
        json_ld_count,
        kinds: kinds.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{CachedFetch, CachedPage};

    const FAKE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <meta name="description" content="Test page description">
    <meta name="robots" content="index, follow">
    <meta name="twitter:title" content="Twitter title">
    <meta name="theme-color" content="#000">
    <meta property="og:type" content="article">
    <meta property="og:title" content="OG title">
    <meta property="article:published_time" content="2026-05-10T16:49:28">
    <meta property="article:tag" id="article:tag:Bitcoin" content="Bitcoin">
    <meta itemprop="datePublished" content="2026-05-10">
    <meta name="viewport" content="width=device-width">
    <title>Test Page Title</title>
    <script type="application/ld+json">{"@type":"NewsArticle"}</script>
</head>
<body>
    <nav>
        <a href="/about">About</a>
        <a href="/contact">Contact</a>
    </nav>
    <main>
        <h1>Main Article Title</h1>
        <h2>Section One</h2>
        <h3>Subsection 1.1</h3>
        <p>This is the main content of the test page. It has enough text for content extraction to work properly. We need multiple sentences to ensure the dom-content-extraction crate can find something meaningful here. The quick brown fox jumps over the lazy dog.</p>
        <a href="https://example.com/news/article-1">First Article</a>
        <a href="https://example.com/news/article-2">Second Article</a>
        <a href="https://other.com/page">External Link</a>
        <a href="/rss/feed.xml">RSS Feed</a>
    </main>
</body>
</html>"##;

    fn fake_cached_page() -> CachedPage {
        CachedPage {
            fetch: CachedFetch {
                input_url: "https://example.com/".to_string(),
                final_url: "https://example.com/".to_string(),
                normalized_final_url: "example.com/".to_string(),
                status: 200,
                fetched_at: "0".to_string(),
            },
            headers: std::collections::HashMap::new(),
            html: FAKE_HTML.to_string(),
        }
    }

    fn fake_fetch_result() -> crate::client::FetchResult {
        crate::client::FetchResult {
            input_url: "https://example.com/".to_string(),
            final_url: "https://example.com/".to_string(),
            status: 200,
            body: FAKE_HTML.to_string(),
            duration_ms: 42,
            attempts: 1,
            ..Default::default()
        }
    }

    #[test]
    fn from_cached_page_extracts_title() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert_eq!(page.title.as_deref(), Some("Test Page Title"));
    }

    #[test]
    fn from_cached_page_extracts_lang() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert_eq!(page.lang.as_deref(), Some("en"));
    }

    #[test]
    fn from_cached_page_extracts_status() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert_eq!(page.status, 200);
    }

    #[test]
    fn from_cached_page_extracts_domain() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert_eq!(page.domain, "example.com");
    }

    #[test]
    fn from_cached_page_extracts_internal_links() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(page.url_facts.total_internal > 0);
    }

    #[test]
    fn from_cached_page_extracts_external_links() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(page.url_facts.total_external > 0);
    }

    #[test]
    fn from_cached_page_detects_feeds() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(!page.feeds.is_empty());
        assert!(page.feeds.iter().any(|f| f.contains("/rss")));
    }

    #[test]
    fn from_cached_page_detects_json_ld() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(page.structured_data.json_ld_count > 0);
        assert!(page.structured_data.kinds.contains(&"json-ld".to_string()));
    }

    #[test]
    fn from_cached_page_extracts_content() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(page.text_content.is_some());
    }

    #[test]
    fn from_cached_page_meta_curated() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(page.meta.iter().any(|m| {
            m.name.as_deref() == Some("description")
                || m.name.as_deref() == Some("og:type")
        }));
    }

    #[test]
    fn from_cached_page_meta_preserves_source_and_id() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert!(page.meta.iter().any(|m| {
            m.name.as_deref() == Some("charset")
                && m.content.as_deref() == Some("utf-8")
                && m.source.as_deref() == Some("charset")
        }));
        assert!(page.meta.iter().any(|m| {
            m.name.as_deref() == Some("content-type")
                && m.source.as_deref() == Some("http-equiv")
        }));
        assert!(page.meta.iter().any(|m| {
            m.name.as_deref() == Some("datePublished")
                && m.source.as_deref() == Some("itemprop")
        }));
        assert!(page.meta.iter().any(|m| {
            m.name.as_deref() == Some("article:tag")
                && m.id.as_deref() == Some("article:tag:Bitcoin")
        }));
    }

    #[test]
    fn meta_tags_main_excludes_low_value_tags() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let tags = page.meta_tags(MetaVerbosity::Main);
        assert!(
            tags.iter()
                .any(|m| m.name.as_deref() == Some("description"))
        );
        assert!(tags.iter().any(|m| m.name.as_deref() == Some("og:title")));
        assert!(
            tags.iter()
                .any(|m| { m.name.as_deref() == Some("article:published_time") })
        );
        assert!(!tags.iter().any(|m| m.name.as_deref() == Some("viewport")));
        assert!(
            !tags
                .iter()
                .any(|m| m.name.as_deref() == Some("theme-color"))
        );
        assert!(!tags.iter().any(|m| m.name.as_deref() == Some("charset")));
    }

    #[test]
    fn meta_tags_extended_includes_protocol_tags() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let tags = page.meta_tags(MetaVerbosity::Extended);
        assert!(tags.iter().any(|m| m.name.as_deref() == Some("charset")));
        assert!(
            tags.iter()
                .any(|m| { m.name.as_deref() == Some("content-type") })
        );
        assert!(
            tags.iter()
                .any(|m| { m.name.as_deref() == Some("twitter:title") })
        );
        assert!(!tags.iter().any(|m| m.name.as_deref() == Some("viewport")));
        assert!(
            !tags
                .iter()
                .any(|m| m.name.as_deref() == Some("theme-color"))
        );
    }

    #[test]
    fn meta_tags_all_includes_every_meta_tag() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let tags = page.meta_tags(MetaVerbosity::All);
        assert_eq!(tags.len(), page.meta.len());
        assert!(tags.iter().any(|m| m.name.as_deref() == Some("viewport")));
        assert!(
            tags.iter()
                .any(|m| m.name.as_deref() == Some("theme-color"))
        );
    }

    #[test]
    fn format_for_llm_produces_output() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.format_for_llm();
        assert!(out.contains("# Page Analysis: example.com"));
        assert!(out.contains("Test Page Title"));
    }

    #[test]
    fn format_links_for_llm_includes_sections() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.format_links_for_llm();
        assert!(out.contains("## URL Groups") || out.contains("## Path Depth"));
    }

    #[test]
    fn format_meta_for_llm_curated_only() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.format_meta_for_llm(MetaVerbosity::Main);
        assert!(out.contains("description"));
        assert!(!out.contains("viewport"));
    }

    #[test]
    fn format_json_for_llm_shows_structured_data() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.format_json_for_llm();
        assert!(out.contains("## Structured Data"));
        assert!(out.contains("json-ld"));
    }

    #[test]
    fn from_cached_page_invalid_url() {
        let mut cp = fake_cached_page();
        cp.fetch.final_url = "not a url".to_string();
        assert!(PageInfo::from_cached_page(&cp).is_err());
    }

    #[test]
    fn empty_html_page() {
        let cp = CachedPage {
            fetch: CachedFetch {
                input_url: "https://example.com/".to_string(),
                final_url: "https://example.com/".to_string(),
                normalized_final_url: "example.com/".to_string(),
                status: 200,
                fetched_at: "0".to_string(),
            },
            headers: std::collections::HashMap::new(),
            html: "<html><body></body></html>".to_string(),
        };
        let page = PageInfo::from_cached_page(&cp).unwrap();
        assert!(page.title.is_none());
        assert!(page.lang.is_none());
        assert_eq!(page.url_facts.total_internal, 0);
        assert_eq!(page.url_facts.total_external, 0);
        assert!(page.feeds.is_empty());
    }

    #[test]
    fn from_fetch_result_extracts_title() {
        let result = fake_fetch_result();
        let page = PageInfo::from_fetch_result(&result).unwrap();
        assert_eq!(page.title.as_deref(), Some("Test Page Title"));
    }

    #[test]
    fn from_fetch_result_extracts_domain() {
        let result = fake_fetch_result();
        let page = PageInfo::from_fetch_result(&result).unwrap();
        assert_eq!(page.domain, "example.com");
    }

    #[test]
    fn from_fetch_result_invalid_url() {
        let mut result = fake_fetch_result();
        result.final_url = "not a url".to_string();
        assert!(PageInfo::from_fetch_result(&result).is_err());
    }

    #[test]
    fn from_cached_page_extracts_headings() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        assert_eq!(page.headings.h1, vec!["Main Article Title"]);
        assert_eq!(page.headings.h2, vec!["Section One"]);
        assert_eq!(page.headings.h3, vec!["Subsection 1.1"]);
    }

    #[test]
    fn headings_output_main_shows_h1_only() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.headings_output(HeadingsVerbosity::Main);
        assert_eq!(out.headings.h1, vec!["Main Article Title"]);
        assert!(out.headings.h2.is_empty());
        assert!(out.headings.h3.is_empty());
    }

    #[test]
    fn headings_output_extended_shows_h1_h2() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.headings_output(HeadingsVerbosity::Extended);
        assert_eq!(out.headings.h1, vec!["Main Article Title"]);
        assert_eq!(out.headings.h2, vec!["Section One"]);
        assert!(out.headings.h3.is_empty());
    }

    #[test]
    fn headings_output_all_shows_all() {
        let page = PageInfo::from_cached_page(&fake_cached_page()).unwrap();
        let out = page.headings_output(HeadingsVerbosity::All);
        assert_eq!(out.headings.h1, vec!["Main Article Title"]);
        assert_eq!(out.headings.h2, vec!["Section One"]);
        assert_eq!(out.headings.h3, vec!["Subsection 1.1"]);
    }
}
