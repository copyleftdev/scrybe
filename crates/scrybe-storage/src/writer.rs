//! Session writer for ClickHouse storage.

use scrybe_core::{types::Session, ScrybeError};

/// Writes session data to ClickHouse.
pub struct SessionWriter;

impl SessionWriter {
    /// Write a session to ClickHouse.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if the write fails.
    pub async fn write(_session: &Session) -> Result<(), ScrybeError> {
        // TODO: Implement actual ClickHouse write
        Ok(())
    }

    /// Batch write multiple sessions to ClickHouse.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if the write fails.
    pub async fn write_batch(_sessions: &[Session]) -> Result<(), ScrybeError> {
        // TODO: Implement batch write
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_session_writer_compiles() {
        // Placeholder test
        assert!(true);
    }
}
