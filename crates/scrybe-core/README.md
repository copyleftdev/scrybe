# Scrybe Core

Core types, error handling, and validation for the Scrybe browser behavior intelligence system.

## Features

- **Type-safe domain models** for browser sessions, signals, and fingerprints
- **TigerStyle-compliant error handling** with explicit error types
- **Configuration management** with secret redaction
- **Comprehensive validation** for all input data

## TigerStyle Compliance

This crate strictly follows TigerStyle principles:

- ✅ No `unwrap()`, `panic!()`, or `unsafe` code
- ✅ Explicit error handling with `Result` types
- ✅ All public APIs documented with rustdoc
- ✅ Test coverage > 90%

## Usage

```rust
use scrybe_core::{Config, SecretConfig, types::Session};

// Load configuration
let config = Config::from_env()?;
let secrets = SecretConfig::from_env()?;

// Secrets are automatically redacted in logs
println!("{:?}", secrets); // Prints: SecretConfig { ... [REDACTED] ... }
```

## Error Handling

All errors use the `ScrybeError` enum with detailed context:

```rust
use scrybe_core::ScrybeError;

let err = ScrybeError::validation_error(
    "screen_width",
    "positive integer",
    "0"
);
println!("{}", err);
// Output: Validation error: field='screen_width', expected='positive integer', actual='0'
```

## Documentation

Run `cargo doc --open` to view the full API documentation.
