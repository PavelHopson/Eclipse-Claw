use std::sync::Arc;

use eclipse_claw_connectors::{DoctorReport, RuntimeSignals};
use eclipse_claw_fetch::{BrowserProfile, FetchClient, FetchConfig};
use eclipse_claw_llm::chain::ProviderChain;

use crate::worker::RemoteCdpWorker;

/// Shared application state — cloned cheaply via Arc.
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<FetchClient>,
    pub llm: Arc<ProviderChain>,
    /// Authenticated client for the separately isolated browser worker.
    pub cdp_worker: Option<RemoteCdpWorker>,
    /// CDP navigation is disabled unless the operator explicitly enables it.
    pub cdp_enabled: bool,
    /// Read-only capability report. It contains booleans and policy metadata,
    /// never credential values or browser-session data.
    pub doctor: Arc<DoctorReport>,
    pub audit: Option<eclipse_claw_audit::AuditStore>,
    pub audit_required: bool,
    pub audit_read_enabled: bool,
}

impl AppState {
    pub async fn new(_max_concurrency: usize) -> Result<Self, String> {
        let config = FetchConfig {
            browser: BrowserProfile::Chrome,
            ..FetchConfig::default()
        };
        let client = FetchClient::new(config).expect("failed to build fetch client");
        // REST never reads provider credentials or launches a provider directly.
        // Trusted CLI/MCP and the isolated worker retain their separate direct mode.
        let llm = ProviderChain::isolated();
        if env_enabled("ECLIPSE_REQUIRE_ISOLATED_WORKERS") && llm.is_empty() {
            return Err(
                "ECLIPSE_REQUIRE_ISOLATED_WORKERS=1 requires a valid authenticated LLM worker"
                    .into(),
            );
        }
        let cdp_requested = env_enabled("ECLIPSE_ENABLE_CDP");
        let cdp_worker = RemoteCdpWorker::from_env(cdp_requested)?;
        let cdp_enabled = cdp_requested && cdp_worker.is_some();
        let audit_required = env_enabled("ECLIPSE_AUDIT_REQUIRED");
        let audit = eclipse_claw_audit::AuditStore::from_env(audit_required)
            .map_err(|error| error.to_string())?;
        let audit_read_enabled = env_enabled("ECLIPSE_AUDIT_READ_ENABLED");
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

        Ok(Self {
            client: Arc::new(client),
            llm: Arc::new(llm),
            cdp_worker,
            cdp_enabled,
            doctor: Arc::new(doctor),
            audit,
            audit_required,
            audit_read_enabled,
        })
    }
}

fn env_enabled(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.trim(), "1" | "true" | "yes" | "on"))
}
