use std::sync::Arc;

use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

use crate::analyzer::aggregate::AggregateUrlFacts;
use crate::analyzer::error::AnalyzerError;
use crate::analyzer::page_info::PageInfo;
use crate::analyzer::sample_options::SampleOptions;
use crate::analyzer::url_facts::UrlFacts;

#[derive(Debug)]
pub struct SampleCollector {
    pub seed_url: String,
    pub domain: String,
    pub pages: Vec<PageInfo>,
    pub aggregate: AggregateUrlFacts,
}

impl SampleCollector {
    pub async fn collect(
        seed_url: &str,
        options: SampleOptions,
        client: &wreq::Client,
        output_dir: &std::path::Path,
    ) -> Result<Self, AnalyzerError> {
        let seed = PageInfo::fetch(seed_url, client).await?;
        let domain = seed.domain.clone();

        let internal_links: Vec<String> = seed
            .links
            .iter()
            .filter(|l| l.is_internal)
            .map(|l| l.url.clone())
            .collect();

        let to_sample = select_sample_urls(&internal_links, options.max_pages);

        let semaphore = Arc::new(Semaphore::new(options.concurrency));
        let mut handles = Vec::new();

        for url in &to_sample {
            let url = url.clone();
            let client = client.clone();
            let permit = semaphore.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                PageInfo::fetch(&url, &client).await
            }));
        }

        let mut pages = vec![seed];
        for handle in handles {
            match handle.await {
                Ok(Ok(page)) => pages.push(page),
                Ok(Err(e)) => eprintln!("warning: skipping page: {}", e),
                Err(e) => eprintln!("warning: task panicked: {}", e),
            }
        }

        let facts_refs: Vec<&UrlFacts> =
            pages.iter().map(|p| &p.url_facts).collect();
        let aggregate = AggregateUrlFacts::from_page_facts(&facts_refs);

        let collector = SampleCollector {
            seed_url: seed_url.to_string(),
            domain: domain.clone(),
            pages,
            aggregate: aggregate.clone(),
        };

        persist_artifacts(&collector, output_dir).await?;

        Ok(collector)
    }

    pub fn format_for_llm(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("# Site Analysis: {}\n\n", self.domain));

        let mut header = Table::new();
        header.set_content_arrangement(ContentArrangement::Dynamic);
        header.load_preset(UTF8_FULL_CONDENSED);
        header.add_row(vec![
            Cell::new("Seed URL").add_attribute(Attribute::Bold),
            Cell::new(&self.seed_url),
        ]);
        header.add_row(vec![
            Cell::new("Pages analyzed").add_attribute(Attribute::Bold),
            Cell::new(self.pages.len()),
        ]);
        header.add_row(vec![
            Cell::new("Total URLs seen").add_attribute(Attribute::Bold),
            Cell::new(self.aggregate.total_urls_seen),
        ]);
        out.push_str(&header.to_string());
        out.push('\n');

        if !self.aggregate.top_first_segments.is_empty() {
            out.push_str("\n## URL Patterns\n");

            let merged_facts =
                UrlFacts::merge(self.pages.iter().map(|p| &p.url_facts).collect());
            let pattern = merged_facts.date_positions();
            if !pattern.is_empty() {
                let fake_facts = UrlFacts {
                    total_internal: 0,
                    total_external: 0,
                    depth_distribution: self.aggregate.depth_distribution.clone(),
                    top_first_segments: self.aggregate.top_first_segments.clone(),
                    url_samples_by_section: self
                        .aggregate
                        .url_samples_by_section
                        .clone(),
                    date_positions: pattern,
                    likely_utility_urls: vec![],
                };
                if let Some(ref p) = fake_facts.detected_url_pattern() {
                    out.push_str(&format!("Detected article pattern: {}\n", p));
                }
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

            for (segment, count) in &self.aggregate.top_first_segments {
                let samples = self
                    .aggregate
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

        if !self.aggregate.depth_distribution.is_empty() {
            out.push_str("\n## Path Depth\n");
            let mut depth_table = Table::new();
            depth_table.set_content_arrangement(ContentArrangement::Dynamic);
            depth_table.load_preset(UTF8_FULL_CONDENSED);
            depth_table.set_header(vec![
                Cell::new("Depth").add_attribute(Attribute::Bold),
                Cell::new("Count").add_attribute(Attribute::Bold),
            ]);
            for (depth, count) in &self.aggregate.depth_distribution {
                depth_table.add_row(vec![Cell::new(depth), Cell::new(count)]);
            }
            out.push_str(&depth_table.to_string());
            out.push('\n');
        }

        if !self.aggregate.likely_utility_urls.is_empty() {
            out.push_str("\n## Utility URLs\n");
            let mut util_table = Table::new();
            util_table.set_content_arrangement(ContentArrangement::Dynamic);
            util_table.load_preset(UTF8_FULL_CONDENSED);
            for url in &self.aggregate.likely_utility_urls {
                util_table.add_row(vec![Cell::new(url)]);
            }
            out.push_str(&util_table.to_string());
            out.push('\n');
        }

        for page in &self.pages {
            out.push_str("\n---\n\n");
            out.push_str(&page.format_for_llm());
        }

        out
    }
}

fn select_sample_urls(internal_links: &[String], max_pages: usize) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    let mut seen = HashSet::new();
    let mut by_segment: HashMap<String, Vec<String>> = HashMap::new();

    for url in internal_links {
        if seen.contains(url) {
            continue;
        }
        seen.insert(url.clone());

        let Ok(parsed) = url::Url::parse(url) else {
            continue;
        };
        let segments: Vec<&str> = parsed
            .path_segments()
            .map(|s| s.filter(|seg| !seg.is_empty()).collect())
            .unwrap_or_default();

        let key = segments.first().unwrap_or(&"_root").to_string();
        by_segment.entry(key).or_default().push(url.clone());
    }

    let mut selected = Vec::new();

    let mut groups: Vec<(String, Vec<String>)> = by_segment.into_iter().collect();
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (_, mut urls) in groups {
        if selected.len() >= max_pages {
            break;
        }
        urls.sort_by(|a, b| {
            let a_has_date = has_date_segments(a);
            let b_has_date = has_date_segments(b);
            b_has_date.cmp(&a_has_date)
        });
        if let Some(url) = urls.into_iter().next() {
            selected.push(url);
        }
    }

    selected.truncate(max_pages);
    selected
}

