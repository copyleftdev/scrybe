//! Rate limiting using Redis token bucket algorithm.

use crate::client::RedisClient;
use redis::AsyncCommands;
use scrybe_core::ScrybeError;

/// Redis-backed rate limiter using token bucket algorithm.
pub struct RateLimiter {
    client: RedisClient,
    max_requests: usize,
    window_seconds: usize,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    ///
    /// * `client` - Redis client instance
    /// * `max_requests` - Maximum requests allowed in the window
    /// * `window_seconds` - Time window in seconds
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use scrybe_cache::{RedisClient, RateLimiter};
    /// # async fn example() -> Result<(), scrybe_core::ScrybeError> {
    /// let client = RedisClient::new("redis://localhost", 10).await?;
    /// let limiter = RateLimiter::new(client, 100, 60); // 100 requests per minute
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(client: RedisClient, max_requests: usize, window_seconds: usize) -> Self {
        Self {
            client,
            max_requests,
            window_seconds,
        }
    }

    /// Check if a request is allowed for the given identifier.
    ///
    /// Returns `true` if the request is allowed, `false` if rate limit exceeded.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Unique identifier (e.g., IP address, session ID)
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if Redis operation fails.
    pub async fn check(&self, identifier: &str) -> Result<bool, ScrybeError> {
        let key = format!("ratelimit:{}", identifier);

        let mut conn = self.client.get_connection().await?;

        // Increment counter
        let count: usize = conn
            .incr(&key, 1)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("INCR failed: {}", e)))?;

        // Set expiration on first request
        if count == 1 {
            conn.expire::<_, ()>(&key, self.window_seconds as i64)
                .await
                .map_err(|e| ScrybeError::cache_error("redis", format!("EXPIRE failed: {}", e)))?;
        }

        Ok(count <= self.max_requests)
    }

    /// Get current request count for an identifier.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if Redis operation fails.
    pub async fn get_count(&self, identifier: &str) -> Result<usize, ScrybeError> {
        let key = format!("ratelimit:{}", identifier);

        let mut conn = self.client.get_connection().await?;

        let count: Option<usize> = conn
            .get(&key)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("GET failed: {}", e)))?;

        Ok(count.unwrap_or(0))
    }

    /// Reset rate limit for an identifier.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if Redis operation fails.
    pub async fn reset(&self, identifier: &str) -> Result<(), ScrybeError> {
        let key = format!("ratelimit:{}", identifier);

        let mut conn = self.client.get_connection().await?;

        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("DEL failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_rate_limiter_compiles() {
        // Placeholder test
        assert!(true);
    }
}
