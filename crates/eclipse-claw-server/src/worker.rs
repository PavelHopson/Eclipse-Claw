use std::time::Duration;

use reqwest::Url;
use serde_json::{Value, json};

const MIN_TOKEN_LENGTH: usize = 32;
const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone)]
pub struct RemoteCdpWorker {
    endpoint: Url,
    token: String,
    client: reqwest::Client,
}

impl RemoteCdpWorker {
    pub fn from_env(required: bool) -> Result<Option<Self>, String> {
        let Some(base_url) = std::env::var("ECLIPSE_CDP_WORKER_URL").ok() else {
            return if required {
                Err(
                    "ECLIPSE_ENABLE_CDP=1 requires an isolated ECLIPSE_CDP_WORKER_URL; in-process browser launch is disabled"
                        .into(),
                )
            } else {
                Ok(None)
            };
        };
        let token = std::env::var("ECLIPSE_CDP_WORKER_TOKEN")
            .map_err(|_| "ECLIPSE_CDP_WORKER_URL requires ECLIPSE_CDP_WORKER_TOKEN".to_string())?;
        Self::new(base_url, token).map(Some)
    }

    fn new(base_url: String, token: String) -> Result<Self, String> {
        if token.len() < MIN_TOKEN_LENGTH || token.chars().any(char::is_whitespace) {
            return Err(
                "CDP worker token must contain at least 32 non-whitespace characters".into(),
            );
        }
        let mut endpoint =
            Url::parse(&base_url).map_err(|error| format!("invalid CDP worker URL: {error}"))?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(
                "CDP worker URL must be an http(s) origin without credentials, query or fragment"
                    .into(),
            );
        }
        endpoint.set_path("/v1/design-tokens");
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(45))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            endpoint,
            token,
            client,
        })
    }

    pub async fn extract(
        &self,
        url: &str,
        hydration_wait_ms: u64,
        viewport_width: u32,
    ) -> Result<Value, String> {
        let mut response = self
            .client
            .post(self.endpoint.clone())
            .bearer_auth(&self.token)
            .json(&json!({
                "url": url,
                "hydration_wait_ms": hydration_wait_ms,
                "viewport_width": viewport_width,
            }))
            .send()
            .await
            .map_err(|error| format!("CDP worker request failed: {error}"))?;
        let status = response.status();
        let bytes = read_limited(&mut response, MAX_RESPONSE_BYTES).await?;
        let payload: Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid CDP worker response: {error}"))?;
        if !status.is_success() || payload["ok"] != true {
            return Err(payload["error"]["message"]
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("CDP worker returned {status}")));
        }
        Ok(payload["data"].clone())
    }
}

async fn read_limited(response: &mut reqwest::Response, maximum: usize) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err("CDP worker response exceeded 5 MiB".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("CDP worker response failed: {error}"))?
    {
        if bytes.len().saturating_add(chunk.len()) > maximum {
            return Err("CDP worker response exceeded 5 MiB".into());
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
        assert!(RemoteCdpWorker::new("http://worker:3100".into(), "short".into()).is_err());
        assert!(
            RemoteCdpWorker::new(
                "http://user:pass@worker:3100".into(),
                "0123456789abcdef0123456789abcdef".into(),
            )
            .is_err()
        );
    }
}
