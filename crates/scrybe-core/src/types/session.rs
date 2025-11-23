//! Session and fingerprint types.

use super::{BehavioralSignals, BrowserSignals, NetworkSignals};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Complete browser session with all collected signals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    /// Session start timestamp
    pub timestamp: DateTime<Utc>,
    /// Network-layer signals
    pub network: NetworkSignals,
    /// Browser environment signals
    pub browser: BrowserSignals,
    /// User behavioral patterns
    pub behavioral: BehavioralSignals,
    /// Computed fingerprint
    pub fingerprint: Fingerprint,
}

/// Unique session identifier (UUID v4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Generate a new random session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Composite fingerprint identifying a browser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fingerprint {
    /// SHA-256 hash of all signals (hex string)
    pub hash: String,
    /// Individual fingerprint components
    pub components: FingerprintComponents,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Individual components that make up the fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FingerprintComponents {
    /// Canvas fingerprint hash
    pub canvas: Option<String>,
    /// WebGL fingerprint hash
    pub webgl: Option<String>,
    /// Audio fingerprint hash
    pub audio: Option<String>,
    /// Font list hash
    pub fonts: Option<String>,
    /// Plugin list hash
    pub plugins: Option<String>,
    /// Screen configuration hash
    pub screen: Option<String>,
    /// Network/TLS hash
    pub network: Option<String>,
}

impl Fingerprint {
    /// Create a new fingerprint with validation.
    ///
    /// # Errors
    ///
    /// Returns `None` if the hash is invalid or confidence is out of range.
    pub fn new(hash: String, components: FingerprintComponents, confidence: f64) -> Option<Self> {
        // Validate hash is 64 hex characters (SHA-256)
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }

        // Validate confidence is in [0, 1]
        if !(0.0..=1.0).contains(&confidence) {
            return None;
        }

        Some(Self {
            hash,
            components,
            confidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2); // Should be different
    }

    #[test]
    fn test_session_id_string_conversion() {
        let id = SessionId::new();
        let string = format!("{}", id); // Use Display trait
        let parsed: SessionId = string.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_session_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let session_id = SessionId::from_uuid(uuid);
        assert_eq!(session_id.as_uuid(), &uuid);
    }

    #[test]
    fn test_fingerprint_validation() {
        let valid_hash = "a".repeat(64);
        let fingerprint =
            Fingerprint::new(valid_hash.clone(), FingerprintComponents::default(), 0.95);
        assert!(fingerprint.is_some());

        // Invalid hash length
        let fingerprint =
            Fingerprint::new("abc123".to_string(), FingerprintComponents::default(), 0.95);
        assert!(fingerprint.is_none());

        // Invalid confidence
        let fingerprint =
            Fingerprint::new(valid_hash.clone(), FingerprintComponents::default(), 1.5);
        assert!(fingerprint.is_none());

        // Invalid hex characters
        let invalid_hash = "z".repeat(64);
        let fingerprint = Fingerprint::new(invalid_hash, FingerprintComponents::default(), 0.95);
        assert!(fingerprint.is_none());
    }

    #[test]
    fn test_fingerprint_components_default() {
        let components = FingerprintComponents::default();
        assert!(components.canvas.is_none());
        assert!(components.webgl.is_none());
        assert!(components.audio.is_none());
    }

    #[test]
    fn test_session_serialization() {
        use crate::types::{HttpVersion, ScreenInfo};
        use std::net::Ipv4Addr;

        let session = Session {
            id: SessionId::new(),
            timestamp: Utc::now(),
            network: NetworkSignals {
                ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                ja3: None,
                ja4: None,
                headers: vec![],
                http_version: HttpVersion::Http2,
            },
            browser: BrowserSignals {
                canvas_hash: None,
                webgl_hash: None,
                audio_hash: None,
                fonts: vec![],
                plugins: vec![],
                timezone: "UTC".to_string(),
                language: "en-US".to_string(),
                screen: ScreenInfo::default(),
                user_agent: "Test".to_string(),
            },
            behavioral: BehavioralSignals {
                mouse_events: vec![],
                scroll_events: vec![],
                click_events: vec![],
                timing: Default::default(),
            },
            fingerprint: Fingerprint::new("a".repeat(64), FingerprintComponents::default(), 0.9)
                .unwrap(),
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, deserialized.id);
    }
}
