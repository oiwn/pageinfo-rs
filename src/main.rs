use clap::{Args, Parser, Subcommand};
use std::error::Error;
mod analyzer;
mod cache;
mod help;
mod html;
mod http;

use crate::cache::Cache;

/// CLI tool to research web pages
#[derive(Parser, Debug)]
#[command(name = "Pageinfo")]
#[command(author = "oiwn <https://github.org/oiwn>")]
#[command(version = "0.1")]
#[command(about = "CLI tool to research web pages", long_about = None)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show built-in help for humans and LLM tools
    Help {
        /// Optional help topic: analyze, http, tool
        topic: Option<String>,
    },
    /// Load page using reqwest
    Http {
        /// URL to load
        #[arg(short, long)]
        url: String,
    },
    /// Analyze a single page and inspect specific evidence views
    Analyze(AnalyzeArgs),
}

#[derive(Args, Debug)]
struct AnalyzeArgs {
    #[command(subcommand)]
    command: Option<AnalyzeCommand>,
    /// URL to analyze
    #[arg(short, long)]
    url: String,
    /// Ignore cache and do not write fetched page to cache
    #[arg(long, conflicts_with = "refresh")]
    no_cache: bool,
    /// Refetch page and overwrite existing cache entry
    #[arg(long)]
    refresh: bool,
}

#[derive(Subcommand, Debug)]
enum AnalyzeCommand {
    /// Show link grouping and URL structure
    Links,
    /// Show curated metadata only
    Meta,
    /// Show structured-data / embedded JSON summary
    Json,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Help { topic } => {
            println!("{}", help::render(topic.as_deref()));
        }
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
        Commands::Analyze(args) => {
            let cache_config = cache::CacheConfig {
                enabled: !args.no_cache,
                refresh: args.refresh,
                ..cache::CacheConfig::default()
            };
            let cache = cache::FileCache::new(cache_config);
            cache.init()?;
            let client = wreq::Client::new();
            let cache_key = cache.key_for_final_url(&args.url)?;
            let cached_page = if args.no_cache || cache.should_refresh() {
                None
            } else {
                cache.load(&cache_key)?
            };
            let page = match cached_page {
                Some(cached) => analyzer::PageInfo::from_cached_page(cached),
                None => {
                    let cached =
                        analyzer::PageInfo::fetch_raw(&args.url, &client).await?;
                    if !args.no_cache {
                        cache.store(cached.clone())?;
                    }
                    analyzer::PageInfo::from_cached_page(cached)
                }
            };
            match page {
                Ok(page) => {
                    let output = match args.command {
                        Some(AnalyzeCommand::Links) => page.format_links_for_llm(),
                        Some(AnalyzeCommand::Meta) => page.format_meta_for_llm(),
                        Some(AnalyzeCommand::Json) => page.format_json_for_llm(),
                        None => page.format_for_llm(),
                    };
                    println!("{output}");
                }
                Err(e) => {
                    eprintln!("Analysis failed: {}", e);
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
    fn help_accepts_topic() {
        let cli = Cli::try_parse_from(["pginf", "help", "tool"]).unwrap();

        match cli.command {
            Commands::Help { topic } => {
                assert_eq!(topic.as_deref(), Some("tool"));
            }
            _ => panic!("expected help command"),
        }
    }

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
            Commands::Analyze(AnalyzeArgs {
                no_cache, refresh, ..
            }) => {
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
            Commands::Analyze(AnalyzeArgs {
                no_cache, refresh, ..
            }) => {
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

    #[test]
    fn analyze_accepts_links_subcommand() {
        let cli = Cli::try_parse_from([
            "pginf",
            "analyze",
            "-u",
            "https://example.com",
            "links",
        ])
        .unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeArgs { command, .. }) => {
                assert!(matches!(command, Some(AnalyzeCommand::Links)));
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn help_tool_mentions_analyze_as_first_step() {
        let text = help::render(Some("tool"));
        assert!(text.contains("Run `pginf analyze -u <URL>` first."));
    }
}
