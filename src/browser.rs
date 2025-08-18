use chromiumoxide::{Browser, BrowserConfig, browser::HeadlessMode};
use chromiumoxide::cdp::browser_protocol::network::{
    self, EventRequestWillBeSent, EventResponseReceived,
    EventLoadingFinished, EventLoadingFailed
};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures_util::pin_mut;

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub resource_type: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone)]
pub struct NetworkResponse {
    pub request_id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub mime_type: String,
    pub timestamp: f64,
    pub encoded_data_length: Option<u64>,
}

#[derive(Debug)]
pub struct NetworkAnalysis {
    pub total_requests: usize,
    pub requests_by_type: HashMap<String, usize>,
    pub total_size: u64,
    pub load_time: u64,
    pub requests: Vec<NetworkRequest>,
    pub responses: Vec<NetworkResponse>,
    pub failed_requests: Vec<String>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    Request(EventRequestWillBeSent),
    Response(EventResponseReceived),
    Finished(EventLoadingFinished),
    Failed(EventLoadingFailed),
}

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

    pub async fn load_url_with_network_analysis(
        &self,
        url: &str,
    ) -> Result<(String, NetworkAnalysis), Box<dyn Error + Send + Sync>> {
        let page = self.browser.new_page("about:blank").await?;
        
        // Enable network domain
        page.execute(network::EnableParams::default()).await?;
        
        let network_events = Arc::new(Mutex::new(Vec::new()));
        let start_time = std::time::Instant::now();
        
        // Set up event listeners
        let events_clone_1 = network_events.clone();
        let request_listener = page.event_listener::<EventRequestWillBeSent>().await?;
        let request_task = tokio::spawn(async move {
            pin_mut!(request_listener);
            while let Some(event) = request_listener.next().await {
                let mut events = events_clone_1.lock().await;
                events.push(NetworkEvent::Request((*event).clone()));
            }
        });

        let events_clone_2 = network_events.clone();
        let response_listener = page.event_listener::<EventResponseReceived>().await?;
        let response_task = tokio::spawn(async move {
            pin_mut!(response_listener);
            while let Some(event) = response_listener.next().await {
                let mut events = events_clone_2.lock().await;
                events.push(NetworkEvent::Response((*event).clone()));
            }
        });

        let events_clone_3 = network_events.clone();
        let finished_listener = page.event_listener::<EventLoadingFinished>().await?;
        let finished_task = tokio::spawn(async move {
            pin_mut!(finished_listener);
            while let Some(event) = finished_listener.next().await {
                let mut events = events_clone_3.lock().await;
                events.push(NetworkEvent::Finished((*event).clone()));
            }
        });

        let events_clone_4 = network_events.clone();
        let failed_listener = page.event_listener::<EventLoadingFailed>().await?;
        let failed_task = tokio::spawn(async move {
            pin_mut!(failed_listener);
            while let Some(event) = failed_listener.next().await {
                let mut events = events_clone_4.lock().await;
                events.push(NetworkEvent::Failed((*event).clone()));
            }
        });
        
        // Navigate and wait
        page.goto(url).await?;
        page.wait_for_navigation().await?;
        
        // Give some extra time for requests to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        let content = page.content().await?;
        let load_time = start_time.elapsed().as_millis() as u64;
        
        // Cancel event listeners
        request_task.abort();
        response_task.abort();
        finished_task.abort();
        failed_task.abort();
        
        let analysis = self.analyze_network_events(&network_events, load_time).await;
        
        Ok((content, analysis))
    }

    async fn analyze_network_events(
        &self,
        events: &Arc<Mutex<Vec<NetworkEvent>>>,
        load_time: u64,
    ) -> NetworkAnalysis {
        let events = events.lock().await;
        let mut requests = Vec::new();
        let mut responses = Vec::new();
        let mut failed_requests = Vec::new();
        let mut requests_by_type = HashMap::new();
        let mut total_size = 0u64;

        for event in events.iter() {
            match event {
                NetworkEvent::Request(req_event) => {
                    let resource_type = req_event.r#type.as_ref()
                        .map(|t| format!("{:?}", t).to_lowercase())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    *requests_by_type.entry(resource_type.clone()).or_insert(0) += 1;
                    
                    let headers = HashMap::new(); // Simplified for now - headers are complex in chromiumoxide

                    requests.push(NetworkRequest {
                        request_id: req_event.request_id.inner().clone(),
                        url: req_event.request.url.clone(),
                        method: req_event.request.method.clone(),
                        headers,
                        resource_type,
                        timestamp: *req_event.timestamp.inner(),
                    });
                }
                NetworkEvent::Response(resp_event) => {
                    total_size += resp_event.response.encoded_data_length as u64;
                    
                    let headers = HashMap::new(); // Simplified for now

                    responses.push(NetworkResponse {
                        request_id: resp_event.request_id.inner().clone(),
                        status: resp_event.response.status as u16,
                        headers,
                        mime_type: resp_event.response.mime_type.clone(),
                        timestamp: *resp_event.timestamp.inner(),
                        encoded_data_length: Some(resp_event.response.encoded_data_length as u64),
                    });
                }
                NetworkEvent::Failed(failed_event) => {
                    failed_requests.push(format!(
                        "Request {} failed: {}", 
                        failed_event.request_id.inner(), 
                        failed_event.error_text
                    ));
                }
                NetworkEvent::Finished(_) => {
                    // We can use this for timing analysis later
                }
            }
        }

        NetworkAnalysis {
            total_requests: requests.len(),
            requests_by_type,
            total_size,
            load_time,
            requests,
            responses,
            failed_requests,
        }
    }
}
