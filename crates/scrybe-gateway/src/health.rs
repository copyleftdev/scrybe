//! Health check endpoints for liveness and readiness probes.

use axum::http::StatusCode;

/// Liveness probe - always returns OK if the process is running.
///
/// This endpoint is used by Kubernetes/orchestrators to determine
/// if the process should be restarted.
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe - checks if the service is ready to accept traffic.
///
/// This would typically check database connections, cache availability,
/// etc. For now, it returns OK immediately.
///
/// # Returns
///
/// - `200 OK`: Service is ready
/// - `503 Service Unavailable`: Service is not ready
pub async fn readiness_check() -> StatusCode {
    // TODO: Add actual readiness checks
    // - Redis connectivity
    // - ClickHouse connectivity
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let status = health_check().await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_check_returns_ok() {
        let status = readiness_check().await;
        assert_eq!(status, StatusCode::OK);
    }
}
