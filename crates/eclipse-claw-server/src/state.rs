use std::sync::Arc;

use eclipse_claw_fetch::{BrowserProfile, FetchClient, FetchConfig};
use eclipse_claw_llm::chain::ProviderChain;

/// Shared application state — cloned cheaply via Arc.
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<FetchClient>,
    pub llm: Arc<ProviderChain>,
}

impl AppState {
    pub async fn new(_max_concurrency: usize) -> Self {
        let config = FetchConfig {
            browser: BrowserProfile::Chrome,
            ..FetchConfig::default()
        };
        let client = FetchClient::new(config).expect("failed to build fetch client");
        let llm = ProviderChain::default().await;

        Self {
            client: Arc::new(client),
            llm: Arc::new(llm),
        }
    }
}
