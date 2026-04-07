use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

mod analyzer;
mod html;
mod http;

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
        Commands::Analyze { url } => {
            let client = wreq::Client::new();
            match analyzer::PageInfo::fetch(url, &client).await {
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
