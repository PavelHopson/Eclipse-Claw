use axum::{Router, routing::{get, post}};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

use crate::handlers;
use crate::state::AppState;

pub fn build(state: AppState, body_limit: usize) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(handlers::health))
        .route("/extract", post(handlers::extract_url))
        .route("/extract/html", post(handlers::extract_html))
        .route("/summarise", post(handlers::summarise_url))
        .route("/batch", post(handlers::batch_extract))
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
