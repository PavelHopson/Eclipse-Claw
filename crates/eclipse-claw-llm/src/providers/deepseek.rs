/// DeepSeek provider — api.deepseek.com (OpenAI-compatible endpoint).
///
/// DeepSeek's API is OpenAI-compatible but with a separate base URL,
/// different model names, and an optional "reasoning" prefix stripped
/// from `deepseek-reasoner` responses.
use async_trait::async_trait;
use serde_json::json;

use crate::clean::strip_thinking_tags;
use crate::error::LlmError;
use crate::provider::{CompletionRequest, LlmProvider};

use super::load_api_key;

const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/v1";

pub struct DeepSeekProvider {
    client: reqwest::Client,
    key: String,
    default_model: String,
}

impl DeepSeekProvider {
    /// Returns `None` if no API key is available (param or `DEEPSEEK_API_KEY` env var).
    pub fn new(key_override: Option<String>, model: Option<String>) -> Option<Self> {
        let key = load_api_key(key_override, "DEEPSEEK_API_KEY")?;

        Some(Self {
            client: reqwest::Client::new(),
            key,
            default_model: model.unwrap_or_else(|| "deepseek-chat".into()),
        })
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    async fn complete(&self, request: &CompletionRequest) -> Result<String, LlmError> {
        let model = if request.model.is_empty() {
            &self.default_model
        } else {
            &request.model
        };

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();

        let mut body = json!({
            "model": model,
            "messages": messages,
        });

        if request.json_mode {
            body["response_format"] = json!({ "type": "json_object" });
        }
        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max) = request.max_tokens {
            body["max_tokens"] = json!(max);
        }

        let url = format!("{}/chat/completions", DEEPSEEK_API_URL);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let safe_text = if text.len() > 500 { &text[..500] } else { &text };
            return Err(LlmError::ProviderError(format!(
                "deepseek returned {status}: {safe_text}"
            )));
        }

        let json: serde_json::Value = resp.json().await?;

        let raw = json["choices"][0]["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| {
                LlmError::InvalidJson(
                    "missing choices[0].message.content in deepseek response".into(),
                )
            })?;

        // deepseek-reasoner wraps output in <think>...</think> — strip it
        Ok(strip_thinking_tags(&raw))
    }

    async fn is_available(&self) -> bool {
        !self.key.is_empty()
    }

    fn name(&self) -> &str {
        "deepseek"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_key_returns_none() {
        assert!(DeepSeekProvider::new(Some(String::new()), None).is_none());
    }

    #[test]
    fn explicit_key_constructs() {
        let provider = DeepSeekProvider::new(
            Some("sk-test-key".into()),
            Some("deepseek-chat".into()),
        )
        .expect("should construct");
        assert_eq!(provider.name(), "deepseek");
        assert_eq!(provider.default_model(), "deepseek-chat");
    }

    #[test]
    fn reasoner_model_constructs() {
        let provider = DeepSeekProvider::new(
            Some("sk-test-key".into()),
            Some("deepseek-reasoner".into()),
        )
        .unwrap();
        assert_eq!(provider.default_model(), "deepseek-reasoner");
    }

    #[test]
    fn default_model_is_deepseek_chat() {
        let provider = DeepSeekProvider::new(Some("sk-key".into()), None).unwrap();
        assert_eq!(provider.default_model(), "deepseek-chat");
    }

    #[test]
    #[ignore = "mutates process env; run with --test-threads=1"]
    fn env_var_key_fallback() {
        unsafe { std::env::set_var("DEEPSEEK_API_KEY", "sk-env-key") };
        let provider = DeepSeekProvider::new(None, None).expect("should construct from env");
        assert_eq!(provider.key, "sk-env-key");
        unsafe { std::env::remove_var("DEEPSEEK_API_KEY") };
    }
}
