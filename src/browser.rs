use chromiumoxide::{Browser, BrowserConfig, Page, browser::HeadlessMode};
use futures_util::StreamExt;
use std::error::Error;

pub struct BrowserClient {
    browser: Browser,
}

impl BrowserClient {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .headless_mode(HeadlessMode::True)
                .build()?,
        )
        .await?;

        // Spawn the handler task
        tokio::spawn(async move { while let Some(_) = handler.next().await {} });

        Ok(Self { browser })
    }

    pub async fn load_url(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let page = self.browser.new_page(url).await?;
        page.wait_for_navigation().await?;

        let content = page.content().await?;
        Ok(content)
    }
}
