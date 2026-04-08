use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::analyzer::date_kind::DateKind;
use crate::analyzer::link::Link;

const UTILITY_KEYWORDS: &[&str] = &[
    "about",
    "contact",
    "privacy",
    "terms",
    "login",
    "careers",
    "advertise",
    "newsletter",
    "sitemap",
    "rss",
    "feed",
    "help",
    "faq",
];

const MAX_TOP_SEGMENTS: usize = 20;
const MAX_URL_SAMPLES_PER_SECTION: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlFacts {
    pub total_internal: usize,
    pub total_external: usize,

    pub depth_distribution: BTreeMap<usize, usize>,

    pub top_first_segments: Vec<(String, usize)>,

    pub url_samples_by_section: BTreeMap<String, Vec<String>>,

    pub date_positions: Vec<(usize, DateKind)>,

    pub likely_utility_urls: Vec<String>,
}

impl UrlFacts {
    pub fn from_links(links: &[Link], _page_domain: &str) -> Self {
        let total_internal = links.iter().filter(|l| l.is_internal).count();
        let total_external = links.len() - total_internal;

        let internal: Vec<&Link> = links.iter().filter(|l| l.is_internal).collect();

        let mut depth_distribution: BTreeMap<usize, usize> = BTreeMap::new();
        let mut first_segment_counts: HashMap<String, usize> = HashMap::new();
        let mut url_samples_by_section: BTreeMap<String, HashSet<String>> =
            BTreeMap::new();
        let mut segments_by_depth: BTreeMap<usize, Vec<Vec<String>>> =
            BTreeMap::new();
        let mut utility_urls: HashSet<String> = HashSet::new();

        for link in &internal {
            let Ok(parsed) = url::Url::parse(&link.url) else {
                continue;
            };
            let segments = path_segments(&parsed);
            let depth = segments.len();

            if depth == 0 {
                continue;
            }

            *depth_distribution.entry(depth).or_insert(0) += 1;

            if let Some(first) = segments.first() {
                *first_segment_counts.entry(first.clone()).or_insert(0) += 1;

                let samples =
                    url_samples_by_section.entry(first.clone()).or_default();
                let path = parsed.path().to_string();
                samples.insert(path);
            }

            if is_utility_url(&segments) {
                utility_urls.insert(link.url.clone());
            }

            segments_by_depth.entry(depth).or_default().push(segments);
        }

        let top_first_segments =
            top_by_count(&first_segment_counts, MAX_TOP_SEGMENTS);
        let date_positions = detect_date_positions(&segments_by_depth);

        let url_samples_by_section: BTreeMap<String, Vec<String>> =
            url_samples_by_section
                .into_iter()
                .map(|(k, set)| {
                    let mut v: Vec<String> = set.into_iter().collect();
                    v.sort();
                    v.truncate(MAX_URL_SAMPLES_PER_SECTION);
                    (k, v)
                })
                .collect();
        let mut likely_utility_urls: Vec<String> =
            utility_urls.into_iter().collect();
        likely_utility_urls.sort();

        Self {
            total_internal,
            total_external,
            depth_distribution,
            top_first_segments,
            url_samples_by_section,
            date_positions,
            likely_utility_urls,
        }
    }

    pub fn detected_url_pattern(&self) -> Option<String> {
        if self.date_positions.is_empty() {
            return None;
        }

        let date_pos: HashSet<usize> =
            self.date_positions.iter().map(|(p, _)| *p).collect();

        for (section, samples) in &self.url_samples_by_section {
            for sample in samples {
                let segments: Vec<&str> =
                    sample.split('/').filter(|s| !s.is_empty()).collect();

                let first = match segments.first() {
                    Some(s) => *s,
                    None => continue,
                };

                if first != section.as_str() {
                    continue;
                }

                let mut pattern = String::new();
                for (i, seg) in segments.iter().enumerate() {
                    pattern.push('/');
                    if date_pos.contains(&i) {
                        let kind = self
                            .date_positions
                            .iter()
                            .find(|(p, _)| *p == i)
                            .map(|(_, k)| *k);
                        match kind {
                            Some(DateKind::Year) => pattern.push_str("{year}"),
                            Some(DateKind::Month) => pattern.push_str("{month}"),
                            Some(DateKind::Day) => pattern.push_str("{day}"),
                            None => pattern.push_str(seg),
                        }
                    } else if i == segments.len() - 1
                        && seg.len() > 8
                        && seg.contains('-')
                    {
                        pattern.push_str("{slug}");
                    } else {
                        pattern.push_str(seg);
                    }
                }

                let date_count =
                    date_pos.iter().filter(|p| **p < segments.len()).count();
                if date_count >= 2 && segments.len() > 3 {
                    return Some(pattern);
                }
            }
        }

        None
    }
}

