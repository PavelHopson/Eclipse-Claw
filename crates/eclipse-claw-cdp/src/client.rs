/// CDP client wrapping chromiumoxide for design token extraction.
///
/// Connects to an existing Chrome instance via DevTools WebSocket (preferred,
/// works with `chrome --remote-debugging-port=9222`) or launches a new
/// headless Chrome process.
use std::time::Duration;

use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use tracing::{debug, info, instrument};

use crate::error::CdpError;
use crate::scripts::EXTRACT_TOKENS;
use crate::tokens::DesignTokens;

/// Configuration for connecting to / launching Chrome.
#[derive(Debug, Clone)]
pub struct CdpConfig {
    /// WebSocket URL of an already-running Chrome instance.
    /// Example: `ws://127.0.0.1:9222/json/version`
    /// When `None`, a new headless Chrome is launched automatically.
    pub chrome_ws: Option<String>,

    /// Page load timeout in seconds (default: 30).
    pub timeout_secs: u64,

    /// Wait this many ms after navigation before running extraction scripts,
    /// to allow JS frameworks to hydrate (default: 1500 ms).
    pub hydration_wait_ms: u64,

    /// Viewport width for extraction (default: 1440).
    pub viewport_width: u32,
}

impl Default for CdpConfig {
    fn default() -> Self {
        Self {
            chrome_ws: None,
            timeout_secs: 30,
            hydration_wait_ms: 1500,
            viewport_width: 1440,
        }
    }
}

/// CDP client. One instance per extraction session.
/// The underlying Browser/Page handles connection + lifecycle.
pub struct CdpClient {
    config: CdpConfig,
}

impl CdpClient {
    pub fn new(config: CdpConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(CdpConfig::default())
    }

    /// Connect to Chrome via WebSocket URL (existing instance).
    /// Call this to get a connected Browser for manual control.
    pub async fn connect(ws_url: &str) -> Result<(Browser, impl futures::Stream<Item = chromiumoxide::cdp::browser_protocol::target::EventTargetCreated>), CdpError> {
        Browser::connect(ws_url)
            .await
            .map_err(|e| CdpError::Launch(e.to_string()))
    }

    /// Extract design tokens from a URL.
    ///
    /// Launches / connects to Chrome, navigates to the URL, waits for hydration,
    /// then runs the extraction script and parses the result.
    #[instrument(skip(self), fields(url = %url))]
    pub async fn extract_design_tokens(&self, url: &str) -> Result<DesignTokens, CdpError> {
        // Validate URL
        url::Url::parse(url).map_err(|_| CdpError::InvalidUrl(url.to_string()))?;

        info!("launching chrome for design token extraction");

        let (browser, mut handler) = match &self.config.chrome_ws {
            Some(ws) => {
                Browser::connect(ws)
                    .await
                    .map_err(|e| CdpError::Launch(e.to_string()))?
            }
            None => {
                let config = BrowserConfig::builder()
                    .no_sandbox()
                    .window_size(self.config.viewport_width, 900)
                    .build()
                    .map_err(|e| CdpError::Launch(e.to_string()))?;

                Browser::launch(config)
                    .await
                    .map_err(|e| CdpError::Launch(e.to_string()))?
            }
        };

        // Drive the browser event loop in the background
        let _driver = tokio::spawn(async move {
            while let Some(_) = handler.next().await {}
        });

        let page = browser
            .new_page(url)
            .await
            .map_err(|e| CdpError::Navigation(e.to_string()))?;

        // Wait for network idle + hydration
        debug!(wait_ms = self.config.hydration_wait_ms, "waiting for hydration");
        tokio::time::sleep(Duration::from_millis(self.config.hydration_wait_ms)).await;

        // Extract page metadata
        let title = page.get_title().await.ok().flatten();

        // Run design token extraction script
        debug!("running token extraction script");
        let result = page
            .evaluate(EXTRACT_TOKENS)
            .await
            .map_err(|e| CdpError::ScriptEval(e.to_string()))?;

        let json_str = result
            .value()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| CdpError::ScriptEval("extraction script returned no value".into()))?;

        let mut tokens: DesignTokens = serde_json::from_str(&json_str)
            .map_err(|e| CdpError::ScriptEval(format!("failed to parse token JSON: {e}")))?;

        tokens.url = url.to_string();
        tokens.title = title;

        info!(
            colors = tokens.colors.backgrounds.len(),
            fonts = tokens.typography.families.len(),
            css_vars = tokens.css_variables.len(),
            "design token extraction complete"
        );

        Ok(tokens)
    }
}
