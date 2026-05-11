use std::sync::LazyLock;

use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use dom_content_extraction::scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::output::RenderOutput;

static A_HREF: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href]").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLink {
    pub href: String,
    pub text: Option<String>,
    pub rel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub raw_url: String,
    pub url: Url,
    pub text: Option<String>,
    pub rel: Option<String>,
    pub is_internal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkFilter {
    All,
    Internal,
    External,
}

impl LinkFilter {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "internal" => Some(Self::Internal),
            "external" => Some(Self::External),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Internal => "internal",
            Self::External => "external",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGroup {
    pub section: String,
    pub count: usize,
    pub samples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LinksOutput {
    pub url: String,
    pub filter: LinkFilter,
    pub total_internal: usize,
    pub total_external: usize,
    pub links: Vec<Link>,
    pub groups: Vec<LinkGroup>,
    pub depth_distribution: Vec<(usize, usize)>,
    pub utility_urls: Vec<String>,
}

/// Extract raw `<a href>` evidence from an HTML document in document order.
pub fn extract_raw_links(document: &Html) -> Vec<RawLink> {
    document
        .select(&A_HREF)
        .filter_map(|element| {
            let href = element.value().attr("href")?;
            let text = element
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            let text = if text.is_empty() { None } else { Some(text) };

            Some(RawLink {
                href: href.to_string(),
                text,
                rel: element.value().attr("rel").map(String::from),
            })
        })
        .collect()
}

/// Extract all links from an HTML document, resolving relative URLs against
/// the page's base URL. Returned links are normalized (lowercase host, no fragment).
pub fn extract_links(document: &Html, base_url: &Url) -> Vec<Link> {
    let opts = LinkOptions {
        normalize: true,
        ..Default::default()
    };
    extract_links_inner(document, base_url, &opts)
}

fn extract_links_inner(
    document: &Html,
    base_url: &Url,
    opts: &LinkOptions,
) -> Vec<Link> {
    let page_domain = extract_registered_domain(base_url);

    let mut links: Vec<Link> = extract_raw_links(document)
        .into_iter()
        .filter_map(|raw| {
            let resolved = base_url.join(&raw.href).ok()?;
            let resolved_domain = extract_registered_domain(&resolved);
            let is_internal = match (&page_domain, &resolved_domain) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };

            Some(Link {
                raw_url: raw.href,
                url: resolved,
                text: raw.text,
                rel: raw.rel,
                is_internal,
            })
        })
        .collect();

    for link in &mut links {
        if opts.normalize {
            link.normalize();
        }
        if opts.strip_tracking_params {
            link.strip_tracking();
        }
    }

    if opts.max > 0 && links.len() > opts.max {
        links.truncate(opts.max);
    }

    links
}

/// Extract the registered domain using the Public Suffix List
/// (e.g. "example.com" from "https://www.example.com/page").
pub fn extract_registered_domain(url: &Url) -> Option<String> {
    psl::domain_str(url.host_str()?).map(String::from)
}

impl Link {
    pub fn normalize(&mut self) {
        if let Some(host) = self.url.host_str().map(|h| h.to_ascii_lowercase()) {
            let _ = self.url.set_host(Some(&host));
        }
        self.url.set_fragment(None);
    }

    pub fn strip_tracking(&mut self) {
        const TRACKING: &[&str] = &[
            "utm_source",
            "utm_medium",
            "utm_campaign",
            "utm_term",
            "utm_content",
            "utm_id",
            "fbclid",
            "gclid",
        ];

        let pairs: Vec<(String, String)> = self
            .url
            .query_pairs()
            .filter(|(k, _)| !TRACKING.iter().any(|tk| k.eq_ignore_ascii_case(tk)))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        if pairs.is_empty() {
            self.url.set_query(None);
        } else {
            self.url.query_pairs_mut().clear().extend_pairs(pairs);
        }
    }

    #[allow(dead_code)]
    pub fn is_same_host(&self, other: &Url) -> bool {
        self.url.host_str() == other.host_str()
    }

    #[allow(dead_code)]
    pub fn is_asset(&self) -> bool {
        const EXTS: &[&str] = &[
            "css", "js", "mjs", "png", "jpg", "jpeg", "gif", "svg", "webp", "ico",
            "bmp", "woff", "woff2", "ttf", "eot", "otf", "mp4", "webm", "mp3",
            "ogg", "wav", "pdf", "doc", "docx", "xls", "xlsx", "zip", "tar", "gz",
        ];

        if let Some(ext) = self.url.path().rsplit('.').next() {
            return EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext));
        }
        false
    }
}

