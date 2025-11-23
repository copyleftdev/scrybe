//! Session cache management with Redis.

use crate::client::RedisClient;
use redis::AsyncCommands;
use scrybe_core::{
    types::{Session, SessionId},
    ScrybeError,
};

/// Redis-backed session cache with TTL.
///
/// Sessions are stored for 1 hour (3600 seconds) to minimize memory usage.
pub struct SessionCache {
    client: RedisClient,
    ttl_seconds: usize,
}

impl SessionCache {
    /// Create a new session cache.
    ///
    /// # Arguments
    ///
    /// * `client` - Redis client instance
    /// * `ttl_seconds` - Time-to-live for sessions (default: 3600 = 1 hour)
    pub fn new(client: RedisClient, ttl_seconds: Option<usize>) -> Self {
        Self {
            client,
            ttl_seconds: ttl_seconds.unwrap_or(3600),
        }
    }

    /// Store a session in the cache with TTL.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if the operation fails.
    pub async fn store(&self, session: &Session) -> Result<(), ScrybeError> {
        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(session).map_err(|e| {
            ScrybeError::cache_error("redis", format!("Serialization failed: {}", e))
        })?;

        let mut conn = self.client.get_connection().await?;

        conn.set_ex::<_, _, ()>(&key, &value, self.ttl_seconds as u64)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("SET failed: {}", e)))?;

        Ok(())
    }

    /// Retrieve a session from the cache.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if the operation fails.
    pub async fn get(&self, session_id: &SessionId) -> Result<Option<Session>, ScrybeError> {
        let key = format!("session:{}", session_id);

        let mut conn = self.client.get_connection().await?;

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("GET failed: {}", e)))?;

        match value {
            Some(json) => {
                let session = serde_json::from_str(&json).map_err(|e| {
                    ScrybeError::cache_error("redis", format!("Deserialization failed: {}", e))
                })?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Delete a session from the cache.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if the operation fails.
    pub async fn delete(&self, session_id: &SessionId) -> Result<(), ScrybeError> {
        let key = format!("session:{}", session_id);

        let mut conn = self.client.get_connection().await?;

        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("DEL failed: {}", e)))?;

        Ok(())
    }

    /// Check if a session exists in the cache.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if the operation fails.
    pub async fn exists(&self, session_id: &SessionId) -> Result<bool, ScrybeError> {
        let key = format!("session:{}", session_id);

        let mut conn = self.client.get_connection().await?;

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("EXISTS failed: {}", e)))?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_session_cache_compiles() {
        // Placeholder test
        assert!(true);
    }
}
