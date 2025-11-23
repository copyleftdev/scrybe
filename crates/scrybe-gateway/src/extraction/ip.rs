//! IP address extraction from HTTP requests.

use axum::extract::ConnectInfo;
use std::net::{IpAddr, SocketAddr};
use tracing::debug;

/// Extract client IP address from connection info.
///
/// This function prioritizes the actual connection IP over forwarded headers
/// to prevent spoofing.
///
/// # Security Note
///
/// In production behind a reverse proxy, you may want to check
/// X-Forwarded-For or X-Real-IP headers, but ONLY if you trust the proxy.
/// For now, we use the direct connection IP.
pub fn extract_ip_info(connect_info: &ConnectInfo<SocketAddr>) -> IpAddr {
    let ip = connect_info.0.ip();
    debug!("Extracted client IP: {}", ip);
    ip
}

/// Hash IP address with salt for privacy-preserving storage.
///
/// Uses SHA-256 to create a one-way hash of the IP address combined with
/// a salt, making it impossible to reverse while still allowing
/// rate limiting and abuse detection.
pub fn hash_ip(ip: &IpAddr, salt: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    hasher.update(ip.to_string().as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_hash_ip_deterministic() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let salt = b"test-salt";
        
        let hash1 = hash_ip(&ip, salt);
        let hash2 = hash_ip(&ip, salt);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_hash_ip_different_ips() {
        let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));
        let salt = b"test-salt";
        
        let hash1 = hash_ip(&ip1, salt);
        let hash2 = hash_ip(&ip2, salt);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_ip_different_salts() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        let hash1 = hash_ip(&ip, b"salt1");
        let hash2 = hash_ip(&ip, b"salt2");
        
        assert_ne!(hash1, hash2);
    }
}
