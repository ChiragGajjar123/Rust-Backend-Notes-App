mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

use crate::config::Config;
use crate::db::create_pool_lazy;
use crate::middleware::auth::AppState;
use crate::middleware::cors::build_cors_layer;
use crate::routes::all_routes;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared initialization: tracing, config, DB pool, and Axum router.
///
/// Returns the fully assembled Axum `Router` and `Config`.
async fn build_app() -> anyhow::Result<(axum::Router, Arc<Config>)> {
    // Load .env file in local development if present
    dotenvy::dotenv().ok();

    // Initialize structured logging / tracing (try_init prevents panics in Lambda runtime)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .ok();

    tracing::info!("Starting Notes App Backend...");

    // Parse runtime configuration from environment variables
    let config = Arc::new(Config::from_env().map_err(|e| anyhow::anyhow!(e))?);

    tracing::info!(
        "Configuration loaded successfully. Max DB Connections: {}",
        config.max_connections
    );

    // Initialize lazy PostgreSQL connection pool for instant 0ms cold starts
    tracing::info!("Initializing lazy PostgreSQL connection pool...");
    let pool = create_pool_lazy(&config.database_url, config.max_connections)?;

    // Create shared application state
    let state = AppState::new(config.clone(), Arc::new(pool));

    // Build application HTTP router with CORS & tracing middleware
    let cors = build_cors_layer(&config);
    let app = all_routes()
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    Ok((app, config))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (app, config) = build_app().await?;

    // Dynamically detect runtime execution environment:
    // AWS Lambda automatically injects AWS_LAMBDA_RUNTIME_API into the container environment.
    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        use lambda_http::tower::ServiceExt;

        tracing::info!("🚀 Running in AWS Lambda mode — awaiting API Gateway events.");

        lambda_http::run(lambda_http::tower::service_fn(move |req: lambda_http::Request| {
            let app = app.clone();
            async move {
                let (parts, body) = req.into_parts();
                let axum_body = match body {
                    lambda_http::Body::Empty => axum::body::Body::empty(),
                    lambda_http::Body::Text(text) => axum::body::Body::from(text),
                    lambda_http::Body::Binary(bytes) => axum::body::Body::from(bytes),
                };
                let axum_req = axum::http::Request::from_parts(parts, axum_body);
                let response = app.oneshot(axum_req).await.map_err(|e| e.to_string())?;
                Ok::<_, String>(response)
            }
        }))
        .await
        .map_err(|e| anyhow::anyhow!("Lambda runtime error: {}", e))?;
    } else {
        tracing::info!("🚀 Running in Standalone mode — binding listener.");

        let addr: std::net::SocketAddr =
            format!("{}:{}", config.server_host, config.server_port).parse()?;
        tracing::info!("🚀 Server running on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}