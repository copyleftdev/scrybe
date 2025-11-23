//! Session writer for ClickHouse storage.

use crate::client::ClickHouseClient;
use scrybe_core::{types::Session, ScrybeError};
use serde::Serialize;

/// Row format for ClickHouse sessions table.
#[derive(Debug, Serialize, clickhouse::Row)]
struct SessionRow {
    session_id: String,
    timestamp: i64,
    fingerprint_hash: String,
    ip: String,
    user_agent: String,
    network_signals: String,
    browser_signals: String,
    behavioral_signals: String,
    bot_probability: f32,
    confidence_score: f32,
}

impl SessionRow {
    /// Convert a Session to ClickHouse row format.
    fn from_session(session: &Session) -> Result<Self, ScrybeError> {
        Ok(Self {
            session_id: session.id.to_string(),
            timestamp: session.timestamp.timestamp_millis(),
            fingerprint_hash: session.fingerprint.hash.clone(),
            ip: session.network.ip.to_string(),
            user_agent: session.browser.user_agent.clone(),
            network_signals: serde_json::to_string(&session.network).map_err(|e| {
                ScrybeError::storage_error(
                    "clickhouse",
                    format!("JSON serialization failed: {}", e),
                )
            })?,
            browser_signals: serde_json::to_string(&session.browser).map_err(|e| {
                ScrybeError::storage_error(
                    "clickhouse",
                    format!("JSON serialization failed: {}", e),
                )
            })?,
            behavioral_signals: serde_json::to_string(&session.behavioral).map_err(|e| {
                ScrybeError::storage_error(
                    "clickhouse",
                    format!("JSON serialization failed: {}", e),
                )
            })?,
            bot_probability: 0.0,  // Will be filled by enrichment pipeline
            confidence_score: 0.0, // Will be filled by enrichment pipeline
        })
    }
}

/// Writes session data to ClickHouse.
pub struct SessionWriter {
    client: ClickHouseClient,
}

impl SessionWriter {
    /// Create a new session writer.
    pub fn new(client: ClickHouseClient) -> Self {
        Self { client }
    }

    /// Write a session to ClickHouse.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if the write fails.
    pub async fn write(&self, session: &Session) -> Result<(), ScrybeError> {
        let row = SessionRow::from_session(session)?;

        let mut insert = self.client.client().insert("sessions").map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Insert preparation failed: {}", e))
        })?;

        insert.write(&row).await.map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Write failed: {}", e))
        })?;

        insert.end().await.map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Write completion failed: {}", e))
        })?;

        Ok(())
    }

    /// Batch write multiple sessions to ClickHouse.
    ///
    /// More efficient for high-throughput scenarios.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::StorageError` if the write fails.
    pub async fn write_batch(&self, sessions: &[Session]) -> Result<(), ScrybeError> {
        if sessions.is_empty() {
            return Ok(());
        }

        let mut insert = self.client.client().insert("sessions").map_err(|e| {
            ScrybeError::storage_error("clickhouse", format!("Insert preparation failed: {}", e))
        })?;

        for session in sessions {
            let row = SessionRow::from_session(session)?;
            insert.write(&row).await.map_err(|e| {
                ScrybeError::storage_error("clickhouse", format!("Write failed: {}", e))
            })?;
        }

        insert.end().await.map_err(|e| {
            ScrybeError::storage_error(
                "clickhouse",
                format!("Batch write completion failed: {}", e),
            )
        })?;

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
