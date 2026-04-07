use std::sync::Arc;

use eclipse_claw_cdp::CdpConfig;
use eclipse_claw_fetch::{BrowserProfile, FetchClient, FetchConfig};
use eclipse_claw_llm::chain::ProviderChain;

/// Shared application state — cloned cheaply via Arc.
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<FetchClient>,
    pub llm: Arc<ProviderChain>,
    /// Chrome DevTools WebSocket URL for design token extraction.
    /// None = auto-launch headless Chrome per request.
    pub chrome_ws: Option<String>,
}

impl AppState {
    pub async fn new(_max_concurrency: usize) -> Self {
        let config = FetchConfig {
            browser: BrowserProfile::Chrome,
            ..FetchConfig::default()
        };
        let client = FetchClient::new(config).expect("failed to build fetch client");
        let llm = ProviderChain::default().await;
        let chrome_ws = std::env::var("ECLIPSE_CHROME_WS").ok();

        Self {
            client: Arc::new(client),
            llm: Arc::new(llm),
            chrome_ws,
        }
    }
}
