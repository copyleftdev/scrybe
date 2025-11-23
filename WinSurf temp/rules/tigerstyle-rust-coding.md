# TigerStyle Rust Coding Standards

**Activation**: Always On (for Rust files: `*.rs`, `Cargo.toml`)

## Core Principles

### 1. Safety First
- **NO `unwrap()` or `panic!()` in production code** - Always use `Result` and `Option` with proper error handling
- Use `.expect()` only in tests or with detailed error messages explaining why failure is impossible
- Prefer `?` operator for error propagation
- Use `#[must_use]` attribute for functions returning `Result` or important values

### 2. Error Handling
```rust
// ✅ GOOD - Explicit error handling with context
pub fn process_fingerprint(data: &[u8]) -> Result<Fingerprint, FingerprintError> {
    let parsed = parse_data(data)
        .map_err(|e| FingerprintError::ParseFailed(format!("Invalid data: {}", e)))?;
    
    validate_fingerprint(&parsed)
        .map_err(|e| FingerprintError::ValidationFailed(e))?;
    
    Ok(parsed)
}

// ❌ BAD - Using unwrap
pub fn process_fingerprint(data: &[u8]) -> Fingerprint {
    parse_data(data).unwrap() // Don't do this!
}
```

### 3. Type Safety
- Use newtype patterns for domain-specific types (SessionId, FingerprintHash, etc.)
- Leverage the type system to prevent invalid states at compile time
- Use `#[derive(Debug, Clone)]` judiciously - avoid Clone for large types
- Prefer `&str` over `String` for function parameters when ownership isn't needed

```rust
// ✅ GOOD - Type-safe session ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(uuid::Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

// ❌ BAD - Raw string as session ID
pub type SessionId = String;
```

### 4. Simplicity Over Cleverness
- Prefer explicit code over implicit behavior
- Avoid excessive macro usage - use functions when possible
- Keep functions focused and small (< 50 lines ideally)
- Use descriptive variable names - no single-letter variables except in very short closures

### 5. Performance Patterns
- Use `Vec::with_capacity()` when size is known
- Prefer `&[T]` over `&Vec<T>` for function parameters
- Use `Cow<str>` for potentially borrowed strings
- Avoid unnecessary allocations - use references when possible
- Use `#[inline]` for hot path functions (< 10 lines)

```rust
// ✅ GOOD - Pre-allocated capacity
let mut events = Vec::with_capacity(expected_count);
for event in source {
    events.push(event);
}

// ❌ BAD - Repeated reallocations
let mut events = Vec::new();
for event in source {
    events.push(event);
}
```

### 6. Testing Requirements
- **Minimum 90% test coverage** for all modules
- Unit tests in same file with `#[cfg(test)]`
- Integration tests in `tests/` directory
- Use `proptest` for property-based testing of core algorithms
- Document test rationale with `/// Test: <description>`

### 7. Documentation Standards
- All public APIs must have `///` documentation
- Include `# Examples` section with working code
- Document `# Errors` section for functions returning `Result`
- Add `# Panics` section if function can panic (should be rare!)
- Use `#![warn(missing_docs)]` in library crates

```rust
/// Generates a multi-layer browser fingerprint from raw telemetry data.
///
/// This function combines network, HTTP, and browser-level signals into
/// a deterministic fingerprint hash using SHA-256.
///
/// # Arguments
///
/// * `telemetry` - Raw telemetry data from browser agent
///
/// # Returns
///
/// A 32-byte fingerprint hash uniquely identifying the browser session.
///
/// # Errors
///
/// Returns `FingerprintError::InvalidData` if telemetry is malformed or incomplete.
///
/// # Examples
///
/// ```
/// use scrybe::fingerprint::generate_fingerprint;
///
/// let telemetry = BrowserTelemetry::default();
/// let fingerprint = generate_fingerprint(&telemetry)?;
/// assert_eq!(fingerprint.len(), 32);
/// ```
pub fn generate_fingerprint(telemetry: &BrowserTelemetry) -> Result<[u8; 32], FingerprintError> {
    // Implementation
}
```

### 8. Dependency Management
- Justify every dependency in code comments
- Prefer `no_std` compatible crates when possible
- Pin versions in `Cargo.toml` with explanation comments
- Audit dependencies for security: `cargo audit`

### 9. Async/Await Patterns
- Use `tokio` for async runtime (already standard in Scrybe)
- Prefer `async fn` over `impl Future`
- Use `tokio::select!` for cancellation-safe operations
- Document async cancellation behavior

### 10. Security-Specific Rules
- Use constant-time comparison for cryptographic operations: `subtle::ConstantTimeEq`
- Zero sensitive data on drop: implement `zeroize::Zeroize`
- Use `ring` or `rustls` for cryptography - no `openssl`
- Validate all external input at API boundaries

## Code Organization

### Module Structure
```rust
// src/fingerprint/mod.rs
pub mod generator;  // Fingerprint generation logic
pub mod hasher;     // Hashing utilities
pub mod validator;  // Fingerprint validation

use generator::FingerprintGenerator;
use hasher::FingerprintHasher;

// Re-export main types
pub use generator::{Fingerprint, FingerprintError};
```

### File Naming
- Use snake_case: `session_manager.rs`, `fingerprint_cache.rs`
- One public type per file for main modules
- Group related utilities in `utils/` or `helpers/` submodules

## Common Patterns

### Bounded Collections (DoS Prevention)
```rust
use std::collections::VecDeque;

pub struct BoundedQueue<T> {
    inner: VecDeque<T>,
    max_size: usize,
}

impl<T> BoundedQueue<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn push(&mut self, item: T) -> Result<(), BoundedQueueError> {
        if self.inner.len() >= self.max_size {
            return Err(BoundedQueueError::CapacityExceeded);
        }
        self.inner.push_back(item);
        Ok(())
    }
}
```

### Configuration Pattern
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    
    #[serde(default)]
    pub enable_tls: bool,
}

fn default_max_connections() -> usize { 10_000 }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: default_max_connections(),
            enable_tls: true,
        }
    }
}
```

## Formatting & Linting

- Run `cargo fmt` before every commit
- Run `cargo clippy -- -D warnings` - no warnings allowed
- Enable these lints in `lib.rs`:

```rust
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]
#![forbid(unsafe_code)] // For security-critical modules
```

## Commit Standards

When committing Rust code:
- Prefix: `feat(core):`, `fix(fingerprint):`, `perf(cache):`, `docs(api):`
- Ensure `cargo test --all-features` passes
- Run `cargo bench` for performance-critical changes
- Update CHANGELOG.md for user-facing changes
