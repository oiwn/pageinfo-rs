use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use dom_content_extraction::scraper::{Html, Selector};
use url::Url;

use crate::analyzer::error::AnalyzerError;
use crate::analyzer::link::{self, Link};
use crate::analyzer::meta_tag::MetaTag;
use crate::analyzer::url_facts::UrlFacts;

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
    pub raw_html: String,
    pub text_content: Option<String>,
}

impl PageInfo {
    pub async fn fetch(
        url: &str,
        client: &wreq::Client,
    ) -> Result<Self, AnalyzerError> {
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
        let domain = link::extract_registered_domain(&parsed)
            .unwrap_or_else(|| parsed.host_str().unwrap_or("unknown").to_string());

        let body = response.text().await.map_err(|_| AnalyzerError::Parse {
            url: url.to_string(),
            reason: "failed to read response body".to_string(),
        })?;

        let document = Html::parse_document(&body);

        let title = extract_title(&document);
        let lang = extract_lang(&document);
        let meta = extract_meta(&document);
        let base_url = parsed.clone();
        let links = link::extract_links(&document, &base_url);
        let url_facts = UrlFacts::from_links(&links, &domain);

        let text_content = dom_content_extraction::get_content(&document).ok();

        Ok(PageInfo {
            url: url.to_string(),
            final_url,
            domain,
            status,
            title,
            lang,
            meta,
            links,
            url_facts,
            raw_html: body,
            text_content,
        })
    }

    pub fn format_for_llm(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("# Page Analysis: {}\n\n", self.domain));

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
        out.push_str(&header.to_string());
        out.push('\n');

        out.push_str(&format!(
            "\n## Links: {} internal | {} external\n",
            self.url_facts.total_internal, self.url_facts.total_external
        ));

        if !self.meta.is_empty() {
            out.push_str("\n## Meta Tags\n");
            let mut meta_table = Table::new();
            meta_table.set_content_arrangement(ContentArrangement::Dynamic);
            meta_table.load_preset(UTF8_FULL_CONDENSED);
            meta_table.set_header(vec![
                Cell::new("Property").add_attribute(Attribute::Bold),
                Cell::new("Content").add_attribute(Attribute::Bold),
            ]);
            for tag in &self.meta {
                let name = tag.name.as_deref().unwrap_or("(unnamed)");
                let content = tag.content.as_deref().unwrap_or("");
                meta_table.add_row(vec![Cell::new(name), Cell::new(content)]);
            }
            out.push_str(&meta_table.to_string());
            out.push('\n');
        }

        out.push_str(&self.format_url_facts());

        if let Some(ref text) = self.text_content {
            out.push_str("\n## Extracted Content\n");
            out.push_str(text);
            out.push('\n');
        }

        out
    }

    fn format_url_facts(&self) -> String {
        let mut out = String::new();
        let facts = &self.url_facts;

        if !facts.top_first_segments.is_empty() {
            out.push_str("\n## URL Patterns\n");

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
