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

pub mod session;

// Re-export main types
pub use session::SessionCache;
