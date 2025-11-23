//! Error types for Scrybe operations.
//!
//! Following TigerStyle principles:
//! - Explicit error handling with `map_err` (no `From` implementations)
//! - Detailed context in error messages
//! - No panics or unwraps in error construction

/// Main error type for all Scrybe operations.
///
/// All errors include detailed context about what went wrong and why.
/// This follows TigerStyle: explicit error handling without automatic conversions.
#[derive(Debug, thiserror::Error)]
pub enum ScrybeError {
    /// Invalid session data was provided.
    #[error("Invalid session data: field='{field}', reason='{reason}'")]
    InvalidSession {
        /// The field that failed validation
        field: String,
        /// Why the validation failed
        reason: String,
    },

    /// Configuration error during setup.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Storage operation failed.
    #[error("Storage error: operation='{operation}', reason='{reason}'")]
    StorageError {
        /// The operation that failed
        operation: String,
        /// Why it failed
        reason: String,
    },

    /// Cache operation failed.
    #[error("Cache error: operation='{operation}', reason='{reason}'")]
    CacheError {
        /// The operation that failed
        operation: String,
        /// Why it failed
        reason: String,
    },

    /// Enrichment pipeline failed.
    #[error("Enrichment failed: stage='{stage}', reason='{reason}'")]
    EnrichmentError {
        /// Which enrichment stage failed
        stage: String,
        /// Why it failed
        reason: String,
    },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: limit={limit} requests/{window}")]
    RateLimit {
        /// The rate limit that was exceeded
        limit: u32,
        /// Time window (e.g., "minute", "hour")
        window: String,
    },

    /// Authentication failed.
    #[error("Authentication failed: {reason}")]
    AuthenticationError {
        /// Why authentication failed
        reason: String,
    },

    /// Validation failed.
    #[error("Validation error: field='{field}', expected='{expected}', actual='{actual}'")]
    ValidationError {
        /// The field that failed validation
        field: String,
        /// What was expected
        expected: String,
        /// What was actually provided
        actual: String,
    },

    /// I/O error occurred.
    #[error("I/O error: operation='{operation}', reason='{reason}'")]
    IoError {
        /// The I/O operation that failed
        operation: String,
        /// Why it failed
        reason: String,
    },
}

impl ScrybeError {
    /// Creates an invalid session error.
    pub fn invalid_session(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidSession {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Creates a configuration error.
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError(message.into())
    }

    /// Creates a storage error.
    pub fn storage_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::StorageError {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Creates a cache error.
    pub fn cache_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CacheError {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Creates an enrichment error.
    pub fn enrichment_error(stage: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::EnrichmentError {
            stage: stage.into(),
            reason: reason.into(),
        }
    }

    /// Creates a rate limit error.
    pub fn rate_limit(limit: u32, window: impl Into<String>) -> Self {
        Self::RateLimit {
            limit,
            window: window.into(),
        }
    }

    /// Creates an authentication error.
    pub fn authentication_error(reason: impl Into<String>) -> Self {
        Self::AuthenticationError {
            reason: reason.into(),
        }
    }

    /// Creates a validation error.
    pub fn validation_error(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            field: field.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Creates an I/O error.
    pub fn io_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::IoError {
            operation: operation.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_session_error() {
        let err = ScrybeError::invalid_session("user_agent", "empty string");
        assert!(err.to_string().contains("user_agent"));
        assert!(err.to_string().contains("empty string"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = ScrybeError::rate_limit(100, "minute");
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("minute"));
    }

    #[test]
    fn test_validation_error_context() {
        let err = ScrybeError::validation_error("screen_width", "positive integer", "0");
        let err_string = err.to_string();
        assert!(err_string.contains("screen_width"));
        assert!(err_string.contains("positive integer"));
        assert!(err_string.contains("0"));
    }

    #[test]
    fn test_all_error_variants_have_display() {
        let errors = vec![
            ScrybeError::invalid_session("field", "reason"),
            ScrybeError::config_error("test"),
            ScrybeError::storage_error("write", "timeout"),
            ScrybeError::cache_error("get", "not found"),
            ScrybeError::enrichment_error("fingerprint", "invalid data"),
            ScrybeError::rate_limit(100, "second"),
            ScrybeError::authentication_error("invalid token"),
            ScrybeError::validation_error("field", "expected", "actual"),
            ScrybeError::io_error("read", "permission denied"),
        ];

        for err in errors {
            // Ensure all variants implement Display and Debug
            let _ = format!("{}", err);
            let _ = format!("{:?}", err);
            assert!(!err.to_string().is_empty());
        }
    }
}
