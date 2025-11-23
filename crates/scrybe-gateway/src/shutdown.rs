//! Graceful shutdown handling.

use tokio::signal;
use tracing::info;

/// Wait for shutdown signal (SIGTERM or Ctrl-C).
///
/// This function will block until one of the following signals is received:
/// - SIGTERM (typical for Docker/Kubernetes)
/// - SIGINT (Ctrl-C)
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl-C signal");
        },
        _ = terminate => {
            info!("Received SIGTERM signal");
        },
    }

    info!("Starting graceful shutdown...");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_shutdown_signal_does_not_complete_immediately() {
        // This test verifies that shutdown_signal() is a future that
        // waits for a signal, rather than completing immediately
        let result = timeout(Duration::from_millis(100), shutdown_signal()).await;
        assert!(
            result.is_err(),
            "shutdown_signal should not complete without a signal"
        );
    }
}
