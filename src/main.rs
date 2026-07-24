mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;


use crate::config::Config;
use crate::db::{create_pool, run_migrations};
use crate::middleware::auth::AppState;
use crate::routes::all_routes;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Arc::new(Config::from_env()?);

    // Create database connection pool
    let pool = create_pool(&config.database_url, config.max_connections).await?;
    
    // Run database migrations
    run_migrations(&pool).await?;

    // Create application state
    let state = AppState::new(config.clone(), Arc::new(pool));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);

    // Build application router
    let app = all_routes()
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}