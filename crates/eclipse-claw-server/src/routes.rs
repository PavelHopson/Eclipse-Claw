use axum::{
    Router,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

use crate::handlers;
use crate::state::AppState;

#[derive(Clone, Default)]
struct AuthToken(Option<String>);

pub fn build(
    state: AppState,
    body_limit: usize,
    max_concurrency: usize,
    server_token: Option<String>,
) -> Router {
    let protected = Router::new()
        .route("/connectors", get(handlers::connectors))
        .route("/connectors/doctor", get(handlers::connector_doctor))
        .route("/extract", post(handlers::extract_url))
        .route("/extract/html", post(handlers::extract_html))
        .route("/summarise", post(handlers::summarise_url))
        .route("/batch", post(handlers::batch_extract))
        .route("/design-tokens", post(handlers::design_tokens))
        .with_state(state)
        .layer(middleware::from_fn_with_state(
            AuthToken(server_token),
            authenticate,
        ));

    Router::new()
        .route("/health", get(handlers::health))
        .merge(protected)
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(ConcurrencyLimitLayer::new(max_concurrency.max(1)))
        .layer(TraceLayer::new_for_http())
}

async fn authenticate(
    State(expected): State<AuthToken>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(expected) = expected.0.as_deref() else {
        return Ok(next.run(request).await);
    };

    let supplied = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if supplied.is_some_and(|value| constant_time_eq(value.as_bytes(), expected.as_bytes())) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod auth_tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    #[test]
    fn compares_tokens_without_early_character_exit() {
        assert!(constant_time_eq(b"same-token", b"same-token"));
        assert!(!constant_time_eq(b"same-token", b"same-tokem"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    fn protected_test_router(token: Option<String>) -> Router {
        Router::new()
            .route("/protected", get(|| async { StatusCode::OK }))
            .layer(middleware::from_fn_with_state(
                AuthToken(token),
                authenticate,
            ))
    }

    #[tokio::test]
    async fn bearer_auth_is_fail_closed_when_configured() {
        let app = protected_test_router(Some("0123456789abcdef0123456789abcdef".into()));
        let response = app
            .oneshot(Request::get("/protected").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_auth_accepts_exact_token() {
        let app = protected_test_router(Some("0123456789abcdef0123456789abcdef".into()));
        let response = app
            .oneshot(
                Request::get("/protected")
                    .header(
                        header::AUTHORIZATION,
                        "Bearer 0123456789abcdef0123456789abcdef",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
