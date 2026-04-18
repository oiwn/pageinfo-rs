use dom_content_extraction::scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub text: Option<String>,
    pub rel: Option<String>,
    pub is_internal: bool,
}

/// Extract all links from an HTML document, resolving relative URLs against
/// the page's base URL.
pub fn extract_links(document: &Html, base_url: &Url) -> Vec<Link> {
    let page_domain = extract_registered_domain(base_url);
    let selector = Selector::parse("a[href]").unwrap();

    document
        .select(&selector)
        .filter_map(|element| {
            let href = element.value().attr("href")?;
            let resolved = base_url.join(href).ok()?;
            let resolved_domain = extract_registered_domain(&resolved);
            let is_internal = match (&page_domain, &resolved_domain) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };

            let text = element
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            let text = if text.is_empty() { None } else { Some(text) };

            Some(Link {
                url: resolved.to_string(),
                text,
                rel: element.value().attr("rel").map(String::from),
                is_internal,
            })
        })
        .collect()
}

/// Extract the registered domain (e.g. "coindesk.com" from "www.coindesk.com").
/// Handles common suffixes like .com, .co.uk, etc. by taking last two or three
/// labels depending on the TLD.
pub fn extract_registered_domain(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return Some(host.to_string());
    }

    let suffix_len = if is_multi_part_tld(&parts) { 3 } else { 2 };

    if parts.len() < suffix_len {
        return Some(host.to_string());
    }

    let start = parts.len() - suffix_len;
    Some(parts[start..].join("."))
}

fn is_multi_part_tld(parts: &[&str]) -> bool {
    let second_level = match parts.last() {
        Some(&s) => s,
        None => return false,
    };

    matches!(
        second_level,
        "uk" | "au" | "br" | "ca" | "cn" | "de" | "fr" | "in" | "jp" | "nz" | "za"
    ) && parts.len() >= 3
        && matches!(
            parts.get(parts.len() - 2).copied(),
            Some("co" | "com" | "org" | "net" | "gov" | "ac" | "edu")
        )
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(links[0].url, "https://example.com/about");
        assert_eq!(links[1].url, "https://example.com/contact");
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
}
