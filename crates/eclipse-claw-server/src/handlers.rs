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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::{get, post};
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/health", get(health))
            .route("/extract/html", post(extract_html))
    }

    async fn body_json(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    // ── /health ────────────────────────────────────────────────

    #[tokio::test]
    async fn health_returns_ok() {
        let app = test_router();
        let req = Request::get("/health").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let json = body_json(res.into_body()).await;
        assert_eq!(json["ok"], true);
        assert_eq!(json["service"], "eclipse-claw-server");
    }

    // ── /extract/html ──────────────────────────────────────────

    #[tokio::test]
    async fn extract_html_returns_markdown_for_simple_html() {
        let app = test_router();
        let payload = json!({
            "html": "<html><body><article><h1>Hello World</h1><p>Test paragraph.</p></article></body></html>"
        });
        let req = Request::post("/extract/html")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let json = body_json(res.into_body()).await;
        assert_eq!(json["ok"], true);
        assert!(json["data"].is_object());
    }

    #[tokio::test]
    async fn extract_html_rejects_empty_html() {
        let app = test_router();
        let payload = json!({ "html": "" });
        let req = Request::post("/extract/html")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let json = body_json(res.into_body()).await;
        assert_eq!(json["ok"], false);
        assert_eq!(json["error"]["code"], "bad_request");
    }

    #[tokio::test]
    async fn extract_html_handles_selectors() {
        let app = test_router();
        let payload = json!({
            "html": "<html><body><nav>Skip</nav><main><h1>Keep</h1></main></body></html>",
            "only_main_content": true,
            "exclude_selectors": ["nav"]
        });
        let req = Request::post("/extract/html")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let json = body_json(res.into_body()).await;
        assert_eq!(json["ok"], true);
    }

    // ── ApiError response mapping ──────────────────────────────

    #[tokio::test]
    async fn bad_request_error_returns_400() {
        let err = ApiError::BadRequest("missing field".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn fetch_error_returns_502() {
        let err = ApiError::Fetch("timeout".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn extraction_error_returns_422() {
        let err = ApiError::Extraction("parse failure".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn internal_error_returns_500() {
        let err = ApiError::Internal("panic".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn error_body_contains_ok_false_and_error_code() {
        let err = ApiError::BadRequest("test".into());
        let response = err.into_response();
        let json = body_json(response.into_body()).await;
        assert_eq!(json["ok"], false);
        assert_eq!(json["error"]["code"], "bad_request");
        assert!(json["error"]["message"].as_str().unwrap().contains("test"));
    }
}
