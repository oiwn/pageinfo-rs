use crate::browser::BrowserClient;
use chromiumoxide::Browser;
use clap::{Parser, Subcommand};
use dom_content_extraction::{get_content, scraper::Html};
use html::PageInfo;
use reqwest::Url;
use std::error::Error;

mod browser;
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
    Fetch {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Fetch { url } => {
            let parsed_url = Url::parse(url)?;
            let response = retrieve_page(&parsed_url).await?;
            println!("Response:\n {}", response);
            let document = Html::parse_document(&response);

            let page_info = PageInfo::new(&document);
            println!("Info: {}", page_info);

            let content = get_content(&document).unwrap();
            println!("Content:\n{}", content);
        }
        Commands::Browse { url } => {
            let browser_client = BrowserClient::new().await.unwrap();
            let html_text = browser_client.load_url(url).await.unwrap();
            let document = Html::parse_document(&html_text);
            let content = get_content(&document).unwrap();
            println!("Content:\n{}", content);
        }
    };

    Ok(())
}

async fn retrieve_page(url: &Url) -> Result<String, Box<dyn Error>> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.63 Safari/537.36";

    let client = reqwest::Client::builder().user_agent(user_agent).build()?;
    let resp = client.get(url.clone()).send().await?;
    Ok(resp.text().await?)
}
