//! Application state shared across handlers.

use scrybe_cache::{NonceValidator, RedisClient};
use std::sync::Arc;

/// Shared application state.
///
/// Contains Redis client and nonce validator for authentication.
#[derive(Clone)]
#[allow(dead_code)] // Ready to use in routes
pub struct AppState {
    /// Redis client for caching
    pub redis_client: Arc<RedisClient>,
    /// Nonce validator for replay attack prevention
    pub nonce_validator: Arc<NonceValidator>,
}

impl AppState {
    /// Create new application state.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `pool_size` - Redis connection pool size
    /// * `nonce_ttl` - Nonce TTL in seconds (default: 300 = 5 minutes)
    ///
    /// # Errors
    ///
    /// Returns error if Redis connection fails.
    #[allow(dead_code)] // Ready for use
    pub async fn new(
        redis_url: &str,
        pool_size: usize,
        nonce_ttl: Option<usize>,
    ) -> Result<Self, scrybe_core::ScrybeError> {
        let redis_client = RedisClient::new(redis_url, pool_size).await?;
        let nonce_validator = NonceValidator::new(redis_client.clone(), nonce_ttl);

        Ok(Self {
            redis_client: Arc::new(redis_client),
            nonce_validator: Arc::new(nonce_validator),
        })
    }

    /// Check if Redis is healthy.
    #[allow(dead_code)] // Ready for use
    pub async fn health_check(&self) -> Result<(), scrybe_core::ScrybeError> {
        self.redis_client.health_check().await
    }
}
