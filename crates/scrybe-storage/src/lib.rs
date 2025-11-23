//! # Scrybe Storage
//!
//! ClickHouse storage interface for browser session data.
//!
//! ## Features
//!
//! - Batch writes for high throughput
//! - Optimized schema for time-series data
//! - Query interface for analytics
//!
//! ## TigerStyle Compliance
//!
//! - No unwrap/panic in production code
//! - Explicit error handling
//! - Connection pooling

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]

pub mod client;
pub mod writer;

// Re-export main types
pub use client::ClickHouseClient;
pub use writer::SessionWriter;
