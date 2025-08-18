use crate::browser::BrowserClient;
use clap::{Parser, Subcommand};
use dom_content_extraction::{get_content, scraper::Html};
use spider_fingerprint::url;
use std::error::Error;
use std::path::PathBuf;

mod browser;
mod html;
mod http;
mod httpreq;
mod stand;

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

    /// Load page using headless browser (spider_chrome)
    Browse {
        /// URL to load
        #[arg(short, long)]
        url: String,
        /// Enable network traffic analysis
        #[arg(long)]
        network_analysis: bool,
    },
    /// Test bot detection with realistic browser fingerprinting
    BotTest {
        /// Output filename for screenshot (default: bot_test.png)
        #[arg(short, long, default_value = "bot_test.png")]
        output: String,

        /// Use headless mode
        #[arg(long)]
        headless: bool,

        /// Chrome endpoint URL (default: http://localhost:9222)
        #[arg(long, default_value = "http://localhost:9222")]
        chrome_url: String,
    },
    /// Execute HTTP request from TOML config file
    HttpReq {
        /// Path to TOML config file
        #[arg(short, long)]
        config_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Http { url } => {
            let url = url::Url::parse(url)?;
            match http::retrieve_page(&url).await {
                Ok(transaction) => {
                    println!("{}", transaction.format_for_llm());

                    // Parse the HTML and extract page info
                    let html = Html::parse_document(&transaction.response.body);
                    let page_info = html::PageInfo::new(&html);

                    println!("\n=== PAGE INFO ===");
                    println!("{}", page_info);
                    println!("================");

                    // You can also access individual components
                    if transaction.response.status == 200 {
                        println!("Success!");
                    }
                }
                Err(e) => {
                    eprintln!("Request failed: {}", e);
                }
            }
        }
        Commands::Browse { url, network_analysis } => {
            let browser_client = BrowserClient::new().await.unwrap();
            
            if *network_analysis {
                // Use network analysis mode
                let (html_text, network_analysis) = browser_client.load_url_with_network_analysis(url).await.unwrap();
                
                // Extract and display content
                let document = Html::parse_document(&html_text);
                let content = get_content(&document).unwrap();
                println!("Content:\n{}", content);
                
                // Display network analysis
                println!("\n=== NETWORK ANALYSIS ===");
                println!("Total Requests: {}", network_analysis.total_requests);
                println!("Load Time: {}ms", network_analysis.load_time);
                println!("Total Size: {:.2}KB", network_analysis.total_size as f64 / 1024.0);
                
                println!("\nRequest Types:");
                for (req_type, count) in &network_analysis.requests_by_type {
                    println!("  {}: {}", req_type, count);
                }
                
                if !network_analysis.failed_requests.is_empty() {
                    println!("\nFailed Requests:");
                    for failure in &network_analysis.failed_requests {
                        println!("  - {}", failure);
                    }
                }
                println!("=========================");
            } else {
                // Use standard mode (current behavior)
                let html_text = browser_client.load_url(url).await.unwrap();
                let document = Html::parse_document(&html_text);
                let content = get_content(&document).unwrap();
                println!("Content:\n{}", content);
            }
        }
        Commands::BotTest {
            output,
            headless,
            chrome_url,
        } => {
            stand::handle_bot_test(output, *headless, chrome_url).await?;
        }
        Commands::HttpReq { config_file } => {
            match httpreq::run_from_file(config_file).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("HTTP request failed: {}", e);
                }
            }
        }
    };

    Ok(())
}
