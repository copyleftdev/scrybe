//! # Scrybe Enrichment Pipeline
//!
//! Fingerprinting and enrichment processing for browser sessions.
//!
//! ## Features
//!
//! - Composite fingerprint generation (SHA-256)
//! - GeoIP enrichment
//! - Similarity detection
//! - Anomaly detection
//!
//! ## TigerStyle Compliance
//!
//! - No unwrap/panic in production code
//! - Explicit error handling
//! - Type-safe fingerprinting

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]

pub mod fingerprint;

// Re-export main types
pub use fingerprint::FingerprintGenerator;
