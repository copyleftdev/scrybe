//! Configuration and secrets management.
//!
//! Provides type-safe configuration loading from environment variables
//! with the `Secret<T>` wrapper to prevent accidental exposure of
//! sensitive data in logs or debug output.

use crate::error::ScrybeError;
use std::env;
use std::fmt;
use std::path::PathBuf;

/// Main configuration for Scrybe services.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Enable TLS
    pub enable_tls: bool,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::ConfigError` if required environment variables
    /// are missing or invalid.
    pub fn from_env() -> Result<Self, ScrybeError> {
        let host = env::var("SCRYBE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("SCRYBE_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|e| ScrybeError::config_error(format!("Invalid SCRYBE_PORT: {}", e)))?;

        let max_connections = env::var("SCRYBE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .map_err(|e| {
                ScrybeError::config_error(format!("Invalid SCRYBE_MAX_CONNECTIONS: {}", e))
            })?;

        let enable_tls = env::var("SCRYBE_ENABLE_TLS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .map_err(|e| ScrybeError::config_error(format!("Invalid SCRYBE_ENABLE_TLS: {}", e)))?;

        let request_timeout_secs = env::var("SCRYBE_REQUEST_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|e| {
                ScrybeError::config_error(format!("Invalid SCRYBE_REQUEST_TIMEOUT_SECS: {}", e))
            })?;

        Ok(Self {
            host,
            port,
            max_connections,
            enable_tls,
            request_timeout_secs,
        })
    }

    /// Create default configuration for testing.
    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            enable_tls: false,
            request_timeout_secs: 30,
        }
    }
}

/// Configuration for sensitive values (credentials, keys, etc.).
///
/// All sensitive values are wrapped in `Secret<T>` to prevent accidental
/// exposure in logs or debug output.
#[derive(Clone)]
pub struct SecretConfig {
    /// ClickHouse connection URL
    pub clickhouse_url: Secret<String>,
    /// ClickHouse password
    pub clickhouse_password: Secret<String>,
    /// Redis connection URL
    pub redis_url: Secret<String>,
    /// API key salt for HMAC
    pub api_key_salt: Secret<String>,
    /// TLS private key path
    pub tls_key_path: Secret<PathBuf>,
}

impl SecretConfig {
    /// Load secret configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::ConfigError` if required environment variables
    /// are missing or invalid.
    pub fn from_env() -> Result<Self, ScrybeError> {
        let clickhouse_url = env::var("CLICKHOUSE_URL")
            .map_err(|_| ScrybeError::config_error("Missing CLICKHOUSE_URL"))?;

        let clickhouse_password = env::var("CLICKHOUSE_PASSWORD")
            .map_err(|_| ScrybeError::config_error("Missing CLICKHOUSE_PASSWORD"))?;

        let redis_url =
            env::var("REDIS_URL").map_err(|_| ScrybeError::config_error("Missing REDIS_URL"))?;

        let api_key_salt = env::var("API_KEY_SALT")
            .map_err(|_| ScrybeError::config_error("Missing API_KEY_SALT"))?;

        let tls_key_path = env::var("TLS_KEY_PATH")
            .map(PathBuf::from)
            .map_err(|_| ScrybeError::config_error("Missing TLS_KEY_PATH"))?;

        Ok(Self {
            clickhouse_url: Secret::new(clickhouse_url),
            clickhouse_password: Secret::new(clickhouse_password),
            redis_url: Secret::new(redis_url),
            api_key_salt: Secret::new(api_key_salt),
            tls_key_path: Secret::new(tls_key_path),
        })
    }

    /// Create test configuration with dummy secrets.
    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            clickhouse_url: Secret::new("http://localhost:8123".to_string()),
            clickhouse_password: Secret::new("test_password".to_string()),
            redis_url: Secret::new("redis://localhost:6379".to_string()),
            api_key_salt: Secret::new("test_salt_12345678901234567890123456789012".to_string()),
            tls_key_path: Secret::new(PathBuf::from("/tmp/test-key.pem")),
        }
    }
}

impl fmt::Debug for SecretConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretConfig")
            .field("clickhouse_url", &self.clickhouse_url)
            .field("clickhouse_password", &self.clickhouse_password)
            .field("redis_url", &self.redis_url)
            .field("api_key_salt", &self.api_key_salt)
            .field("tls_key_path", &self.tls_key_path)
            .finish()
    }
}

/// Wrapper for sensitive values that prevents accidental exposure.
///
/// When printed or logged, `Secret<T>` displays `[REDACTED]` instead
/// of the actual value. The value can only be accessed via the `expose()`
/// method, making it explicit when secrets are being used.
///
/// # Example
///
/// ```
/// use scrybe_core::Secret;
///
/// let password = Secret::new("super_secret_123".to_string());
/// println!("{:?}", password); // Prints: [REDACTED]
///
/// // Explicit access required
/// let actual_password = password.expose();
/// ```
#[derive(Clone)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Create a new secret value.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Expose the underlying secret value.
    ///
    /// This method makes it explicit that you are accessing sensitive data.
    /// Use with caution and never log or print the returned value.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Consume the secret and return the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::test_default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.max_connections > 0);
    }

    #[test]
    fn test_secret_redaction() {
        let secret = Secret::new("sensitive_data".to_string());
        let debug_output = format!("{:?}", secret);
        let display_output = format!("{}", secret);

        assert_eq!(debug_output, "[REDACTED]");
        assert_eq!(display_output, "[REDACTED]");
        assert!(!debug_output.contains("sensitive_data"));
        assert!(!display_output.contains("sensitive_data"));
    }

    #[test]
    fn test_secret_expose() {
        let secret = Secret::new("my_secret".to_string());
        assert_eq!(secret.expose(), "my_secret");
    }

    #[test]
    fn test_secret_into_inner() {
        let secret = Secret::new(42);
        assert_eq!(secret.into_inner(), 42);
    }

    #[test]
    fn test_secret_config_debug_redaction() {
        let config = SecretConfig::test_default();
        let debug_output = format!("{:?}", config);

        // Should contain field names but redact values
        assert!(debug_output.contains("SecretConfig"));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("test_password"));
        assert!(!debug_output.contains("test_salt"));
    }

    #[test]
    fn test_config_from_env_missing_vars() {
        // Clear environment variables
        env::remove_var("CLICKHOUSE_URL");

        let result = SecretConfig::from_env();
        assert!(result.is_err());

        match result {
            Err(ScrybeError::ConfigError(msg)) => {
                assert!(msg.contains("CLICKHOUSE_URL"));
            }
            _ => panic!("Expected ConfigError"),
        }
    }
}
