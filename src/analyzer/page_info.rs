use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use dom_content_extraction::scraper::{Html, Selector};
use url::Url;

use crate::analyzer::error::AnalyzerError;
use crate::analyzer::link;
use crate::analyzer::meta_tag::MetaTag;
use crate::analyzer::url_facts::UrlFacts;
use crate::cache::CachedPage;

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
    pub url_facts: UrlFacts,
    pub feeds: Vec<String>,
    pub structured_data: StructuredDataSummary,
    pub text_content: Option<String>,
}

impl PageInfo {
    pub async fn fetch_raw(
        url: &str,
        client: &wreq::Client,
    ) -> Result<CachedPage, AnalyzerError> {
        let parsed = Url::parse(url)
            .map_err(|e| AnalyzerError::InvalidUrl(e.to_string()))?;

        let response = client.get(parsed.clone()).send().await.map_err(|_| {
            AnalyzerError::Fetch {
                url: url.to_string(),
                status: 0,
            }
        })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            return Err(AnalyzerError::Fetch {
                url: url.to_string(),
                status,
            });
        }

        let final_url = response.url().to_string();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (k.to_string(), v.to_str().unwrap_or("<invalid>").to_string())
            })
            .collect();

        let body = response.text().await.map_err(|_| AnalyzerError::Parse {
            url: url.to_string(),
            reason: "failed to read response body".to_string(),
        })?;

        let cache =
            crate::cache::FileCache::new(crate::cache::CacheConfig::default());
        cache
            .cache_page(url, &final_url, status, headers, body)
            .map_err(|e| AnalyzerError::Parse {
                url: url.to_string(),
                reason: e.to_string(),
            })
    }

    pub fn from_cached_page(cached: CachedPage) -> Result<Self, AnalyzerError> {
        Self::from_raw_html(
            &cached.fetch.input_url,
            &cached.fetch.final_url,
            cached.fetch.status,
            cached.html,
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
        let text_content = dom_content_extraction::get_content(&document).ok();
        Ok(PageInfo {
            url: input_url.to_string(),
            final_url: final_url.to_string(),
            domain,
            status,
            title,
            lang,
            meta,
            url_facts,
            feeds,
            structured_data,
            text_content,
        })
    }

    pub fn format_for_llm(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("# Page Analysis: {}\n\n", self.domain));
        out.push_str(&self.format_header());
        out.push_str(&self.format_summary());
        out.push_str(&self.format_meta_for_llm());
        out.push_str(&self.format_links_for_llm());
        out.push_str(&self.format_json_for_llm());
        out.push_str(&self.format_content_for_llm());
        out
    }

    pub fn format_links_for_llm(&self) -> String {
        let mut out = String::new();
        let facts = &self.url_facts;

        if !facts.top_first_segments.is_empty() {
            out.push_str("\n## URL Groups\n");

            if let Some(ref pattern) = facts.detected_url_pattern() {
                out.push_str(&format!("Detected article pattern: {}\n", pattern));
            }

            let mut sections_table = Table::new();
            sections_table.set_content_arrangement(ContentArrangement::Dynamic);
            sections_table.load_preset(UTF8_FULL_CONDENSED);
            sections_table.set_header(vec![
                Cell::new("Section").add_attribute(Attribute::Bold),
                Cell::new("Links")
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new("Sample URLs").add_attribute(Attribute::Bold),
            ]);

            for (segment, count) in &facts.top_first_segments {
                let samples = facts
                    .url_samples_by_section
                    .get(segment)
                    .map(|urls| urls.join("\n"))
                    .unwrap_or_default();
                sections_table.add_row(vec![
                    Cell::new(segment),
                    Cell::new(count).set_alignment(CellAlignment::Right),
                    Cell::new(&samples),
                ]);
            }

            out.push_str(&sections_table.to_string());
            out.push('\n');
        }

        if !facts.depth_distribution.is_empty() {
            out.push_str("\n## Path Depth\n");
            let mut depth_table = Table::new();
            depth_table.set_content_arrangement(ContentArrangement::Dynamic);
            depth_table.load_preset(UTF8_FULL_CONDENSED);
            depth_table.set_header(vec![
                Cell::new("Depth").add_attribute(Attribute::Bold),
                Cell::new("Count").add_attribute(Attribute::Bold),
            ]);
            for (depth, count) in &facts.depth_distribution {
                depth_table.add_row(vec![Cell::new(depth), Cell::new(count)]);
            }
            out.push_str(&depth_table.to_string());
            out.push('\n');
        }

        if !facts.likely_utility_urls.is_empty() {
            out.push_str("\n## Utility URLs\n");
            let mut util_table = Table::new();
            util_table.set_content_arrangement(ContentArrangement::Dynamic);
            util_table.load_preset(UTF8_FULL_CONDENSED);
            for url in &facts.likely_utility_urls {
                util_table.add_row(vec![Cell::new(url)]);
            }
            out.push_str(&util_table.to_string());
            out.push('\n');
        }

        out
    }

    pub fn format_meta_for_llm(&self) -> String {
        let curated = curated_meta(&self.meta);
        if curated.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        out.push_str("\n## Curated Metadata\n");
        let mut meta_table = Table::new();
        meta_table.set_content_arrangement(ContentArrangement::Dynamic);
        meta_table.load_preset(UTF8_FULL_CONDENSED);
        meta_table.set_header(vec![
            Cell::new("Property").add_attribute(Attribute::Bold),
            Cell::new("Content").add_attribute(Attribute::Bold),
        ]);
        for tag in curated {
            let name = tag.name.as_deref().unwrap_or("(unnamed)");
            let content = tag.content.as_deref().unwrap_or("");
            meta_table.add_row(vec![Cell::new(name), Cell::new(content)]);
        }
        out.push_str(&meta_table.to_string());
        out.push('\n');
        out
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

    fn format_content_for_llm(&self) -> String {
        let mut out = String::new();
        if let Some(ref text) = self.text_content {
            out.push_str("\n## Extracted Content\n");
            out.push_str(text);
            out.push('\n');
        }
        out
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

fn extract_meta(document: &Html) -> Vec<MetaTag> {
    let selector = match Selector::parse("meta") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    document
        .select(&selector)
        .filter_map(|el| {
            let attrs = el.value();
            let name = attrs
                .attr("name")
                .or_else(|| attrs.attr("property"))
                .map(String::from);
            let content = attrs.attr("content").map(String::from);

            if name.is_some() || content.is_some() {
                Some(MetaTag { name, content })
            } else {
                None
            }
        })
        .collect()
}

fn curated_meta(meta: &[MetaTag]) -> Vec<MetaTag> {
    let mut curated = Vec::new();
    for tag in meta {
        let Some(name) = tag.name.as_deref() else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        let keep = matches!(
            lower.as_str(),
            "description"
                | "robots"
                | "og:type"
                | "og:locale"
                | "content-language"
                | "language"
                | "page-category"
                | "article:section"
                | "section"
                | "category"
        );
        if keep {
            curated.push(tag.clone());
        }
    }
    curated
}

fn detect_feeds(links: &[link::Link]) -> Vec<String> {
    let mut feeds = std::collections::BTreeSet::new();
    for link in links {
        let lower = link.url.to_ascii_lowercase();
        if lower.contains("/rss")
            || lower.contains("rss/")
            || lower.contains("/feed")
            || lower.contains("feed/")
            || lower.contains("atom")
        {
            feeds.insert(link.url.clone());
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