fn has_date_segments(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    parsed
        .path_segments()
        .map(|segs| {
            segs.filter(|s| !s.is_empty())
                .any(|s| crate::analyzer::date_kind::classify_segment(s).is_some())
        })
        .unwrap_or(false)
}

fn url_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result)
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

async fn persist_artifacts(
    collector: &SampleCollector,
    output_dir: &std::path::Path,
) -> Result<(), AnalyzerError> {
    let domain_dir = output_dir.join(&collector.domain);
    let pages_dir = domain_dir.join("pages");
    tokio::fs::create_dir_all(&pages_dir)
        .await
        .map_err(AnalyzerError::Io)?;

    for page in &collector.pages {
        let hash = url_hash(&page.final_url);

        let html_path = pages_dir.join(format!("{}.html", hash));
        tokio::fs::write(&html_path, &page.raw_html)
            .await
            .map_err(AnalyzerError::Io)?;

        let json_path = pages_dir.join(format!("{}.json", hash));
        let facts_json =
            serde_json::to_string_pretty(&page.url_facts).map_err(|e| {
                AnalyzerError::Parse {
                    url: page.url.clone(),
                    reason: e.to_string(),
                }
            })?;
        tokio::fs::write(&json_path, facts_json)
            .await
            .map_err(AnalyzerError::Io)?;
    }

    let agg_path = domain_dir.join("aggregate.json");
    let agg_json =
        serde_json::to_string_pretty(&collector.aggregate).map_err(|e| {
            AnalyzerError::Parse {
                url: collector.seed_url.clone(),
                reason: e.to_string(),
            }
        })?;
    tokio::fs::write(&agg_path, agg_json)
        .await
        .map_err(AnalyzerError::Io)?;

    let report_path = domain_dir.join("report.md");
    let report = collector.format_for_llm();
    tokio::fs::write(&report_path, report)
        .await
        .map_err(AnalyzerError::Io)?;

    Ok(())
}
