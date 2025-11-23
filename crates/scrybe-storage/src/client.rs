//! ClickHouse client with connection pooling.

use clickhouse::Client;
use scrybe_core::ScrybeError;

/// ClickHouse client for session storage.
///
/// Provides connection pooling and health checks for ClickHouse database.
#[derive(Clone)]
pub struct ClickHouseClient {
    client: Client,
}

impl ClickHouseClient {
    /// Create a new ClickHouse client.
    ///
    /// # Arguments
    ///
    /// * `url` - ClickHouse server URL (e.g., `http://localhost:8123`)
    /// * `database` - Database name (default: "scrybe")
    /// * `username` - Username (default: "default")
    /// * `password` - Password
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if connection fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use scrybe_storage::ClickHouseClient;
    /// # async fn example() -> Result<(), scrybe_core::ScrybeError> {
    /// let client = ClickHouseClient::new(
    ///     "http://localhost:8123",
    ///     "scrybe",
    ///     "default",
    ///     ""
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        url: &str,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ScrybeError> {
        let client = Client::default()
            .with_url(url)
            .with_database(database)
            .with_user(username)
            .with_password(password);

        // Test connection with PING
        client.query("SELECT 1").execute().await.map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Connection failed: {}", e))
        })?;

        Ok(Self { client })
    }

    /// Get the underlying ClickHouse client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Check if ClickHouse is healthy.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if health check fails.
    pub async fn health_check(&self) -> Result<(), ScrybeError> {
        self.client.query("SELECT 1").execute().await.map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Health check failed: {}", e))
        })?;

        Ok(())
    }

    /// Initialize database schema.
    ///
    /// Creates the sessions table if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if schema creation fails.
    pub async fn init_schema(&self) -> Result<(), ScrybeError> {
        let schema = r#"
            CREATE TABLE IF NOT EXISTS sessions (
                session_id UUID,
                timestamp DateTime64(3, 'UTC'),
                fingerprint_hash String,
                ip String,
                user_agent String,
                network_signals String,
                browser_signals String,
                behavioral_signals String,
                bot_probability Float32,
                confidence_score Float32,
                INDEX idx_fingerprint fingerprint_hash TYPE bloom_filter GRANULARITY 1,
                INDEX idx_ip ip TYPE tokenbf_v1(32768, 3, 0) GRANULARITY 1
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (timestamp, session_id)
            TTL timestamp + INTERVAL 90 DAY
            SETTINGS index_granularity = 8192;
        "#;

        self.client.query(schema).execute().await.map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Schema creation failed: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_clickhouse_client_compiles() {
        // Placeholder - requires ClickHouse for full testing
        assert!(true);
    }
}
