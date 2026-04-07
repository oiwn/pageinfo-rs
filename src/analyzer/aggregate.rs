use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::analyzer::date_kind::DateKind;
use crate::analyzer::url_facts::UrlFacts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateUrlFacts {
    pub total_urls_seen: usize,
    pub depth_distribution: BTreeMap<usize, usize>,
    pub top_first_segments: Vec<(String, usize)>,
    pub url_samples_by_section: BTreeMap<String, Vec<String>>,
    pub date_positions: Vec<(usize, DateKind)>,
    pub likely_utility_urls: Vec<String>,
}

impl AggregateUrlFacts {
    pub fn from_page_facts(facts: &[&UrlFacts]) -> Self {
        let builder = UrlFacts::merge(facts.to_vec());
        Self {
            total_urls_seen: builder.total_urls_seen(),
            depth_distribution: builder.depth_distribution(),
            top_first_segments: builder.top_first_segments(),
            url_samples_by_section: builder.url_samples_by_section(),
            date_positions: builder.date_positions(),
            likely_utility_urls: builder.utility_urls(),
        }
    }
}
