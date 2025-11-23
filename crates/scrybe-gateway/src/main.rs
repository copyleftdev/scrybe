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
mod shutdown;

use axum::{routing::get, Router};
use scrybe_core::{Config, ScrybeError};
use std::net::SocketAddr;
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

    // Build router
    let app = Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .layer(TraceLayer::new_for_http());

    // Create server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ScrybeError::io_error("bind", e.to_string()))?;

    info!("Gateway ready to accept connections");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
        .map_err(|e| ScrybeError::io_error("serve", e.to_string()))?;

    info!("Gateway shutdown complete");

    Ok(())
}
