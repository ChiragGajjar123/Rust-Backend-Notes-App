mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

use crate::config::Config;
use crate::db::{create_pool, run_migrations};
use crate::middleware::auth::AppState;
use crate::middleware::cors::build_cors_layer;
use crate::routes::all_routes;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file in local development if present
    dotenvy::dotenv().ok();

    // Initialize structured logging / tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Notes App Backend (Pure Rust Async Web Service)...");

    // Parse runtime configuration from environment variables
    let config = Arc::new(Config::from_env().map_err(|e| anyhow::anyhow!(e))?);

    tracing::info!(
        "Configuration loaded successfully. Host: {}, Port: {}, Max DB Connections: {}",
        config.server_host,
        config.server_port,
        config.max_connections
    );

    // Initialize high-concurrency PostgreSQL connection pool
    tracing::info!("Connecting to PostgreSQL database...");
    let pool = create_pool(&config.database_url, config.max_connections).await?;
    tracing::info!("Database pool created successfully.");

    // Run embedded SQL database migrations automatically on startup
    tracing::info!("Running database migrations...");
    run_migrations(&pool).await?;
    tracing::info!("Database migrations executed successfully.");

    // Create shared application state
    let state = AppState::new(config.clone(), Arc::new(pool));

    // Build CORS middleware
    let cors = build_cors_layer(&config);

    // Build application HTTP router
    let app = all_routes()
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Bind server listener to assigned host and port (Render compatible)
    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}