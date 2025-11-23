//! Nonce validation for replay attack prevention.

use crate::client::RedisClient;
use redis::AsyncCommands;
use scrybe_core::ScrybeError;

/// Nonce validator for replay attack prevention.
///
/// Stores nonces in Redis with 5-minute TTL. Each nonce can only be used once.
pub struct NonceValidator {
    client: RedisClient,
    ttl_seconds: usize,
}

impl NonceValidator {
    /// Create a new nonce validator.
    ///
    /// # Arguments
    ///
    /// * `client` - Redis client
    /// * `ttl_seconds` - TTL for nonces (default: 300 = 5 minutes)
    pub fn new(client: RedisClient, ttl_seconds: Option<usize>) -> Self {
        Self {
            client,
            ttl_seconds: ttl_seconds.unwrap_or(300),
        }
    }

    /// Validate a nonce (check if it's new and mark as used).
    ///
    /// Returns `true` if nonce is valid (never seen before).
    /// Returns `false` if nonce was already used (replay attack).
    ///
    /// # Arguments
    ///
    /// * `nonce` - Nonce string to validate
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if Redis operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use scrybe_cache::{RedisClient, NonceValidator};
    /// # async fn example(client: RedisClient) -> Result<(), scrybe_core::ScrybeError> {
    /// let validator = NonceValidator::new(client, None);
    ///
    /// if validator.validate_nonce("unique-nonce-123").await? {
    ///     println!("Valid nonce");
    /// } else {
    ///     println!("Replay attack detected!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_nonce(&self, nonce: &str) -> Result<bool, ScrybeError> {
        let key = format!("nonce:{}", nonce);
        let mut conn = self.client.get_connection().await?;

        // Try to set the key with NX (only if not exists) and EX (expiry)
        let result: Option<String> = conn
            .set_nx(&key, "1")
            .await
            .map_err(|e| ScrybeError::cache_error("nonce", format!("SET NX failed: {}", e)))?;

        // If SET NX succeeded, set the TTL
        if result.is_some() {
            conn.expire::<_, ()>(&key, self.ttl_seconds as i64)
                .await
                .map_err(|e| ScrybeError::cache_error("nonce", format!("EXPIRE failed: {}", e)))?;
            Ok(true) // Nonce is valid (new)
        } else {
            Ok(false) // Nonce already exists (replay attack)
        }
    }

    /// Check if a nonce exists (without marking as used).
    ///
    /// # Arguments
    ///
    /// * `nonce` - Nonce to check
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if Redis operation fails.
    pub async fn exists(&self, nonce: &str) -> Result<bool, ScrybeError> {
        let key = format!("nonce:{}", nonce);
        let mut conn = self.client.get_connection().await?;

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| ScrybeError::cache_error("nonce", format!("EXISTS failed: {}", e)))?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_nonce_validator_compiles() {
        // Placeholder - requires Redis for full testing
        assert!(true);
    }
}
