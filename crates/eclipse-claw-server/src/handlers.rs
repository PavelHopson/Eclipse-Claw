use axum::{Json, extract::State};
use eclipse_claw_cdp::{CdpClient, CdpConfig};
use eclipse_claw_core::ExtractionOptions;
use eclipse_claw_llm::provider::{CompletionRequest, LlmProvider, Message};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::ApiError;
use crate::state::AppState;

// ── Request / Response types ────────────────────────────────────────────────

/// `POST /extract` — fetch a URL and extract its content.
#[derive(Debug, Deserialize)]
pub struct ExtractRequest {
    /// URL to fetch and extract.
    pub url: String,
    /// CSS selectors to include (empty = auto-detect main content).
    #[serde(default)]
    pub include_selectors: Vec<String>,
    /// CSS selectors to exclude from the output.
    #[serde(default)]
    pub exclude_selectors: Vec<String>,
    /// Only return the primary article/main element.
    #[serde(default)]
    pub only_main_content: bool,
    /// Include raw HTML of the extracted node.
    #[serde(default)]
    pub include_raw_html: bool,
}

/// `POST /extract/html` — extract from raw HTML provided inline.
#[derive(Debug, Deserialize)]
pub struct ExtractHtmlRequest {
    /// Raw HTML to parse.
    pub html: String,
    /// Optional source URL — used for resolving relative links.
    pub url: Option<String>,
    #[serde(default)]
    pub include_selectors: Vec<String>,
    #[serde(default)]
    pub exclude_selectors: Vec<String>,
    #[serde(default)]
    pub only_main_content: bool,
    #[serde(default)]
    pub include_raw_html: bool,
}

/// `POST /summarise` — fetch + extract + pass to LLM for a summary.
#[derive(Debug, Deserialize)]
pub struct SummariseRequest {
    /// URL to summarise.
    pub url: String,
    /// Custom system prompt (optional; default is a concise summariser).
    pub system_prompt: Option<String>,
    /// LLM model override (empty = provider default).
    #[serde(default)]
    pub model: String,
    /// Max tokens in the LLM response.
    pub max_tokens: Option<u32>,
}

/// `POST /batch` — fetch and extract multiple URLs in parallel.
#[derive(Debug, Deserialize)]
pub struct BatchRequest {
    pub urls: Vec<String>,
    #[serde(default)]
    pub only_main_content: bool,
}

#[derive(Debug, Serialize)]
pub struct BatchItem {
    pub url: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Handlers ────────────────────────────────────────────────────────────────

/// `POST /extract` — fetch URL, return structured extraction.
pub async fn extract_url(
    State(state): State<AppState>,
    Json(req): Json<ExtractRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.url.is_empty() {
        return Err(ApiError::BadRequest("url is required".into()));
    }

    let options = ExtractionOptions {
        include_selectors: req.include_selectors,
        exclude_selectors: req.exclude_selectors,
        only_main_content: req.only_main_content,
        include_raw_html: req.include_raw_html,
    };

    let result = state
        .client
        .fetch_and_extract_with_options(&req.url, &options)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(json!({ "ok": true, "data": result })))
}

/// `POST /extract/html` — parse raw HTML inline.
pub async fn extract_html(
    Json(req): Json<ExtractHtmlRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.html.is_empty() {
        return Err(ApiError::BadRequest("html is required".into()));
    }

    let options = ExtractionOptions {
        include_selectors: req.include_selectors,
        exclude_selectors: req.exclude_selectors,
        only_main_content: req.only_main_content,
        include_raw_html: req.include_raw_html,
    };

    let result = eclipse_claw_core::extract_with_options(
        &req.html,
        req.url.as_deref(),
        &options,
    )
    .map_err(ApiError::from)?;

    Ok(Json(json!({ "ok": true, "data": result })))
}

