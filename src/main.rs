use crate::browser::BrowserClient;
use clap::{Parser, Subcommand};
use dom_content_extraction::{get_content, scraper::Html};
use spider_fingerprint::url;
use std::error::Error;

mod browser;
mod html;
mod http;
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
        Commands::Browse { url } => {
            let browser_client = BrowserClient::new().await.unwrap();
            let html_text = browser_client.load_url(url).await.unwrap();
            let document = Html::parse_document(&html_text);
            let content = get_content(&document).unwrap();
            println!("Content:\n{}", content);
        }
        Commands::BotTest {
            output,
            headless,
            chrome_url,
        } => {
            stand::handle_bot_test(output, *headless, chrome_url).await?;
        }
    };

    Ok(())
}