impl LinksOutput {
    fn render_value(&self) -> serde_json::Value {
        let depth_distribution: Vec<serde_json::Value> = self
            .depth_distribution
            .iter()
            .map(|(depth, count)| serde_json::json!([depth, count]))
            .collect();
        let links: Vec<serde_json::Value> = self
            .links
            .iter()
            .map(|link| {
                serde_json::json!({
                    "raw_url": link.raw_url,
                    "url": link.url.as_str(),
                    "text": link.text,
                    "rel": link.rel,
                    "is_internal": link.is_internal,
                })
            })
            .collect();
        let mut obj = serde_json::json!({
            "url": &self.url,
            "filter": self.filter.as_str(),
            "total_internal": self.total_internal,
            "total_external": self.total_external,
            "links": links,
            "groups": &self.groups,
            "depth_distribution": depth_distribution,
            "utility_urls": &self.utility_urls,
        });

        if self.filter == LinkFilter::Internal {
            obj.as_object_mut().unwrap().remove("total_external");
        }
        if self.filter == LinkFilter::External {
            obj.as_object_mut().unwrap().remove("total_internal");
        }

        obj
    }
}

impl RenderOutput for LinksOutput {
    fn render_text(&self) -> String {
        let mut out = String::new();

        out.push_str("\n## Links\n");
        out.push_str(&format!("URL: {}\n", self.url));
        out.push_str(&format!("Filter: {}\n", self.filter.as_str()));
        out.push_str(&format!("Internal: {}\n", self.total_internal));
        out.push_str(&format!("External: {}\n", self.total_external));

        let mut links_table = Table::new();
        links_table.set_content_arrangement(ContentArrangement::Dynamic);
        links_table.load_preset(UTF8_FULL_CONDENSED);
        links_table.set_header(vec![
            Cell::new("Type").add_attribute(Attribute::Bold),
            Cell::new("URL").add_attribute(Attribute::Bold),
            Cell::new("Raw").add_attribute(Attribute::Bold),
            Cell::new("Text").add_attribute(Attribute::Bold),
            Cell::new("Rel").add_attribute(Attribute::Bold),
        ]);

        for link in &self.links {
            links_table.add_row(vec![
                Cell::new(if link.is_internal {
                    "internal"
                } else {
                    "external"
                }),
                Cell::new(link.url.as_str()),
                Cell::new(&link.raw_url),
                Cell::new(link.text.as_deref().unwrap_or("")),
                Cell::new(link.rel.as_deref().unwrap_or("")),
            ]);
        }

        if self.links.is_empty() {
            out.push_str("(no links matched)\n");
        } else {
            out.push_str(&links_table.to_string());
            out.push('\n');
        }

        if !self.groups.is_empty() {
            out.push_str("\n## URL Groups\n");

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

            for group in &self.groups {
                sections_table.add_row(vec![
                    Cell::new(&group.section),
                    Cell::new(group.count).set_alignment(CellAlignment::Right),
                    Cell::new(group.samples.join("\n")),
                ]);
            }

            out.push_str(&sections_table.to_string());
            out.push('\n');
        }

        if !self.depth_distribution.is_empty() {
            out.push_str("\n## Path Depth\n");
            let mut depth_table = Table::new();
            depth_table.set_content_arrangement(ContentArrangement::Dynamic);
            depth_table.load_preset(UTF8_FULL_CONDENSED);
            depth_table.set_header(vec![
                Cell::new("Depth").add_attribute(Attribute::Bold),
                Cell::new("Count").add_attribute(Attribute::Bold),
            ]);
            for (depth, count) in &self.depth_distribution {
                depth_table.add_row(vec![Cell::new(depth), Cell::new(count)]);
            }
            out.push_str(&depth_table.to_string());
            out.push('\n');
        }

        if !self.utility_urls.is_empty() {
            out.push_str("\n## Utility URLs\n");
            let mut util_table = Table::new();
            util_table.set_content_arrangement(ContentArrangement::Dynamic);
            util_table.load_preset(UTF8_FULL_CONDENSED);
            for url in &self.utility_urls {
                util_table.add_row(vec![Cell::new(url)]);
            }
            out.push_str(&util_table.to_string());
            out.push('\n');
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

#[derive(Debug, Clone, Default)]
pub struct LinkOptions {
    pub normalize: bool,
    pub strip_tracking_params: bool,
    pub max: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link(url: &str, is_internal: bool) -> Link {
        Link {
            raw_url: url.to_string(),
            url: Url::parse(url).unwrap(),
            text: None,
            rel: None,
            is_internal,
        }
    }

    #[test]
    fn extract_registered_domain_simple() {
        let url = Url::parse("https://example.com/page").unwrap();
        assert_eq!(extract_registered_domain(&url), Some("example.com".into()));
    }

    #[test]
    fn extract_registered_domain_www() {
        let url = Url::parse("https://www.example.com/page").unwrap();
        assert_eq!(extract_registered_domain(&url), Some("example.com".into()));
    }

    #[test]
    fn extract_registered_domain_multi_part_tld() {
        let url = Url::parse("https://www.example.co.uk/page").unwrap();
        assert_eq!(
            extract_registered_domain(&url),
            Some("example.co.uk".into())
        );
    }

    #[test]
    fn extract_registered_domain_multi_part_jp() {
        let url = Url::parse("https://www.example.co.jp/page").unwrap();
        assert_eq!(
            extract_registered_domain(&url),
            Some("example.co.jp".into())
        );
    }

    #[test]
    fn extract_registered_domain_no_host() {
        let url = Url::parse("file:///path/to/file").unwrap();
        assert_eq!(extract_registered_domain(&url), None);
    }

    #[test]
    fn extract_raw_links_preserves_href_text_rel_and_order() {
        let html = r#"<html><body>
            <a href="/about" rel="nofollow"> About </a>
            <a href=" contact "></a>
            <a href="https://other.com/page">External</a>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let links = extract_raw_links(&doc);

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].href, "/about");
        assert_eq!(links[0].text.as_deref(), Some("About"));
        assert_eq!(links[0].rel.as_deref(), Some("nofollow"));
        assert_eq!(links[1].href, " contact ");
        assert!(links[1].text.is_none());
        assert_eq!(links[2].href, "https://other.com/page");
    }

    #[test]
    fn extract_links_basic() {
        let html = r#"<html><body>
            <a href="https://example.com/about">About Us</a>
            <a href="https://other.com/page">External</a>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);

        assert_eq!(links.len(), 2);
        assert!(links[0].is_internal);
        assert!(!links[1].is_internal);
    }

