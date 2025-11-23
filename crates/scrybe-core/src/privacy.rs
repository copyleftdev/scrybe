//! Privacy and GDPR compliance utilities.

use crate::ScrybeError;
use sha2::{Digest, Sha256};

/// Hash an IP address with salt for privacy-preserving storage.
///
/// This ensures IP addresses are never stored in plain text,
/// complying with GDPR data minimization principles.
///
/// # Arguments
///
/// * `ip` - IP address to hash
/// * `salt` - Salt for hashing (should be unique per deployment)
///
/// # Returns
///
/// SHA-256 hash of the IP address as hex string
///
/// # Example
///
/// ```
/// use scrybe_core::privacy::hash_ip;
///
/// let salt = b"deployment-specific-salt";
/// let hashed = hash_ip("192.168.1.1", salt);
/// assert_eq!(hashed.len(), 64); // SHA-256 produces 64 hex chars
/// ```
pub fn hash_ip(ip: &str, salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Validate that no PII (Personally Identifiable Information) is present.
///
/// Checks for common PII patterns that should never be collected.
///
/// # Returns
///
/// `ScrybeError::ValidationError` if PII is detected
pub fn validate_no_pii(data: &str) -> Result<(), ScrybeError> {
    // Check for email patterns
    if data.contains('@') && data.contains('.') {
        return Err(ScrybeError::validation_error(
            "data",
            "no PII",
            "potential email address detected",
        ));
    }

    // Check for phone number patterns (simple check)
    let digit_count = data.chars().filter(|c| c.is_numeric()).count();
    if digit_count >= 10 {
        return Err(ScrybeError::validation_error(
            "data",
            "no PII",
            "potential phone number detected",
        ));
    }

    Ok(())
}

/// GDPR consent status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentStatus {
    /// User has given explicit consent
    Given,
    /// User has not given consent
    NotGiven,
    /// User has withdrawn consent
    Withdrawn,
}

/// GDPR data subject rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSubjectRight {
    /// Right of access (Article 15)
    Access,
    /// Right to rectification (Article 16)
    Rectification,
    /// Right to erasure (Article 17)
    Erasure,
    /// Right to restriction of processing (Article 18)
    Restriction,
    /// Right to data portability (Article 20)
    Portability,
    /// Right to object (Article 21)
    Objection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_ip_deterministic() {
        let salt = b"test-salt";
        let hash1 = hash_ip("192.168.1.1", salt);
        let hash2 = hash_ip("192.168.1.1", salt);
        assert_eq!(hash1, hash2, "IP hashes should be deterministic");
    }

    #[test]
    fn test_hash_ip_different_salts() {
        let hash1 = hash_ip("192.168.1.1", b"salt1");
        let hash2 = hash_ip("192.168.1.1", b"salt2");
        assert_ne!(
            hash1, hash2,
            "Different salts should produce different hashes"
        );
    }

    #[test]
    fn test_hash_ip_different_ips() {
        let salt = b"test-salt";
        let hash1 = hash_ip("192.168.1.1", salt);
        let hash2 = hash_ip("192.168.1.2", salt);
        assert_ne!(
            hash1, hash2,
            "Different IPs should produce different hashes"
        );
    }

    #[test]
    fn test_validate_no_pii_clean_data() {
        assert!(validate_no_pii("Mozilla/5.0").is_ok());
        assert!(validate_no_pii("en-US").is_ok());
    }

    #[test]
    fn test_validate_no_pii_detects_email() {
        let result = validate_no_pii("user@example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_pii_detects_phone() {
        let result = validate_no_pii("1234567890");
        assert!(result.is_err());
    }
}