fn path_segments(url: &url::Url) -> Vec<String> {
    url.path_segments()
        .map(|segs| segs.filter(|s| !s.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

fn top_by_count(
    counts: &HashMap<String, usize>,
    limit: usize,
) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> =
        counts.iter().map(|(k, &v)| (k.clone(), v)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(limit);
    v
}

fn detect_date_positions(
    by_depth: &BTreeMap<usize, Vec<Vec<String>>>,
) -> Vec<(usize, DateKind)> {
    let mut numeric_by_position: BTreeMap<usize, Vec<u32>> = BTreeMap::new();

    for paths in by_depth.values() {
        for path in paths {
            for (pos, seg) in path.iter().enumerate() {
                if let Ok(val) = seg.parse::<u32>() {
                    numeric_by_position.entry(pos).or_default().push(val);
                }
            }
        }
    }

    let total_paths: usize = by_depth.values().map(|v| v.len()).max().unwrap_or(0);
    if total_paths == 0 {
        return Vec::new();
    }

    let mut result: Vec<(usize, DateKind)> = Vec::new();

    let year_positions: Vec<usize> = numeric_by_position
        .iter()
        .filter(|(_, values)| {
            values.len() >= total_paths / 3
                && values.iter().all(|v| *v >= 1900 && *v <= 2100)
        })
        .map(|(&pos, _)| pos)
        .collect();

    for &year_pos in &year_positions {
        result.push((year_pos, DateKind::Year));

        if let Some(month_vals) = numeric_by_position.get(&(year_pos + 1))
            && month_vals.len() >= total_paths / 3
            && month_vals.iter().all(|v| *v >= 1 && *v <= 12)
        {
            result.push((year_pos + 1, DateKind::Month));

            if let Some(day_vals) = numeric_by_position.get(&(year_pos + 2))
                && day_vals.len() >= total_paths / 3
                && day_vals.iter().all(|v| *v >= 1 && *v <= 31)
            {
                result.push((year_pos + 2, DateKind::Day));
            }
        }
    }

    result.sort_by_key(|(pos, _)| *pos);
    result.dedup_by(|a, b| a.0 == b.0);
    result
}

fn is_utility_url(segments: &[String]) -> bool {
    segments.iter().any(|seg| {
        let lower = seg.to_lowercase();
        UTILITY_KEYWORDS.iter().any(|kw| lower == *kw)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link(url: &str, is_internal: bool) -> Link {
        Link {
            url: url.to_string(),
            text: None,
            rel: None,
            is_internal,
        }
    }

    #[test]
    fn test_empty_links() {
        let facts = UrlFacts::from_links(&[], "example.com");
        assert_eq!(facts.total_internal, 0);
        assert_eq!(facts.total_external, 0);
        assert!(facts.depth_distribution.is_empty());
    }

    #[test]
    fn test_basic_counts() {
        let links = vec![
            make_link("https://example.com/markets/btc", true),
            make_link("https://example.com/tech/eth", true),
            make_link("https://other.com/page", false),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        assert_eq!(facts.total_internal, 2);
        assert_eq!(facts.total_external, 1);
    }

    #[test]
    fn test_depth_distribution() {
        let links = vec![
            make_link("https://example.com/a", true),
            make_link("https://example.com/a/b", true),
            make_link("https://example.com/a/b/c", true),
            make_link("https://example.com/a/b/c/d", true),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        assert_eq!(facts.depth_distribution.get(&1), Some(&1));
        assert_eq!(facts.depth_distribution.get(&2), Some(&1));
        assert_eq!(facts.depth_distribution.get(&3), Some(&1));
        assert_eq!(facts.depth_distribution.get(&4), Some(&1));
    }

    #[test]
    fn test_top_first_segments() {
        let links = vec![
            make_link("https://example.com/markets/btc", true),
            make_link("https://example.com/markets/eth", true),
            make_link("https://example.com/tech/ai", true),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        assert_eq!(facts.top_first_segments[0], ("markets".to_string(), 2));
        assert_eq!(facts.top_first_segments[1], ("tech".to_string(), 1));
    }

    #[test]
    fn test_url_samples_collected() {
        let links = vec![
            make_link("https://example.com/markets/btc", true),
            make_link("https://example.com/markets/eth", true),
            make_link("https://example.com/tech/ai", true),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        let markets_samples = facts.url_samples_by_section.get("markets").unwrap();
        assert_eq!(markets_samples.len(), 2);
        assert!(
            markets_samples[0].ends_with("btc")
                || markets_samples[0].ends_with("eth")
        );
    }

    #[test]
    fn test_date_detection() {
        let links = vec![
            make_link("https://example.com/markets/2026/04/06/btc", true),
            make_link("https://example.com/tech/2026/04/05/ai", true),
            make_link("https://example.com/policy/2025/12/28/law", true),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        assert!(facts.date_positions.contains(&(1, DateKind::Year)));
        assert!(facts.date_positions.contains(&(2, DateKind::Month)));
        assert!(facts.date_positions.contains(&(3, DateKind::Day)));
    }

    #[test]
    fn test_utility_urls() {
        let links = vec![
            make_link("https://example.com/about", true),
            make_link("https://example.com/privacy", true),
            make_link("https://example.com/markets/btc", true),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        assert_eq!(facts.likely_utility_urls.len(), 2);
    }

    #[test]
    fn test_detected_url_pattern() {
        let links = vec![
            make_link(
                "https://example.com/markets/2026/04/06/some-long-slug-here",
                true,
            ),
            make_link(
                "https://example.com/tech/2026/04/05/another-article-slug",
                true,
            ),
        ];
        let facts = UrlFacts::from_links(&links, "example.com");
        let pattern = facts.detected_url_pattern();
        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert!(p.contains("{year}"));
        assert!(p.contains("{month}"));
        assert!(p.contains("{day}"));
        assert!(p.contains("{slug}"));
    }
}