    #[test]
    fn extract_links_resolves_relative() {
        let html = r#"<html><body>
            <a href="/about">About</a>
            <a href="contact">Contact</a>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);

        assert_eq!(links[0].raw_url, "/about");
        assert_eq!(links[0].url.as_str(), "https://example.com/about");
        assert_eq!(links[1].raw_url, "contact");
        assert_eq!(links[1].url.as_str(), "https://example.com/contact");
    }

    #[test]
    fn extract_links_text_and_rel() {
        let html = r#"<html><body>
            <a href="https://example.com" rel="nofollow">Click here</a>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);

        assert_eq!(links[0].text.as_deref(), Some("Click here"));
        assert_eq!(links[0].rel.as_deref(), Some("nofollow"));
    }

    #[test]
    fn extract_links_empty_text() {
        let html = r#"<html><body>
            <a href="https://example.com">  </a>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);

        assert!(links[0].text.is_none());
    }

    #[test]
    fn extract_links_no_href_ignored() {
        let html = r#"<html><body>
            <a name="anchor">No href</a>
            <a href="https://example.com">Has href</a>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);

        assert_eq!(links.len(), 1);
    }

    #[test]
    fn normalize_lowercases_host() {
        let mut link = make_link("https://EXAMPLE.COM/Page", true);
        link.normalize();
        assert_eq!(link.url.as_str(), "https://example.com/Page");
    }

    #[test]
    fn normalize_drops_fragment() {
        let mut link = make_link("https://example.com/page#section", true);
        link.normalize();
        assert_eq!(link.url.as_str(), "https://example.com/page");
    }

    #[test]
    fn normalize_preserves_query() {
        let mut link = make_link("https://example.com/page?foo=bar", true);
        link.normalize();
        assert_eq!(link.url.as_str(), "https://example.com/page?foo=bar");
    }

    #[test]
    fn normalize_handles_mailto() {
        let mut link = make_link("mailto:foo@bar.com", false);
        link.normalize();
        assert_eq!(link.url.as_str(), "mailto:foo@bar.com");
    }

    #[test]
    fn strip_tracking_removes_utm() {
        let mut link =
            make_link("https://example.com/page?utm_source=fb&keep=1", true);
        link.strip_tracking();
        assert_eq!(link.url.as_str(), "https://example.com/page?keep=1");
    }

    #[test]
    fn strip_tracking_removes_fbclid_gclid() {
        let mut link =
            make_link("https://example.com/page?fbclid=abc&gclid=def&real=1", true);
        link.strip_tracking();
        assert_eq!(link.url.as_str(), "https://example.com/page?real=1");
    }

    #[test]
    fn strip_tracking_case_insensitive() {
        let mut link =
            make_link("https://example.com/page?UTM_SOURCE=x&utm_medium=y", true);
        link.strip_tracking();
        assert_eq!(link.url.as_str(), "https://example.com/page");
    }

    #[test]
    fn strip_tracking_removes_all_query_when_only_tracking() {
        let mut link =
            make_link("https://example.com/page?utm_source=x&utm_medium=y", true);
        link.strip_tracking();
        assert_eq!(link.url.as_str(), "https://example.com/page");
    }

    #[test]
    fn strip_tracking_preserves_no_query_url() {
        let mut link = make_link("https://example.com/page", true);
        link.strip_tracking();
        assert_eq!(link.url.as_str(), "https://example.com/page");
    }

    #[test]
    fn link_options_defaults() {
        let opts = LinkOptions::default();
        assert!(!opts.normalize);
        assert!(!opts.strip_tracking_params);
        assert_eq!(opts.max, 0);
    }

    #[test]
    fn links_output_json_includes_selected_facts() {
        let output = LinksOutput {
            url: "https://example.com/".to_string(),
            filter: LinkFilter::All,
            total_internal: 2,
            total_external: 1,
            links: vec![Link {
                raw_url: "/docs".to_string(),
                url: Url::parse("https://example.com/docs").unwrap(),
                text: Some("Docs".to_string()),
                rel: None,
                is_internal: true,
            }],
            groups: vec![LinkGroup {
                section: "docs".to_string(),
                count: 2,
                samples: vec!["/docs".to_string()],
            }],
            depth_distribution: vec![(1, 2)],
            utility_urls: vec!["https://example.com/privacy".to_string()],
        };

        let parsed: serde_json::Value =
            serde_json::from_str(&output.render_json()).unwrap();
        assert_eq!(parsed["url"], "https://example.com/");
        assert_eq!(parsed["filter"], "all");
        assert_eq!(parsed["total_internal"], 2);
        assert_eq!(parsed["total_external"], 1);
        assert_eq!(parsed["links"][0]["raw_url"], "/docs");
        assert_eq!(parsed["links"][0]["url"], "https://example.com/docs");
        assert_eq!(parsed["links"][0]["text"], "Docs");
        assert_eq!(parsed["links"][0]["is_internal"], true);
        assert_eq!(parsed["groups"][0]["section"], "docs");
        assert_eq!(parsed["depth_distribution"][0][0], 1);
    }

    #[test]
    fn links_output_toon_uses_same_value() {
        let output = LinksOutput {
            url: "https://example.com/".to_string(),
            filter: LinkFilter::Internal,
            total_internal: 2,
            total_external: 1,
            links: Vec::new(),
            groups: Vec::new(),
            depth_distribution: Vec::new(),
            utility_urls: Vec::new(),
        };

        let expected = toon_format::encode_default(&output.render_value()).unwrap();
        assert_eq!(output.render_toon(), expected);
        assert!(!output.render_json().contains("total_external"));
    }

    #[test]
    fn is_same_host_match() {
        let link = make_link("https://example.com/page", true);
        let other = Url::parse("https://example.com/other").unwrap();
        assert!(link.is_same_host(&other));
    }

    #[test]
    fn is_same_host_different_subdomain() {
        let link = make_link("https://www.example.com/page", true);
        let other = Url::parse("https://blog.example.com/other").unwrap();
        assert!(!link.is_same_host(&other));
    }

    #[test]
    fn is_same_host_different_domain() {
        let link = make_link("https://example.com/page", true);
        let other = Url::parse("https://other.com/page").unwrap();
        assert!(!link.is_same_host(&other));
    }

    #[test]
    fn is_asset_css() {
        let link = make_link("https://example.com/static/style.css", true);
        assert!(link.is_asset());
    }

    #[test]
    fn is_asset_js() {
        let link = make_link("https://example.com/js/main.js", true);
        assert!(link.is_asset());
    }

    #[test]
    fn is_asset_image() {
        let link = make_link("https://example.com/img/photo.png", true);
        assert!(link.is_asset());
    }

    #[test]
    fn is_asset_font() {
        let link = make_link("https://example.com/fonts/roboto.woff2", true);
        assert!(link.is_asset());
    }

    #[test]
    fn is_asset_html_page() {
        let link = make_link("https://example.com/about", true);
        assert!(!link.is_asset());
    }

    #[test]
    fn is_asset_query_after_extension() {
        let link = make_link("https://example.com/bundle.js?v=3", true);
        assert!(link.is_asset());
    }

    #[test]
    fn is_asset_no_extension() {
        let link = make_link("https://example.com/page", true);
        assert!(!link.is_asset());
    }
}
