use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;
use serde::Deserialize;

use crate::error::LlmError;
use crate::provider::{CompletionRequest, LlmProvider};

const MIN_TOKEN_LENGTH: usize = 32;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

/// Authenticated client for a separately isolated Eclipse Claw LLM worker.
pub struct WorkerProvider {
    endpoint: Url,
    token: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct WorkerResponse {
    ok: bool,
    data: Option<WorkerData>,
    error: Option<WorkerError>,
}

#[derive(Deserialize)]
struct WorkerData {
    text: String,
}

#[derive(Deserialize)]
struct WorkerError {
    message: String,
}

impl WorkerProvider {
    pub fn from_env() -> Result<Option<Self>, LlmError> {
        let Some(base_url) = std::env::var("ECLIPSE_LLM_WORKER_URL").ok() else {
            return Ok(None);
        };
        let token = std::env::var("ECLIPSE_LLM_WORKER_TOKEN").map_err(|_| {
            LlmError::ProviderError(
                "ECLIPSE_LLM_WORKER_URL requires ECLIPSE_LLM_WORKER_TOKEN".into(),
            )
        })?;
        Self::new(base_url, token).map(Some)
    }

    pub fn new(base_url: String, token: String) -> Result<Self, LlmError> {
        if token.len() < MIN_TOKEN_LENGTH || token.chars().any(char::is_whitespace) {
            return Err(LlmError::ProviderError(
                "LLM worker token must contain at least 32 non-whitespace characters".into(),
            ));
        }
        let mut endpoint = Url::parse(&base_url)
            .map_err(|error| LlmError::ProviderError(format!("invalid LLM worker URL: {error}")))?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(LlmError::ProviderError(
                "LLM worker URL must be an http(s) origin without credentials, query or fragment"
                    .into(),
            ));
        }
        endpoint.set_path("/v1/complete");

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(90))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| LlmError::ProviderError(error.to_string()))?;

        Ok(Self {
            endpoint,
            token,
            client,
        })
    }
}

#[async_trait]
impl LlmProvider for WorkerProvider {
    async fn complete(&self, request: &CompletionRequest) -> Result<String, LlmError> {
        let mut response = self
            .client
            .post(self.endpoint.clone())
            .bearer_auth(&self.token)
            .json(request)
            .send()
            .await
            .map_err(|error| {
                LlmError::ProviderError(format!("LLM worker request failed: {error}"))
            })?;
        let status = response.status();
        let bytes = read_limited(&mut response, MAX_RESPONSE_BYTES).await?;
        let payload = serde_json::from_slice::<WorkerResponse>(&bytes).map_err(|error| {
            LlmError::InvalidJson(format!("invalid LLM worker response: {error}"))
        })?;

        if status.is_success() && payload.ok {
            return payload.data.map(|data| data.text).ok_or_else(|| {
                LlmError::InvalidJson("LLM worker response omitted data.text".into())
            });
        }

        Err(LlmError::ProviderError(
            payload
                .error
                .map(|error| error.message)
                .unwrap_or_else(|| format!("LLM worker returned {status}")),
        ))
    }

    async fn is_available(&self) -> bool {
        // Startup diagnostics are deliberately read-only and do not probe the worker.
        true
    }

    fn name(&self) -> &str {
        "isolated-worker"
    }
}

async fn read_limited(
    response: &mut reqwest::Response,
    maximum: usize,
) -> Result<Vec<u8>, LlmError> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err(LlmError::ProviderError(
            "LLM worker response exceeded 4 MiB".into(),
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| LlmError::ProviderError(format!("LLM worker response failed: {error}")))?
    {
        if bytes.len().saturating_add(chunk.len()) > maximum {
            return Err(LlmError::ProviderError(
                "LLM worker response exceeded 4 MiB".into(),
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_weak_tokens_and_url_credentials() {
        assert!(WorkerProvider::new("http://worker:3100".into(), "short".into()).is_err());
        assert!(
            WorkerProvider::new(
                "http://user:pass@worker:3100".into(),
                "0123456789abcdef0123456789abcdef".into(),
            )
            .is_err()
        );
    }

    #[test]
    fn accepts_private_container_origin_as_explicit_operator_configuration() {
        assert!(
            WorkerProvider::new(
                "http://llm-worker:3100".into(),
                "0123456789abcdef0123456789abcdef".into(),
            )
            .is_ok()
        );
    }
}
