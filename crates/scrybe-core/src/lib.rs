//! # Scrybe Core
//!
//! Core types, error handling, and validation for the Scrybe browser
//! behavior intelligence system.
//!
//! ## TigerStyle Compliance
//!
//! This crate follows TigerStyle principles:
//! - **Safety**: No unwrap, panic, or unsafe code
//! - **Simplicity**: Explicit error handling with Result types
//! - **Correctness**: Type-driven design with validation
//! - **Performance**: Zero-copy where possible
//!
//! ## Modules
//!
//! - [`error`]: Error types for all Scrybe operations
//! - [`config`]: Configuration and secrets management
//! - [`types`]: Core domain types

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod privacy;
pub mod types;

// Re-export commonly used types
pub use config::{Config, Secret};
pub use error::ScrybeError;
