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
