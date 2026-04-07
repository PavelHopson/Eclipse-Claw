use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("fetch error: {0}")]
    Fetch(String),

    #[error("extraction error: {0}")]
    Extraction(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Fetch(_) => (StatusCode::BAD_GATEWAY, "fetch_error"),
            ApiError::Extraction(_) => (StatusCode::UNPROCESSABLE_ENTITY, "extraction_error"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = json!({
            "ok": false,
            "error": { "code": code, "message": self.to_string() }
        });

        (status, axum::Json(body)).into_response()
    }
}

impl From<eclipse_claw_fetch::FetchError> for ApiError {
    fn from(e: eclipse_claw_fetch::FetchError) -> Self {
        ApiError::Fetch(e.to_string())
    }
}

impl From<eclipse_claw_core::ExtractError> for ApiError {
    fn from(e: eclipse_claw_core::ExtractError) -> Self {
        ApiError::Extraction(e.to_string())
    }
}
