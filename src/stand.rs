use chromiumoxide::browser::{Browser, BrowserConfig, HeadlessMode};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::handler::HandlerConfig;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use spider_fingerprint::{
    EmulationConfiguration, Fingerprint,
    configs::Tier,
    spoof_headers,
    spoof_viewport::{Viewport, get_random_viewport},
};
use std::time::Duration;

// Implementation function
pub async fn handle_bot_test(
    output: &str,
    headless: bool,
    chrome_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
    use chromiumoxide::{
        browser::Browser, handler::HandlerConfig, page::ScreenshotParams,
    };
    use futures::StreamExt;
    use spider_fingerprint::{
        EmulationConfiguration, Fingerprint, configs::Tier, spoof_headers,
        spoof_viewport::get_random_viewport,
    };

    println!("🕷️  Initializing stealth browser session...");

    // Setup realistic fingerprinting
    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

    let mut emulation_config = EmulationConfiguration::setup_defaults(&ua);
    emulation_config.fingerprint = Fingerprint::Basic;
    emulation_config.tier = Tier::Basic;
    emulation_config.user_agent_data = Some(false);
    emulation_config.dismiss_dialogs = true;

    let vp = get_random_viewport();
    let viewport: Viewport = vp.into();

    println!(
        "📱 Using viewport: {}x{} (mobile: {})",
        viewport.width, viewport.height, viewport.emulating_mobile
    );

    // Generate stealth scripts and headers
    let emulation_script = spider_fingerprint::emulate(
        &ua,
        &emulation_config,
        &Some(&viewport),
        &None,
    );
    let headers = spoof_headers::emulate_headers(
        ua,
        &None,
        &None,
        true,
        &Some(viewport),
        &None,
        &Some(spoof_headers::HeaderDetailLevel::Extensive),
    );
    let extra_headers = spoof_headers::headers_to_hashmap(headers);

    // Configure handler for maximum stealth
    let config = HandlerConfig {
        request_intercept: true,
        viewport: Some(chromiumoxide::handler::viewport::Viewport {
            width: viewport.width,
            height: viewport.height,
            device_scale_factor: viewport.device_scale_factor,
            emulating_mobile: viewport.emulating_mobile,
            is_landscape: viewport.is_landscape,
            has_touch: viewport.has_touch,
        }),
        extra_headers: Some(extra_headers),
        ignore_analytics: true,
        cache_enabled: true,
        ..HandlerConfig::default()
    };

    // Connect to Chrome
    println!("🔗 Connecting to Chrome at {}...", chrome_url);

    // Launch browser instead of connecting
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .headless_mode(if headless {
                HeadlessMode::True
            } else {
                HeadlessMode::False
            })
            // .user_agent(ua)
            .build()?,
    )
    .await?;

    let handler_task =
        tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    // Create page and apply stealth
    let page = browser.new_page("about:blank").await?;

    if let Some(script) = emulation_script {
        page.evaluate_on_new_document(&script).await?;
        println!("✅ Applied fingerprint spoofing script");
    }

    // Navigate to test site
    println!("🌐 Navigating to bot.sannysoft.com...");
    page.goto("https://bot.sannysoft.com/").await?;

    // Wait for page to load and run detection
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Take screenshot
    println!("📸 Capturing screenshot...");
    page.save_screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .quality(95)
            .build(),
        &output,
    )
    .await?;

    println!("✅ Screenshot saved: {}", output);

    // Show some results
    if let Ok(Some(title)) = page.get_title().await {
        println!("📄 Page title: {}", title);
    }

    handler_task.abort();
    Ok(())
}

