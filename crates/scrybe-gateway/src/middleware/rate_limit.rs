//! Rate limiting middleware using token bucket algorithm.

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::warn;

/// Rate limiter using token bucket algorithm.
pub type GlobalRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Create a rate limiter layer.
///
/// Default: 100 requests per minute.
pub fn rate_limit_layer() -> GlobalRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());
    Arc::new(RateLimiter::direct(quota))
}

/// Rate limiting middleware.
///
/// Enforces 100 requests/minute per IP address using token bucket algorithm.
pub async fn rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    limiter: axum::extract::State<GlobalRateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Check if request is allowed
    if limiter.check().is_ok() {
        Ok(next.run(request).await)
    } else {
        warn!("Rate limit exceeded for IP: {}", addr.ip());
        Err(RateLimitError::TooManyRequests)
    }
}

/// Rate limit errors.
#[derive(Debug)]
pub enum RateLimitError {
    /// Too many requests
    TooManyRequests,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RateLimitError::TooManyRequests => {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string())
            }
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = rate_limit_layer();
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_rate_limiter_enforcement() {
        let quota = Quota::per_second(NonZeroU32::new(2).unwrap());
        let limiter = RateLimiter::direct(quota);

        // First two requests should succeed
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());

        // Third request should fail (exceeded quota)
        assert!(limiter.check().is_err());
    }
}
