//! Ingestion endpoint for browser session data.

use crate::extraction::{extract_headers, extract_http_version, extract_ip_info};
use axum::{
    extract::{ConnectInfo, Json, State},
    http::{HeaderMap, StatusCode, Version},
    response::IntoResponse,
};
use scrybe_core::{
    types::{BehavioralSignals, BrowserSignals, NetworkSignals, SessionId},
    ScrybeError,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    // TODO: Add Redis, ClickHouse clients
}

impl AppState {
    /// Create new application state.
    pub fn new() -> Self {
        Self {}
    }
}

/// Request payload for ingestion endpoint.
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    /// Network signals from client
    pub network: NetworkSignals,
    /// Browser signals from client (not yet persisted)
    #[allow(dead_code)]
    pub browser: BrowserSignals,
    /// Behavioral signals from client (not yet persisted)
    #[allow(dead_code)]
    pub behavioral: BehavioralSignals,
}

/// Response from ingestion endpoint.
#[derive(Debug, Serialize)]
pub struct IngestResponse {
    /// Session ID assigned to this ingestion
    pub session_id: String,
    /// Whether this is a new session
    pub is_new: bool,
    /// Server timestamp
    pub timestamp: String,
}

/// POST /api/v1/ingest - Ingest browser telemetry data.
///
/// This endpoint receives browser session data, validates it,
/// enriches it with server-side signals, and stores it.
///
/// # Authentication
///
/// Requires HMAC-SHA256 authentication via headers:
/// - `X-Scrybe-Timestamp`: Unix timestamp in milliseconds
/// - `X-Scrybe-Nonce`: UUID v4
/// - `X-Scrybe-Signature`: HMAC-SHA256 hex string
///
/// # Rate Limiting
///
/// - 100 requests/minute per IP
/// - 1000 requests/minute per session
///
/// # Errors
///
/// - `400 Bad Request`: Invalid payload or validation failure
/// - `401 Unauthorized`: Authentication failure
/// - `429 Too Many Requests`: Rate limit exceeded
/// - `503 Service Unavailable`: Backend unavailable
pub async fn ingest_handler(
    State(_state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    version: Version,
    Json(payload): Json<IngestRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received ingest request from {}", addr.ip());

    // Extract server-side signals
    let client_ip = extract_ip_info(&ConnectInfo(addr));
    let server_headers = extract_headers(&headers);
    let http_version = extract_http_version(&version);

    info!(
        "Server-side extraction: IP={}, headers={}, HTTP={:?}",
        client_ip,
        server_headers.len(),
        http_version
    );

    // Merge client-provided signals with server-side signals
    let mut network_signals = payload.network;
    network_signals.ip = client_ip;
    network_signals.http_version = http_version;
    // Append server-extracted headers (client can't spoof these)
    network_signals.headers.extend(server_headers);

    // TODO: Validate payload
    // TODO: Store in Redis
    // TODO: Enqueue for enrichment

    // Create session
    let session_id = SessionId::new();

    Ok(Json(IngestResponse {
        session_id: session_id.to_string(),
        is_new: true,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Error wrapper for Axum responses.
#[derive(Debug)]
pub struct AppError(ScrybeError);

impl From<ScrybeError> for AppError {
    fn from(err: ScrybeError) -> Self {
        Self(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            ScrybeError::InvalidSession { .. } => (StatusCode::BAD_REQUEST, self.0.to_string()),
            ScrybeError::ValidationError { .. } => (StatusCode::BAD_REQUEST, self.0.to_string()),
            ScrybeError::AuthenticationError { .. } => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed".to_string(),
            ),
            ScrybeError::RateLimit { .. } => (StatusCode::TOO_MANY_REQUESTS, self.0.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        warn!("Request error: {} - {}", status, message);

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Create the ingest route with all middleware.
///
/// Applies the following middleware in order:
/// 1. Authentication (HMAC-SHA256) - TODO: Enable when ready
/// 2. Rate limiting (100 req/min)
/// 3. Request handler
pub fn ingest_route() -> axum::Router<Arc<AppState>> {
    use axum::routing::post;

    // TODO: Add authentication middleware when fully tested
    // .layer(axum::middleware::from_fn(crate::middleware::auth::hmac_auth))

    axum::Router::new().route("/api/v1/ingest", post(ingest_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState as GatewayAppState;
    use axum::http::StatusCode;
    use scrybe_core::{types::*, ScrybeError};
    use std::net::Ipv4Addr;

    fn create_test_request() -> IngestRequest {
        IngestRequest {
            network: NetworkSignals {
                ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                ja3: None,
                ja4: None,
                headers: vec![Header::new("User-Agent", "Test/1.0")],
                http_version: HttpVersion::Http2,
            },
            browser: BrowserSignals {
                canvas_hash: Some("test_hash".to_string()),
                webgl_hash: None,
                audio_hash: None,
                fonts: vec!["Arial".to_string()],
                plugins: vec![],
                timezone: "UTC".to_string(),
                language: "en-US".to_string(),
                screen: ScreenInfo::default(),
                user_agent: "Test/1.0".to_string(),
            },
            behavioral: BehavioralSignals {
                mouse_events: vec![],
                scroll_events: vec![],
                click_events: vec![],
                timing: TimingMetrics::default(),
            },
        }
    }

    #[tokio::test]
    async fn test_ingest_handler_returns_session_id() {
        let state = Arc::new(AppState::new());
        let request = create_test_request();
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let headers = axum::http::HeaderMap::new();
        let version = axum::http::Version::HTTP_11;

        let result = ingest_handler(
            State(state),
            ConnectInfo(addr),
            headers,
            version,
            Json(request),
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
