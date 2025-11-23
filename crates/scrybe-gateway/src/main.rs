//! # Scrybe Ingestion Gateway
//!
//! HTTP gateway for receiving browser session data using Axum.
//!
//! ## Features
//!
//! - HMAC-SHA256 authentication
//! - Rate limiting
//! - Health check endpoints
//! - Graceful shutdown
//!
//! ## TigerStyle Compliance
//!
//! - No unwrap/panic in production code
//! - Explicit error handling
//! - Bounded request sizes

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]

mod health;
mod middleware;
mod routes;
mod shutdown;

use axum::{routing::get, Router};
use routes::ingest::AppState;
use scrybe_core::{Config, ScrybeError};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), ScrybeError> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Scrybe Gateway...");

    // Load configuration
    let config = Config::from_env()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    info!("Gateway listening on {}", addr);

    // Create application state
    let state = Arc::new(AppState::new());

    // Build router with all routes and middleware
    let app = Router::new()
        // Health check routes (no authentication required)
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        // API routes (with authentication and rate limiting)
        .merge(routes::ingest_route())
        // Global middleware
        .layer(axum::middleware::from_fn(middleware::security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Create server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ScrybeError::io_error("bind", e.to_string()))?;

    info!("API endpoints:");
    info!("  GET  /health - Liveness probe");
    info!("  GET  /health/ready - Readiness probe");
    info!("  POST /api/v1/ingest - Ingest browser telemetry");

    info!("Gateway ready to accept connections");
    info!("Security: HMAC-SHA256 authentication enabled");
    info!("Rate limit: 100 requests/minute per IP");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
        .map_err(|e| ScrybeError::io_error("serve", e.to_string()))?;

    info!("Gateway shutdown complete");

    Ok(())
}
