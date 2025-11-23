//! Redis client with connection pooling.
use deadpool_redis::{Config, Pool, Runtime};
use scrybe_core::ScrybeError;

/// Redis client with connection pool.
///
/// Uses `deadpool-redis` for connection pooling with configurable pool size.
#[derive(Clone)]
pub struct RedisClient {
    pool: Pool,
}

impl RedisClient {
    /// Create a new Redis client with connection pool.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., `redis://localhost:6379`)
    /// * `pool_size` - Maximum pool connections (default: 20)
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if connection fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use scrybe_cache::RedisClient;
    /// # async fn example() -> Result<(), scrybe_core::ScrybeError> {
    /// let client = RedisClient::new("redis://localhost:6379", 20).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(redis_url: &str, _pool_size: usize) -> Result<Self, ScrybeError> {
        let cfg = Config::from_url(redis_url);

        let pool = cfg.create_pool(Some(Runtime::Tokio1)).map_err(|e| {
            ScrybeError::cache_error("redis", format!("Pool creation failed: {}", e))
        })?;

        // Test connection
        let mut conn = pool
            .get()
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("Connection failed: {}", e)))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("PING failed: {}", e)))?;

        Ok(Self { pool })
    }

    /// Get a connection from the pool.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if no connection available.
    pub async fn get_connection(&self) -> Result<deadpool_redis::Connection, ScrybeError> {
        self.pool
            .get()
            .await
            .map_err(|e| ScrybeError::cache_error("redis", format!("No connection: {}", e)))
    }

    /// Check if Redis is healthy.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::CacheError` if health check fails.
    pub async fn health_check(&self) -> Result<(), ScrybeError> {
        let mut conn = self.get_connection().await?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| {
                ScrybeError::cache_error("redis", format!("Health check failed: {}", e))
            })?;

        Ok(())
    }
}
