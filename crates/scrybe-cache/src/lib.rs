//! # Scrybe Cache
//!
//! Redis session cache for high-performance session management.
//!
//! ## Features
//!
//! - Session storage with TTL
//! - Fingerprint correlation
//! - Nonce validation
//! - Rate limiting
//!
//! ## TigerStyle Compliance
//!
//! - No unwrap/panic in production code
//! - Explicit error handling
//! - Connection pooling

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]

/// Redis client with connection pooling.
pub mod client;
/// Nonce validation for replay attack prevention.
pub mod nonce;
/// Session cache management.
pub mod session;

// Re-export main types
pub use client::RedisClient;
pub use nonce::NonceValidator;
pub use session::SessionCache;
