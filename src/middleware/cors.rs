use crate::config::Config;
use axum::{
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn cors_layer(config: &Arc<Config>) -> CorsLayer {
    let allowed_origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(if config.cors_allowed_origins.contains(&"*".to_string()) {
            AllowOrigin::any()
        } else if allowed_origins.is_empty() {
            AllowOrigin::any()
        } else {
            AllowOrigin::list(allowed_origins)
        })
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

pub fn health_check() -> impl IntoResponse {
    "OK"
}

pub fn create_health_router() -> Router {
    Router::new().route("/health", get(health_check))
}