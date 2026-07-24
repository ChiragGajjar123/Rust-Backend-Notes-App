mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

use crate::config::Config;
#[allow(unused_imports)]
use crate::db::{create_pool, create_pool_lazy, run_migrations};
use crate::middleware::auth::AppState;
use crate::routes::all_routes;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared initialization: tracing, config, DB pool, migrations, and Axum router.
///
/// Returns the fully assembled Axum `Router` ready to serve requests.
/// This is used by both the standalone server and the Lambda entry point.
async fn build_app() -> anyhow::Result<axum::Router> {
    // Load .env file in local development if present
    dotenvy::dotenv().ok();

    // Initialize structured logging / tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Notes App Backend...");

    // Parse runtime configuration from environment variables
    let config = Arc::new(Config::from_env().map_err(|e| anyhow::anyhow!(e))?);

    tracing::info!(
        "Configuration loaded successfully. Max DB Connections: {}",
        config.max_connections
    );

    // In standalone mode, connect to database and run migrations on startup.
    // In Lambda mode, use connect_lazy for instant 0ms cold-start initialization.
    #[cfg(feature = "standalone")]
    let pool = {
        tracing::info!("Connecting to PostgreSQL database...");
        let pool = create_pool(&config.database_url, config.max_connections).await?;
        tracing::info!("Database pool created successfully.");

        tracing::info!("Running database migrations...");
        run_migrations(&pool).await?;
        tracing::info!("Database migrations executed successfully.");
        pool
    };

    #[cfg(feature = "lambda")]
    let pool = {
        tracing::info!("Initializing lazy PostgreSQL connection pool for Lambda...");
        create_pool_lazy(&config.database_url, config.max_connections)?
    };

    // Create shared application state
    let state = AppState::new(config.clone(), Arc::new(pool));

    // Build application HTTP router with tracing
    #[allow(unused_mut)]
    let mut app = all_routes()
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // In standalone mode, apply tower-http CORS middleware.
    // In Lambda mode, CORS is handled at the API Gateway level (enterprise standard).
    #[cfg(feature = "standalone")]
    {
        let cors = crate::middleware::cors::build_cors_layer(&config);
        app = app.layer(cors);
        tracing::info!("CORS middleware applied (standalone mode).");
    }

    #[cfg(feature = "lambda")]
    {
        tracing::info!("Running in Lambda mode — CORS handled by API Gateway.");
    }

    Ok(app)
}

// ─── Standalone entry point (EC2 / Docker / local dev) ──────────────────────
#[cfg(feature = "standalone")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = build_app().await?;

    let config = Arc::new(Config::from_env().map_err(|e| anyhow::anyhow!(e))?);
    let addr: std::net::SocketAddr =
        format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ─── Lambda entry point (API Gateway + Lambda) ─────────────────────────────
#[cfg(feature = "lambda")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = build_app().await?;

    tracing::info!("🚀 Lambda handler ready — awaiting API Gateway events.");
    lambda_http::run(app).await.map_err(|e| anyhow::anyhow!("Lambda runtime error: {}", e))?;

    Ok(())
}