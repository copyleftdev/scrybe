//! HTTP header extraction and parsing.

use axum::http::HeaderMap;
use scrybe_core::types::{Header, HttpVersion};
use tracing::warn;

/// Extract relevant headers from HTTP request.
///
/// Filters out sensitive headers (Authorization, Cookie) and captures
/// headers useful for fingerprinting.
pub fn extract_headers(headers: &HeaderMap) -> Vec<Header> {
    let mut result = Vec::new();

    // List of headers to capture (case-insensitive)
    let capture_headers = [
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        "referer",
        "sec-fetch-dest",
        "sec-fetch-mode",
        "sec-fetch-site",
        "sec-ch-ua",
        "sec-ch-ua-mobile",
        "sec-ch-ua-platform",
    ];

    for header_name in &capture_headers {
        if let Some(value) = headers.get(*header_name) {
            if let Ok(value_str) = value.to_str() {
                result.push(Header::new(*header_name, value_str));
            } else {
                warn!("Failed to parse header: {}", header_name);
            }
        }
    }

    result
}

/// Extract HTTP version from request.
pub fn extract_http_version(version: &http::Version) -> HttpVersion {
    HttpVersion::from_hyper(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};

    #[test]
    fn test_extract_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("Mozilla/5.0"),
        );
        headers.insert(
            HeaderName::from_static("accept-language"),
            HeaderValue::from_static("en-US,en;q=0.9"),
        );

        let extracted = extract_headers(&headers);

        assert_eq!(extracted.len(), 2);
        assert!(extracted.iter().any(|h| h.name == "user-agent"));
        assert!(extracted.iter().any(|h| h.name == "accept-language"));
    }

    #[test]
    fn test_extract_headers_filters_sensitive() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("Mozilla/5.0"),
        );
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer secret"),
        );
        headers.insert(
            HeaderName::from_static("cookie"),
            HeaderValue::from_static("session=abc123"),
        );

        let extracted = extract_headers(&headers);

        // Should only capture user-agent, not authorization or cookie
        assert_eq!(extracted.len(), 1);
        assert!(extracted.iter().any(|h| h.name == "user-agent"));
        assert!(!extracted.iter().any(|h| h.name == "authorization"));
        assert!(!extracted.iter().any(|h| h.name == "cookie"));
    }

    #[test]
    fn test_extract_http_version() {
        let version = http::Version::HTTP_11;
        let extracted = extract_http_version(&version);
        assert_eq!(extracted, HttpVersion::Http11);

        let version = http::Version::HTTP_2;
        let extracted = extract_http_version(&version);
        assert_eq!(extracted, HttpVersion::Http2);
    }
}
