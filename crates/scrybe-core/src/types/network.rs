//! Network-layer signals (TLS, IP, HTTP headers).

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Network-layer signals collected from the connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSignals {
    /// Client IP address
    pub ip: IpAddr,
    /// JA3 TLS fingerprint (if available)
    pub ja3: Option<String>,
    /// JA4 TLS fingerprint (if available)
    pub ja4: Option<String>,
    /// HTTP headers
    pub headers: Vec<Header>,
    /// HTTP version used
    pub http_version: HttpVersion,
}

/// HTTP header key-value pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Header {
    /// Header name (e.g., "User-Agent")
    pub name: String,
    /// Header value
    pub value: String,
}

impl Header {
    /// Create a new header.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// HTTP protocol version.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum HttpVersion {
    /// HTTP/1.0
    Http10,
    /// HTTP/1.1
    #[default]
    Http11,
    /// HTTP/2
    Http2,
    /// HTTP/3
    Http3,
}

impl HttpVersion {
    /// Convert from axum/hyper version type.
    pub fn from_hyper(version: &http::Version) -> Self {
        match *version {
            http::Version::HTTP_10 => Self::Http10,
            http::Version::HTTP_11 => Self::Http11,
            http::Version::HTTP_2 => Self::Http2,
            http::Version::HTTP_3 => Self::Http3,
            _ => Self::Http11, // Default to HTTP/1.1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_header_creation() {
        let header = Header::new("User-Agent", "Mozilla/5.0");
        assert_eq!(header.name, "User-Agent");
        assert_eq!(header.value, "Mozilla/5.0");
    }

    #[test]
    fn test_network_signals_serialization() {
        let signals = NetworkSignals {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            ja3: Some("abc123".to_string()),
            ja4: None,
            headers: vec![Header::new("User-Agent", "Test")],
            http_version: HttpVersion::Http2,
        };

        let json = serde_json::to_string(&signals).unwrap();
        let deserialized: NetworkSignals = serde_json::from_str(&json).unwrap();
        assert_eq!(signals, deserialized);
    }

    #[test]
    fn test_http_version_default() {
        let version = HttpVersion::default();
        assert_eq!(version, HttpVersion::Http11);
    }
}
