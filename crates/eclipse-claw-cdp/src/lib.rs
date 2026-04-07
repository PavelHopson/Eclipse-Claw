//! eclipse-claw-cdp: Chrome DevTools Protocol integration.
//!
//! Extracts design tokens from live pages via getComputedStyle() —
//! exact computed values for colors, typography, spacing, shadows, and CSS variables.
//!
//! Requires a running Chrome instance (`chrome --remote-debugging-port=9222`)
//! or auto-launches headless Chrome via chromiumoxide.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use eclipse_claw_cdp::{CdpClient, CdpConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = CdpClient::with_defaults();
//!     let tokens = client.extract_design_tokens("https://linear.app").await.unwrap();
//!     println!("{}", serde_json::to_string_pretty(&tokens).unwrap());
//! }
//! ```

pub mod client;
pub mod error;
pub mod scripts;
pub mod tokens;

pub use client::{CdpClient, CdpConfig};
pub use error::CdpError;
pub use tokens::{
    ColorEntry, ColorTokens, CssVariable, DesignTokens, FontFamily, SpacingTokens,
    TypographyTokens,
};
