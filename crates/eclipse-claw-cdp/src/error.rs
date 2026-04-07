use thiserror::Error;

#[derive(Debug, Error)]
pub enum CdpError {
    #[error("chrome launch error: {0}")]
    Launch(String),

    #[error("cdp protocol error: {0}")]
    Protocol(String),

    #[error("navigation error: {0}")]
    Navigation(String),

    #[error("script evaluation error: {0}")]
    ScriptEval(String),

    #[error("invalid url: {0}")]
    InvalidUrl(String),

    #[error("timeout waiting for page load")]
    Timeout,
}

impl From<chromiumoxide::error::CdpError> for CdpError {
    fn from(e: chromiumoxide::error::CdpError) -> Self {
        CdpError::Protocol(e.to_string())
    }
}
