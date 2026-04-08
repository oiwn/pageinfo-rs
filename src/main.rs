use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

mod analyzer;
mod cache;
mod html;
mod http;

use crate::cache::Cache;

/// CLI tool to research web pages
#[derive(Parser, Debug)]
#[command(name = "Pageinfo")]
#[command(author = "oiwn <https://github.org/oiwn>")]
#[command(version = "0.1")]
#[command(about = "CLI tool to research web pages", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Load page using reqwest
    Http {
        /// URL to load
        #[arg(short, long)]
        url: String,
    },
    /// Analyze a single page (link patterns, URL taxonomy, metadata)
    Analyze {
        /// URL to analyze
        #[arg(short, long)]
        url: String,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Sample multiple pages from a site and build aggregate statistics
    Sample {
        /// Seed URL to start from
        #[arg(short, long)]
        url: String,
        /// Maximum number of pages to sample
        #[arg(short, long, default_value = "5")]
        max_pages: usize,
        /// Concurrency limit for fetching
        #[arg(short, long, default_value = "3")]
        concurrency: usize,
        /// Output directory for stored artifacts
        #[arg(short, long, default_value = "/tmp/site-analyzer")]
        output_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Http { url } => {
            let parsed = url::Url::parse(url)?;
            match http::retrieve_page(&parsed).await {
                Ok(transaction) => {
                    println!("{}", transaction.format_for_llm());

                    let document =
                        dom_content_extraction::scraper::Html::parse_document(
                            &transaction.response.body,
                        );
                    let page_info = html::PageInfo::new(&document);

                    println!("\n=== PAGE INFO ===");
                    println!("{}", page_info);
                    println!("================");
                }
                Err(e) => {
                    eprintln!("Request failed: {}", e);
                }
            }
        }
        Commands::Analyze {
            url,
            no_cache,
            refresh,
        } => {
            let cache_config = cache::CacheConfig {
                enabled: !no_cache,
                refresh: *refresh,
                ..cache::CacheConfig::default()
            };
            let cache = cache::FileCache::new(cache_config);
            cache.init()?;
            let client = wreq::Client::new();
            let cache_key = cache.key_for_final_url(url)?;
            let cached_page = if *no_cache || cache.should_refresh() {
                None
            } else {
                cache.load(&cache_key)?
            };
            let page = match cached_page {
                Some(cached) => analyzer::PageInfo::from_cached_page(cached),
                None => {
                    let cached =
                        analyzer::PageInfo::fetch_raw(url, &client).await?;
                    if !*no_cache {
                        cache.store(cached.clone())?;
                    }
                    analyzer::PageInfo::from_cached_page(cached)
                }
            };
            match page {
                Ok(page) => {
                    println!("{}", page.format_for_llm());
                }
                Err(e) => {
                    eprintln!("Analysis failed: {}", e);
                }
            }
        }
        Commands::Sample {
            url,
            max_pages,
            concurrency,
            output_dir,
        } => {
            let client = wreq::Client::new();
            let options = analyzer::SampleOptions {
                max_pages: *max_pages,
                concurrency: *concurrency,
            };
            match analyzer::SampleCollector::collect(
                url, options, &client, output_dir,
            )
            .await
            {
                Ok(collector) => {
                    println!("{}", collector.format_for_llm());
                }
                Err(e) => {
                    eprintln!("Sampling failed: {}", e);
                }
            }
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn analyze_accepts_no_cache_flag() {
        let cli = Cli::try_parse_from([
            "pginf",
            "analyze",
            "-u",
            "https://example.com",
            "--no-cache",
        ])
        .unwrap();

        match cli.command {
            Commands::Analyze {
                no_cache, refresh, ..
            } => {
                assert!(no_cache);
                assert!(!refresh);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn analyze_accepts_refresh_flag() {
        let cli = Cli::try_parse_from([
            "pginf",
            "analyze",
            "-u",
            "https://example.com",
            "--refresh",
        ])
        .unwrap();

        match cli.command {
            Commands::Analyze {
                no_cache, refresh, ..
            } => {
                assert!(!no_cache);
                assert!(refresh);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn analyze_rejects_no_cache_with_refresh() {
        let err = Cli::try_parse_from([
            "pginf",
            "analyze",
            "-u",
            "https://example.com",
            "--no-cache",
            "--refresh",
        ])
        .unwrap_err();

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }
}