pub async fn test_bot_detection() -> Result<(), Box<dyn std::error::Error>> {
    // Create realistic user agent
    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

    // Setup fingerprint configuration for maximum stealth
    let mut emulation_config = EmulationConfiguration::setup_defaults(&ua);
    emulation_config.fingerprint = Fingerprint::Basic; // Use Basic fingerprinting
    emulation_config.tier = Tier::Basic; // Use Premium tier for best stealth
    emulation_config.user_agent_data = Some(false); // Disable user agent data hints
    emulation_config.dismiss_dialogs = true; // Auto-dismiss dialogs

    // Get random realistic viewport
    let vp = get_random_viewport();
    let viewport = vp.into();

    // Generate emulation script for fingerprint spoofing
    let emulation_script = spider_fingerprint::emulate(
        &ua,
        &emulation_config,
        &Some(&viewport),
        &None,
    );

    // Generate realistic headers
    let headers = spoof_headers::emulate_headers(
        ua,
        &None,
        &None,
        true,
        &Some(viewport),
        &None,
        &Some(spoof_headers::HeaderDetailLevel::Extensive),
    );

    let extra_headers = spoof_headers::headers_to_hashmap(headers);

    // Configure browser handler with stealth settings
    let config = HandlerConfig {
        request_intercept: true,
        viewport: Some(chromiumoxide::handler::viewport::Viewport {
            width: viewport.width,
            height: viewport.height,
            device_scale_factor: viewport.device_scale_factor,
            emulating_mobile: viewport.emulating_mobile,
            is_landscape: viewport.is_landscape,
            has_touch: viewport.has_touch,
        }),
        extra_headers: Some(extra_headers),
        ignore_https_errors: true,
        cache_enabled: true,
        service_worker_enabled: true,
        ignore_analytics: true, // Block analytics for stealth
        ..HandlerConfig::default()
    };

    // Connect to Chrome instance (assuming it's running on localhost:9222)
    let (mut browser, mut handler) =
        Browser::connect_with_config("http://localhost:9222", config).await?;

    // Spawn handler task
    let handle = tokio::task::spawn(async move {
        while let Some(_event) = handler.next().await {
            // Process events
        }
    });

    // Create new page
    let page = browser.new_page("about:blank").await?;

    // Inject fingerprint spoofing script if available
    if let Some(script) = emulation_script {
        page.evaluate_on_new_document(&script).await?;
    }

    // Navigate to bot detection test site
    println!("Navigating to bot detection test...");
    page.goto("https://bot.sannysoft.com/").await?;

    // Wait for page to fully load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Take full page screenshot
    println!("Taking screenshot...");
    let screenshot_data = page
        .screenshot(
            ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(true)
                .quality(90)
                .build(),
        )
        .await?;

    // Save screenshot
    let filename = "bot_detection_test.png";
    tokio::fs::write(filename, &screenshot_data).await?;

    println!("Screenshot saved as: {}", filename);
    println!("Size: {} bytes", screenshot_data.len());

    // Optionally get page title and some basic info
    if let Ok(title) = page.get_title().await {
        println!(
            "Page title: {}",
            title.unwrap_or_else(|| "No title".to_string())
        );
    }

    // Clean up
    handle.abort();

    Ok(())
}

// Alternative version if you want to launch Chrome instead of connecting
pub async fn test_bot_detection_with_launch()
-> Result<(), Box<dyn std::error::Error>> {
    use chromiumoxide::browser::BrowserConfig;

    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

    let mut emulation_config = EmulationConfiguration::setup_defaults(&ua);
    emulation_config.fingerprint = Fingerprint::Basic;
    emulation_config.tier = Tier::Basic;
    emulation_config.user_agent_data = Some(false);

    let vp = get_random_viewport();
    let viewport = vp.into();

    let emulation_script = spider_fingerprint::emulate(
        &ua,
        &emulation_config,
        &Some(&viewport),
        &None,
    );

    let headers = spoof_headers::emulate_headers(
        ua,
        &None,
        &None,
        true,
        &Some(viewport),
        &None,
        &Some(spoof_headers::HeaderDetailLevel::Extensive),
    );
    let extra_headers = spoof_headers::headers_to_hashmap(headers);

    // Launch Chrome with custom configuration
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            // .user_agent(ua)
            .disable_default_args()
            .args(vec![
                "--no-first-run",
                "--disable-blink-features=AutomationControlled",
                "--disable-dev-shm-usage",
                "--no-sandbox",
                "--disable-web-security",
                "--disable-features=VizDisplayCompositor",
            ])
            .build()?,
    )
    .await?;

    let handle =
        tokio::task::spawn(
            async move { while let Some(_) = handler.next().await {} },
        );

    let page = browser.new_page("about:blank").await?;

    // Apply emulation script
    if let Some(script) = emulation_script {
        page.evaluate_on_new_document(&script).await?;
    }

    // Set extra headers
    if !extra_headers.is_empty() {
        use chromiumoxide::cdp::browser_protocol::network::Headers;
        use chromiumoxide::cdp::browser_protocol::network::SetExtraHttpHeadersParams;

        let headers_json = serde_json::to_value(&extra_headers)?;
        let headers = chromiumoxide::cdp::browser_protocol::network::Headers::new(
            headers_json,
        );
        page.execute(SetExtraHttpHeadersParams::new(headers))
            .await?;
    }

    page.goto("https://bot.sannysoft.com/").await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    page.save_screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .build(),
        "bot_detection_launch.png",
    )
    .await?;

    println!("Screenshot saved as: bot_detection_launch.png");

    handle.abort();
    Ok(())
}
