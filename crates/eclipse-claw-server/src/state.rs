use std::sync::Arc;

use eclipse_claw_connectors::{DoctorReport, RuntimeSignals};
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
    /// CDP navigation is disabled unless the operator explicitly enables it.
    pub cdp_enabled: bool,
    /// Read-only capability report. It contains booleans and policy metadata,
    /// never credential values or browser-session data.
    pub doctor: Arc<DoctorReport>,
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
        let cdp_enabled = std::env::var("ECLIPSE_ENABLE_CDP")
            .ok()
            .is_some_and(|value| matches!(value.trim(), "1" | "true" | "yes" | "on"));
        let doctor = DoctorReport::from_signals(RuntimeSignals {
            local_fetch_ready: true,
            local_llm_ready: !llm.is_empty(),
            cloud_key_present: eclipse_claw_connectors::cloud_key_present(),
            cloud_fallback_enabled: eclipse_claw_connectors::cloud_fallback_enabled(),
            public_egress_only: true,
            session_cookie_transfer_enabled: false,
            untrusted_content_boundary: true,
            cdp_browser_enabled: cdp_enabled,
        });

        Self {
            client: Arc::new(client),
            llm: Arc::new(llm),
            chrome_ws,
            cdp_enabled,
            doctor: Arc::new(doctor),
        }
    }
}
