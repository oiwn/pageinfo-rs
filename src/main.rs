use clap::{Parser, Subcommand};
use std::error::Error;
mod analyzer;
mod cache;
mod client;
mod help;
mod html;
mod http_display;
mod resolve;
mod skills;

/// CLI tool to research web pages
#[derive(Parser, Debug)]
#[command(name = "pginf")]
#[command(author = "oiwn <https://github.org/oiwn>")]
#[command(version = "0.2")]
#[command(about = "CLI tool to research web pages", long_about = None)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Proxy URL (e.g. socks5://user:pass@host:port)
    #[arg(long, global = true)]
    proxy: Option<String>,
    /// Browser emulation name (e.g. chrome137, firefox, safari)
    #[arg(long, global = true)]
    browser: Option<String>,
    /// Request timeout in seconds
    #[arg(long, global = true)]
    timeout: Option<u64>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show built-in help for humans and LLM tools
    Help {
        /// Optional help topic: tool
        topic: Option<String>,
    },
    /// Fetch page, cache it, print HTTP metadata
    Fetch {
        /// URL to fetch
        url: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Show link grouping and URL structure
    Links {
        /// URL to analyze
        url: String,
        /// Show only internal (inbound) links
        #[arg(long)]
        inbound: bool,
        /// Show only external (outbound) links
        #[arg(long)]
        outbound: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Show curated metadata
    Meta {
        /// URL to analyze
        url: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Show structured data (JSON-LD, Next.js, inline JSON)
    Json {
        /// URL to analyze
        url: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Extract text content from page
    Text {
        /// URL to analyze
        url: String,
        /// Output format: text (default) or markdown
        #[arg(long, default_value = "text")]
        format: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Show raw HTTP transaction (request/response debug)
    Http {
        /// URL to load
        #[arg(short, long)]
        url: String,
    },
    /// Show HTML content, optionally filtered by CSS selector
    Html {
        /// URL to fetch
        #[arg(short, long)]
        url: String,
        /// CSS selector to filter elements (e.g. "div.article", "h1, h2", "meta[property]")
        #[arg(short, long)]
        selector: Option<String>,
        /// Ignore cache and do not write fetched page to cache
        #[arg(long, conflicts_with = "refresh")]
        no_cache: bool,
        /// Refetch page and overwrite existing cache entry
        #[arg(long)]
        refresh: bool,
    },
    /// Install pginf skill files for AI coding agents
    Install {
        #[command(subcommand)]
        command: InstallCommand,
    },
}

#[derive(Subcommand, Debug)]
enum InstallCommand {
    /// Install skill files
    Skills {
        #[command(subcommand)]
        target: SkillsTarget,
    },
}

#[derive(Subcommand, Debug)]
enum SkillsTarget {
    /// Install into <project>/.agents/skills/pginf/
    Local,
    /// Install into ~/.agents/skills/pginf/
    Global,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut page_client = client::PageClient::builder();
    if let Some(ref proxy) = cli.proxy {
        page_client = page_client.proxy(proxy)?;
    } else {
        page_client = page_client.proxy_from_env();
    }
    if let Some(ref browser) = cli.browser {
        page_client = page_client.browser(client::parse_browser(browser)?);
    }
    if let Some(secs) = cli.timeout {
        page_client = page_client.timeout(std::time::Duration::from_secs(secs));
    }
    let page_client = page_client.build();

    match &cli.command {
        Commands::Help { topic } => {
            println!("{}", help::render(topic.as_deref()));
        }
        Commands::Fetch {
            url,
            json,
            no_cache,
            refresh,
        } => {
            let resolved =
                resolve::resolve_page(url, &page_client, *no_cache, *refresh)
                    .await?;
            if *json {
                println!("{}", format_fetch_json(&resolved));
            } else {
                println!("{}", format_fetch_markdown(&resolved));
            }
        }
        Commands::Links {
            url,
            inbound,
            outbound,
            json,
            no_cache,
            refresh,
        } => {
            let resolved =
                resolve::resolve_page(url, &page_client, *no_cache, *refresh)
                    .await?;
            let page =
                analyzer::PageInfo::from_fetch_result(&resolved.fetch_result)?;
            if *json {
                println!("{}", page.links_json(*inbound, *outbound));
            } else {
                println!("{}", page.format_links_for_llm());
            }
        }
        Commands::Meta {
            url,
            json,
            no_cache,
            refresh,
        } => {
            let resolved =
                resolve::resolve_page(url, &page_client, *no_cache, *refresh)
                    .await?;
            let page =
                analyzer::PageInfo::from_fetch_result(&resolved.fetch_result)?;
            if *json {
                println!("{}", page.meta_json());
            } else {
                println!("{}", page.format_meta_for_llm());
            }
        }
        Commands::Json {
            url,
            json,
            no_cache,
            refresh,
        } => {
            let resolved =
                resolve::resolve_page(url, &page_client, *no_cache, *refresh)
                    .await?;
            let page =
                analyzer::PageInfo::from_fetch_result(&resolved.fetch_result)?;
            if *json {
                println!("{}", page.json_data_json());
            } else {
                println!("{}", page.format_json_for_llm());
            }
        }
        Commands::Text {
            url,
            format,
            json,
            no_cache,
            refresh,
        } => {
            let resolved =
                resolve::resolve_page(url, &page_client, *no_cache, *refresh)
                    .await?;
            let page =
                analyzer::PageInfo::from_fetch_result(&resolved.fetch_result)?;
            let as_markdown = format == "markdown";
            if *json {
                println!("{}", page.text_json(as_markdown));
            } else {
                println!("{}", page.format_text(as_markdown));
            }
        }
        Commands::Http { url } => {
            let parsed = url::Url::parse(url)?;
            match http_display::retrieve_page(&parsed, &page_client).await {
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
        Commands::Html {
            url,
            selector,
            no_cache,
            refresh,
        } => {
            let resolved =
                resolve::resolve_page(url, &page_client, *no_cache, *refresh)
                    .await?;
            match selector {
                None => {
                    println!("{}", resolved.fetch_result.body);
                }
                Some(sel) => {
                    let css = dom_content_extraction::scraper::Selector::parse(sel)
                        .map_err(|e| {
                            format!("Invalid CSS selector '{}': {e}", sel)
                        })?;
                    let document =
                        dom_content_extraction::scraper::Html::parse_document(
                            &resolved.fetch_result.body,
                        );
                    let matches: Vec<_> = document.select(&css).collect();
                    if matches.is_empty() {
                        eprintln!("No elements matching '{}'", sel);
                    } else {
                        println!(
                            "{} element(s) matching '{}':\n",
                            matches.len(),
                            sel
                        );
                        for (i, el) in matches.iter().enumerate() {
                            if matches.len() > 1 {
                                println!("--- Element {} ---", i + 1);
                            }
                            println!("{}", el.html());
                        }
                    }
                }
            }
        }
        Commands::Install { command } => match command {
            InstallCommand::Skills { target } => match target {
                SkillsTarget::Local => match skills::install_local() {
                    Ok(msg) => println!("{msg}"),
                    Err(e) => eprintln!("{e}"),
                },
                SkillsTarget::Global => match skills::install_global() {
                    Ok(msg) => println!("{msg}"),
                    Err(e) => eprintln!("{e}"),
                },
            },
        },
    };

    Ok(())
}

fn format_fetch_markdown(resolved: &resolve::ResolveOutput) -> String {
    let r = &resolved.fetch_result;
    let mut out = String::new();
    out.push_str("## Fetch Result\n\n");
    out.push_str(&format!("- **Input URL:** {}\n", r.input_url));
    out.push_str(&format!("- **Final URL:** {}\n", r.final_url));
    out.push_str(&format!("- **Status:** {}\n", r.status));
    out.push_str(&format!("- **Duration:** {}ms\n", r.duration_ms));
    out.push_str(&format!(
        "- **Cached:** {}\n",
        if resolved.from_cache { "yes" } else { "no" }
    ));
    out.push_str(&format!("- **Body size:** {} bytes\n", r.body.len()));
    if !r.headers.is_empty() {
        out.push_str("\n### Response Headers\n\n");
        for (k, v) in &r.headers {
            out.push_str(&format!("- `{}`: {}\n", k, v));
        }
    }
    out
}

fn format_fetch_json(resolved: &resolve::ResolveOutput) -> String {
    let r = &resolved.fetch_result;
    let obj = serde_json::json!({
        "input_url": r.input_url,
        "final_url": r.final_url,
        "status": r.status,
        "duration_ms": r.duration_ms,
        "cached": resolved.from_cache,
        "body_size": r.body.len(),
        "headers": r.headers,
    });
    serde_json::to_string_pretty(&obj).unwrap_or_default()
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
    fn fetch_parses_url() {
        let cli =
            Cli::try_parse_from(["pginf", "fetch", "https://example.com"]).unwrap();
        match cli.command {
            Commands::Fetch {
                url,
                json,
                no_cache,
                ..
            } => {
                assert_eq!(url, "https://example.com");
                assert!(!json);
                assert!(!no_cache);
            }
            _ => panic!("expected fetch command"),
        }
    }

    #[test]
    fn fetch_accepts_json_flag() {
        let cli = Cli::try_parse_from([
            "pginf",
            "fetch",
            "https://example.com",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Fetch { json, .. } => assert!(json),
            _ => panic!("expected fetch command"),
        }
    }

    #[test]
    fn fetch_accepts_no_cache() {
        let cli = Cli::try_parse_from([
            "pginf",
            "fetch",
            "https://example.com",
            "--no-cache",
        ])
        .unwrap();
        match cli.command {
            Commands::Fetch { no_cache, .. } => assert!(no_cache),
            _ => panic!("expected fetch command"),
        }
    }

    #[test]
    fn fetch_rejects_no_cache_with_refresh() {
        let err = Cli::try_parse_from([
            "pginf",
            "fetch",
            "https://example.com",
            "--no-cache",
            "--refresh",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn links_parses_url() {
        let cli =
            Cli::try_parse_from(["pginf", "links", "https://example.com"]).unwrap();
        match cli.command {
            Commands::Links {
                url,
                inbound,
                outbound,
                json,
                ..
            } => {
                assert_eq!(url, "https://example.com");
                assert!(!inbound);
                assert!(!outbound);
                assert!(!json);
            }
            _ => panic!("expected links command"),
        }
    }

    #[test]
    fn links_accepts_inbound() {
        let cli = Cli::try_parse_from([
            "pginf",
            "links",
            "https://example.com",
            "--inbound",
        ])
        .unwrap();
        match cli.command {
            Commands::Links { inbound, .. } => assert!(inbound),
            _ => panic!("expected links command"),
        }
    }

    #[test]
    fn links_accepts_outbound() {
        let cli = Cli::try_parse_from([
            "pginf",
            "links",
            "https://example.com",
            "--outbound",
        ])
        .unwrap();
        match cli.command {
            Commands::Links { outbound, .. } => assert!(outbound),
            _ => panic!("expected links command"),
        }
    }

    #[test]
    fn meta_parses_url() {
        let cli =
            Cli::try_parse_from(["pginf", "meta", "https://example.com"]).unwrap();
        match cli.command {
            Commands::Meta { url, json, .. } => {
                assert_eq!(url, "https://example.com");
                assert!(!json);
            }
            _ => panic!("expected meta command"),
        }
    }

    #[test]
    fn json_cmd_parses_url() {
        let cli =
            Cli::try_parse_from(["pginf", "json", "https://example.com"]).unwrap();
        match cli.command {
            Commands::Json { url, json, .. } => {
                assert_eq!(url, "https://example.com");
                assert!(!json);
            }
            _ => panic!("expected json command"),
        }
    }

    #[test]
    fn text_parses_url() {
        let cli =
            Cli::try_parse_from(["pginf", "text", "https://example.com"]).unwrap();
        match cli.command {
            Commands::Text {
                url, format, json, ..
            } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(format, "text");
                assert!(!json);
            }
            _ => panic!("expected text command"),
        }
    }

    #[test]
    fn text_accepts_markdown_format() {
        let cli = Cli::try_parse_from([
            "pginf",
            "text",
            "https://example.com",
            "--format",
            "markdown",
        ])
        .unwrap();
        match cli.command {
            Commands::Text { format, .. } => assert_eq!(format, "markdown"),
            _ => panic!("expected text command"),
        }
    }

    #[test]
    fn html_parses_with_url_only() {
        let cli =
            Cli::try_parse_from(["pginf", "html", "-u", "https://example.com"])
                .unwrap();
        match cli.command {
            Commands::Html {
                url,
                selector,
                no_cache,
                refresh,
            } => {
                assert_eq!(url, "https://example.com");
                assert!(selector.is_none());
                assert!(!no_cache);
                assert!(!refresh);
            }
            _ => panic!("expected html command"),
        }
    }

    #[test]
    fn html_parses_with_selector() {
        let cli = Cli::try_parse_from([
            "pginf",
            "html",
            "-u",
            "https://example.com",
            "-s",
            "div.article",
        ])
        .unwrap();
        match cli.command {
            Commands::Html { selector, .. } => {
                assert_eq!(selector.as_deref(), Some("div.article"));
            }
            _ => panic!("expected html command"),
        }
    }

    #[test]
    fn install_skills_local_parses() {
        let cli =
            Cli::try_parse_from(["pginf", "install", "skills", "local"]).unwrap();
        match cli.command {
            Commands::Install {
                command: InstallCommand::Skills { target },
            } => {
                assert!(matches!(target, SkillsTarget::Local));
            }
            _ => panic!("expected install skills local"),
        }
    }

    #[test]
    fn install_skills_global_parses() {
        let cli =
            Cli::try_parse_from(["pginf", "install", "skills", "global"]).unwrap();
        match cli.command {
            Commands::Install {
                command: InstallCommand::Skills { target },
            } => {
                assert!(matches!(target, SkillsTarget::Global));
            }
            _ => panic!("expected install skills global"),
        }
    }

    #[test]
    fn help_tool_mentions_fetch_as_first_step() {
        let text = help::render(Some("tool"));
        assert!(text.contains("pginf fetch"));
    }

    #[test]
    fn format_fetch_markdown_contains_status() {
        let resolved = resolve::ResolveOutput {
            fetch_result: client::FetchResult {
                input_url: "https://example.com".to_string(),
                final_url: "https://example.com".to_string(),
                status: 200,
                headers: std::collections::HashMap::new(),
                body: "<html></html>".to_string(),
                duration_ms: 42,
            },
            from_cache: false,
        };
        let out = format_fetch_markdown(&resolved);
        assert!(out.contains("200"));
        assert!(out.contains("42ms"));
        assert!(out.contains("example.com"));
    }

    #[test]
    fn format_fetch_json_valid() {
        let resolved = resolve::ResolveOutput {
            fetch_result: client::FetchResult {
                input_url: "https://example.com".to_string(),
                final_url: "https://example.com".to_string(),
                status: 200,
                headers: std::collections::HashMap::new(),
                body: "<html></html>".to_string(),
                duration_ms: 42,
            },
            from_cache: false,
        };
        let out = format_fetch_json(&resolved);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["status"], 200);
        assert_eq!(parsed["duration_ms"], 42);
    }
}
