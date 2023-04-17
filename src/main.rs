use clap::{Arg, Command};
use dom_content_extraction::scraper::Html;
use dom_content_extraction::{get_node_text, DensityTree};
use html::PageInfo;
use reqwest::Url;
use std::error::Error;

mod html;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Pageinfo")
        .version("0.1")
        .author("oiwn <https://github.org/oiwn>>")
        .about("Retrieves a web page")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .required(true),
        )
        .get_matches();

    let url = matches.get_one::<String>("url").unwrap();
    let parsed_url = Url::parse(url)?;

    let content = retrieve_page(&parsed_url).await?;
    let document = Html::parse_document(&content);

    // Extract html info
    let page_info = PageInfo::new(&document);
    println!("Info: {}", page_info);

    let dtree = DensityTree::from_document(&document);
    let sorted_nodes = dtree.sorted_nodes();

    let longest_text = sorted_nodes
        .iter()
        .rev()
        .take(8)
        .map(|x| get_node_text(x.node_id, &document))
        .max_by_key(|s| s.len())
        .unwrap();

    println!("Content:\n{}", longest_text);

    Ok(())
}

async fn retrieve_page(url: &Url) -> Result<String, Box<dyn Error>> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.63 Safari/537.36";

    let client = reqwest::Client::builder().user_agent(user_agent).build()?;
    let resp = client.get(url.clone()).send().await?;
    let content = resp.text().await?;
    Ok(content)
}