/// `POST /summarise` — fetch URL, extract, pass markdown to LLM.
pub async fn summarise_url(
    State(state): State<AppState>,
    Json(req): Json<SummariseRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.url.is_empty() {
        return Err(ApiError::BadRequest("url is required".into()));
    }

    if state.llm.is_empty() {
        return Err(ApiError::Internal(
            "no LLM providers configured (set OPENAI_API_KEY / DEEPSEEK_API_KEY / ANTHROPIC_API_KEY or run Ollama)".into(),
        ));
    }

    // Step 1: extract content
    let extracted = state
        .client
        .fetch_and_extract(&req.url)
        .await
        .map_err(ApiError::from)?;

    let markdown = &extracted.content.markdown;
    let title = extracted
        .metadata
        .title
        .as_deref()
        .unwrap_or("(no title)");

    // Step 2: build LLM request
    let system = req.system_prompt.unwrap_or_else(|| {
        "You are a concise content summariser. Given a web page's markdown content, \
         produce a clear, structured summary in 3-5 bullet points. Focus on key facts, \
         avoid filler language.".into()
    });

    let user_message = format!(
        "Summarise this page:\n\n**Title:** {title}\n**URL:** {}\n\n---\n\n{markdown}",
        req.url
    );

    let llm_req = CompletionRequest {
        model: req.model,
        messages: vec![
            Message { role: "system".into(), content: system },
            Message { role: "user".into(), content: user_message },
        ],
        temperature: None,
        max_tokens: req.max_tokens,
        json_mode: false,
    };

    // Step 3: call LLM chain
    let summary = state
        .llm
        .complete(&llm_req)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "ok": true,
        "data": {
            "url": req.url,
            "title": title,
            "word_count": extracted.metadata.word_count,
            "summary": summary,
        }
    })))
}

/// `POST /batch` — extract multiple URLs concurrently.
pub async fn batch_extract(
    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.urls.is_empty() {
        return Err(ApiError::BadRequest("urls array is required and must not be empty".into()));
    }
    if req.urls.len() > 50 {
        return Err(ApiError::BadRequest("maximum 50 URLs per batch".into()));
    }

    let options = ExtractionOptions {
        only_main_content: req.only_main_content,
        ..ExtractionOptions::default()
    };

    let url_refs: Vec<&str> = req.urls.iter().map(|u| u.as_str()).collect();
    let results = state
        .client
        .fetch_and_extract_batch_with_options(&url_refs, 8, &options)
        .await;

    let items: Vec<BatchItem> = results
        .into_iter()
        .map(|r| match r.result {
            Ok(data) => BatchItem {
                url: r.url,
                ok: true,
                data: Some(serde_json::to_value(data).unwrap_or(Value::Null)),
                error: None,
            },
            Err(e) => BatchItem {
                url: r.url,
                ok: false,
                data: None,
                error: Some(e.to_string()),
            },
        })
        .collect();

    let total = items.len();
    let succeeded = items.iter().filter(|i| i.ok).count();

    Ok(Json(json!({
        "ok": true,
        "data": {
            "total": total,
            "succeeded": succeeded,
            "failed": total - succeeded,
            "results": items,
        }
    })))
}

/// `POST /design-tokens` — extract design tokens via Chrome DevTools Protocol.
///
/// Requires Chrome with `--remote-debugging-port=9222` running on the server,
/// or set `ECLIPSE_CHROME_WS` env var. Each request launches/reuses a Chrome tab.
pub async fn design_tokens(
    State(state): State<AppState>,
    Json(req): Json<DesignTokensRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.url.is_empty() {
        return Err(ApiError::BadRequest("url is required".into()));
    }

    let config = CdpConfig {
        chrome_ws: state.chrome_ws.clone().or(req.chrome_ws),
        hydration_wait_ms: req.hydration_wait_ms.unwrap_or(1500),
        viewport_width: req.viewport_width.unwrap_or(1440),
        ..CdpConfig::default()
    };

    let client = CdpClient::new(config);
    let tokens = client
        .extract_design_tokens(&req.url)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({ "ok": true, "data": tokens })))
}

/// `POST /design-tokens` request body.
#[derive(Debug, Deserialize)]
pub struct DesignTokensRequest {
    pub url: String,
    /// Override Chrome DevTools WebSocket URL (optional).
    pub chrome_ws: Option<String>,
    /// Ms to wait after navigation for JS hydration (default: 1500).
    pub hydration_wait_ms: Option<u64>,
    /// Viewport width for extraction (default: 1440).
    pub viewport_width: Option<u32>,
}

/// `GET /health` — liveness probe.
pub async fn health() -> Json<Value> {
    Json(json!({ "ok": true, "service": "eclipse-claw-server" }))
}
