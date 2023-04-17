use dom_content_extraction::scraper::{Html, Selector};
use std::collections::HashMap;
use std::fmt;

/// Extract valuable data from html
pub struct PageInfo {
    title: Option<String>,
    meta: Vec<HashMap<String, String>>,
    html_attrs: HashMap<String, String>,
}

impl PageInfo {
    pub fn new(page: &Html) -> Self {
        PageInfo {
            title: Self::extract_title(page),
            meta: Self::extract_meta_tags(page),
            html_attrs: Self::extract_html_attributes(page),
        }
    }

    fn extract_title(page: &Html) -> Option<String> {
        let title_selector = Selector::parse("title").unwrap();
        page.select(&title_selector)
            .next()
            .map(|element| element.inner_html())
    }

    pub fn extract_meta_tags(page: &Html) -> Vec<HashMap<String, String>> {
        let meta_selector = Selector::parse("meta").unwrap();

        page.select(&meta_selector)
            .map(|element| {
                element
                    .value()
                    .attrs()
                    .map(|(name, value)| (name.to_owned(), value.to_owned()))
                    .collect()
            })
            .collect()
    }

    fn extract_html_attributes(page: &Html) -> HashMap<String, String> {
        let html_selector = Selector::parse("html").unwrap();
        let html_element = page.select(&html_selector).next().unwrap();
        html_element
            .value()
            .attrs()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect()
    }
}

impl fmt::Display for PageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Title: {}", self.title.clone().unwrap_or_default())?;
        writeln!(f, "Meta tags:")?;
        for (i, meta) in self.meta.iter().enumerate() {
            writeln!(f, "Meta tag {}:", i + 1)?;
            for (k, v) in meta {
                writeln!(f, "  {}: {}", k, v)?;
            }
        }
        writeln!(f, "\nHTML tag attributes:")?;
        for (k, v) in &self.html_attrs {
            writeln!(f, "  {}: {}", k, v)?;
        }
        Ok(())
    }
}
