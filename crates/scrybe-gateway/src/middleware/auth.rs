//! HMAC-SHA256 authentication middleware.
//!
//! Ready for integration - currently not wired pending complete testing.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::{debug, warn};

#[allow(dead_code)]
type HmacSha256 = Hmac<Sha256>;

/// HMAC authentication middleware.
///
/// Validates requests using HMAC-SHA256 signatures with the following headers:
/// - `X-Scrybe-Timestamp`: Unix timestamp in milliseconds
/// - `X-Scrybe-Nonce`: UUID v4 for replay protection
/// - `X-Scrybe-Signature`: HMAC-SHA256 hex string
///
/// The signature is computed over: `{timestamp}:{nonce}:{body}`
/// HMAC authentication middleware with nonce validation.
///
/// Validates HMAC signatures and checks nonce uniqueness via Redis.
#[allow(dead_code)] // Ready to wire into routes
pub async fn hmac_auth(
    State(state): State<Arc<crate::state::AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    debug!("Validating HMAC authentication");

    // Extract headers
    let timestamp = extract_header(&headers, "x-scrybe-timestamp")?;
    let nonce = extract_header(&headers, "x-scrybe-nonce")?;
    let provided_signature = extract_header(&headers, "x-scrybe-signature")?;

    // Validate timestamp (must be within 5 minutes)
    validate_timestamp(&timestamp)?;

    // Validate nonce for replay protection
    let nonce_valid = state
        .nonce_validator
        .validate_nonce(&nonce)
        .await
        .map_err(|_| AuthError::InvalidNonce)?;

    if !nonce_valid {
        warn!("Replay attack detected: nonce already used");
        return Err(AuthError::ReplayAttack);
    }

    // Read body for signature verification
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| AuthError::InvalidSignature)?;

    // Compute expected signature
    let message = format!(
        "{}:{}:{}",
        timestamp,
        nonce,
        String::from_utf8_lossy(&body_bytes)
    );
    let hmac_key = get_hmac_key();
    let expected_signature = compute_signature(&message, &hmac_key)?;

    // Constant-time comparison (prevents timing attacks)
    if bool::from(
        expected_signature
            .as_bytes()
            .ct_eq(provided_signature.as_bytes()),
    ) {
        debug!("HMAC authentication successful");

        // Restore body for downstream handlers
        let request = Request::from_parts(parts, Body::from(body_bytes));

        Ok(next.run(request).await)
    } else {
        warn!("HMAC authentication failed: signature mismatch");
        Err(AuthError::InvalidSignature)
    }
}

/// Extract a header value.
fn extract_header(headers: &HeaderMap, name: &str) -> Result<String, AuthError> {
    headers
        .get(name)
        .ok_or(AuthError::MissingHeader(name.to_string()))?
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| AuthError::InvalidHeader(name.to_string()))
}

/// Validate timestamp is within 5 minutes.
fn validate_timestamp(timestamp_str: &str) -> Result<(), AuthError> {
    let timestamp_ms: i64 = timestamp_str
        .parse()
        .map_err(|_| AuthError::InvalidTimestamp)?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let diff_ms = (now_ms - timestamp_ms).abs();

    const FIVE_MINUTES_MS: i64 = 5 * 60 * 1000;

    if diff_ms > FIVE_MINUTES_MS {
        Err(AuthError::TimestampExpired)
    } else {
        Ok(())
    }
}

/// Compute HMAC-SHA256 signature.
fn compute_signature(message: &str, key: &[u8]) -> Result<String, AuthError> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| AuthError::InvalidKey)?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Get HMAC key from environment.
///
/// TODO: Load from SecretConfig instead of environment directly.
fn get_hmac_key() -> Vec<u8> {
    std::env::var("SCRYBE_HMAC_KEY")
        .ok()
        .and_then(|k| hex::decode(k).ok())
        .unwrap_or_else(|| b"development-key-do-not-use-in-production".to_vec())
}

/// Authentication errors.
#[derive(Debug)]
pub enum AuthError {
    /// Missing or invalid header
    MissingHeader(String),
    /// Invalid timestamp (too old or future)
    InvalidTimestamp,
    /// Invalid HMAC signature
    InvalidSignature,
    /// Invalid nonce (cache error)
    InvalidNonce,
    /// Replay attack detected (nonce reused)
    ReplayAttack,
    /// Timestamp expired (> 5 minutes)
    TimestampExpired,
    /// Invalid HMAC key
    InvalidKey,
    /// Invalid header
    InvalidHeader(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingHeader(header) => (
                StatusCode::BAD_REQUEST,
                format!("Missing header: {}", header),
            ),
            AuthError::InvalidTimestamp => {
                (StatusCode::UNAUTHORIZED, "Invalid timestamp".to_string())
            }
            AuthError::InvalidSignature => {
                (StatusCode::UNAUTHORIZED, "Invalid signature".to_string())
            }
            AuthError::InvalidNonce => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Nonce validation failed".to_string(),
            ),
            AuthError::ReplayAttack => (StatusCode::CONFLICT, "Replay attack detected".to_string()),
            AuthError::TimestampExpired => {
                (StatusCode::UNAUTHORIZED, "Timestamp expired".to_string())
            }
            AuthError::InvalidKey => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
            ),
            AuthError::InvalidHeader(header) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid header: {}", header),
            ),
        };

        warn!("Authentication error: {}", message);

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_signature() {
        let message = "1234567890:test-nonce:body";
        let key = b"test-key";
        let signature = compute_signature(message, key);
        assert!(signature.is_ok());
        assert_eq!(signature.unwrap().len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_validate_timestamp_current() {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let result = validate_timestamp(&now_ms.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_timestamp_expired() {
        let old_ms = chrono::Utc::now().timestamp_millis() - (10 * 60 * 1000); // 10 minutes ago
        let result = validate_timestamp(&old_ms.to_string());
        assert!(matches!(result, Err(AuthError::TimestampExpired)));
    }
}
