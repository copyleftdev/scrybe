---
trigger: always_on
---

# Testing & Quality Standards

**Activation**: Always On (all code files)

## Testing Requirements

### Minimum Coverage: 90%
- All production code must have ≥90% test coverage
- Use `cargo tarpaulin` or `cargo llvm-cov` for coverage reports
- CI/CD pipeline must enforce coverage threshold
- Coverage reports generated on every PR

```bash
# Run coverage check
cargo tarpaulin --workspace --out Html --output-dir coverage/
```

## Test Types & Structure

### 1. Unit Tests
**Location**: Same file as code, in `#[cfg(test)]` module

```rust
// src/fingerprint/generator.rs

pub fn generate_fingerprint(data: &BrowserTelemetry) -> Result<Fingerprint, FingerprintError> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_fingerprint_success() {
        let telemetry = BrowserTelemetry::default();
        let result = generate_fingerprint(&telemetry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().hash.len(), 32);
    }
    
    #[test]
    fn test_generate_fingerprint_invalid_data() {
        let telemetry = BrowserTelemetry {
            user_agent: String::new(), // Invalid
            ..Default::default()
        };
        let result = generate_fingerprint(&telemetry);
        assert!(matches!(result, Err(FingerprintError::InvalidData(_))));
    }
    
    #[test]
    fn test_fingerprint_deterministic() {
        let telemetry = BrowserTelemetry::default();
        let fp1 = generate_fingerprint(&telemetry).unwrap();
        let fp2 = generate_fingerprint(&telemetry).unwrap();
        assert_eq!(fp1, fp2, "Fingerprints should be deterministic");
    }
}
```

### 2. Integration Tests
**Location**: `tests/` directory in project root

```rust
// tests/ingestion_api.rs

use scrybe::server::Server;
use scrybe::config::ServerConfig;

#[tokio::test]
async fn test_ingestion_endpoint_success() {
    // Start test server
    let config = ServerConfig::test_default();
    let server = Server::new(config).await.unwrap();
    let addr = server.local_addr();
    
    tokio::spawn(async move {
        server.run().await.unwrap();
    });
    
    // Send valid telemetry
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}/v1/ingest", addr))
        .json(&test_telemetry())
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 201);
    let body: IngestResponse = response.json().await.unwrap();
    assert!(body.session_id.is_some());
}
```

### 3. Property-Based Tests
**Use `proptest` for algorithms and data structures**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_fingerprint_hash_length(
        user_agent in ".*",
        screen_width in 0u32..10000,
        screen_height in 0u32..10000,
    ) {
        let telemetry = BrowserTelemetry {
            user_agent,
            screen_width,
            screen_height,
            ..Default::default()
        };
        
        let fingerprint = generate_fingerprint(&telemetry).unwrap();
        prop_assert_eq!(fingerprint.hash.len(), 32);
    }
    
    #[test]
    fn test_bounded_queue_never_exceeds_capacity(
        items in prop::collection::vec(0u32..1000, 0..1000),
        max_size in 1usize..100,
    ) {
        let mut queue = BoundedQueue::new(max_size);
        
        for item in items {
            let _ = queue.push(item);
            prop_assert!(queue.len() <= max_size);
        }
    }
}
```

### 4. Benchmark Tests
**Location**: `benches/` directory

```rust
// benches/fingerprint_generation.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scrybe::fingerprint::generate_fingerprint;

fn benchmark_fingerprint_generation(c: &mut Criterion) {
    let telemetry = test_telemetry();
    
    c.bench_function("generate_fingerprint", |b| {
        b.iter(|| {
            generate_fingerprint(black_box(&telemetry))
        });
    });
}

criterion_group!(benches, benchmark_fingerprint_generation);
criterion_main!(benches);
```

## Test Naming Conventions

### Function Names
```rust
#[test]
fn test_<component>_<scenario>_<expected_result>() {
    // Examples:
    // test_session_manager_create_session_success()
    // test_fingerprint_hasher_invalid_input_returns_error()
    // test_rate_limiter_exceeds_quota_returns_429()
}
```

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Group related tests
    mod session_creation {
        use super::*;
        
        #[test]
        fn test_creates_unique_session_id() { }
        
        #[test]
        fn test_stores_session_in_cache() { }
    }
    
    mod session_validation {
        use super::*;
        
        #[test]
        fn test_validates_session_signature() { }
        
        #[test]
        fn test_rejects_expired_session() { }
    }
}
```

## Test Data & Fixtures

### Use Builder Pattern for Complex Data
```rust
pub struct TelemetryBuilder {
    user_agent: String,
    screen_width: u32,
    screen_height: u32,
    // ... other fields
}

impl TelemetryBuilder {
    pub fn new() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Test)".to_string(),
            screen_width: 1920,
            screen_height: 1080,
        }
    }
    
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.user_agent = ua.to_string();
        self
    }
    
    pub fn screen_size(mut self, width: u32, height: u32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }
    
    pub fn build(self) -> BrowserTelemetry {
        BrowserTelemetry {
            user_agent: self.user_agent,
            screen_width: self.screen_width,
            screen_height: self.screen_height,
            ..Default::default()
        }
    }
}

// Usage in tests
#[test]
fn test_mobile_fingerprint() {
    let telemetry = TelemetryBuilder::new()
        .user_agent("Mozilla/5.0 (iPhone)")
        .screen_size(375, 812)
        .build();
    
    let fingerprint = generate_fingerprint(&telemetry).unwrap();
    // assertions...
}
```

## Async Testing

### Use `tokio::test` for Async Code
```rust
#[tokio::test]
async fn test_async_ingestion() {
    let result = ingest_telemetry(&test_data()).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_ingestion() {
    let handles: Vec<_> = (0..100)
        .map(|_| {
            tokio::spawn(async {
                ingest_telemetry(&test_data()).await
            })
        })
        .collect();
    
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

## Mocking & Test Doubles

### Use `mockall` for Trait Mocking
```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait FingerprintStore {
    async fn save(&self, fp: &Fingerprint) -> Result<(), StoreError>;
    async fn get(&self, id: &str) -> Result<Option<Fingerprint>, StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fingerprint_service_saves_to_store() {
        let mut mock_store = MockFingerprintStore::new();
        
        mock_store
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));
        
        let service = FingerprintService::new(mock_store);
        let result = service.process_telemetry(&test_data()).await;
        
        assert!(result.is_ok());
    }
}
```

## Error Testing

### Test All Error Paths
```rust
#[cfg(test)]
mod error_tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_error_display() {
        let err = FingerprintError::InvalidData("bad format".to_string());
        assert_eq!(
            err.to_string(),
            "Invalid data: bad format"
        );
    }
    
    #[test]
    fn test_error_chain_preservation() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = FingerprintError::IoError(inner);
        
        let source = err.source();
        assert!(source.is_some());
    }
    
    #[test]
    fn test_all_error_variants_covered() {
        // Ensure all error variants are tested
        let errors = vec![
            FingerprintError::InvalidData("test".into()),
            FingerprintError::ValidationFailed("test".into()),
            FingerprintError::StorageError("test".into()),
        ];
        
        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
```

## Performance Testing

### Benchmark Critical Paths
```rust
// Target: < 5ms fingerprint generation
#[test]
fn test_fingerprint_generation_performance() {
    let telemetry = test_telemetry();
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let _ = generate_fingerprint(&telemetry);
    }
    
    let duration = start.elapsed();
    let avg_duration = duration.as_micros() / 1000;
    
    assert!(
        avg_duration < 5000,
        "Average fingerprint generation took {}μs (target: <5000μs)",
        avg_duration
    );
}
```

## Test Documentation

### Document Test Intent
```rust
/// Test: Verifies that fingerprints are deterministic
/// 
/// Given the same browser telemetry data, the fingerprint
/// generation algorithm should always produce identical hashes.
/// This is critical for session identification across requests.
#[test]
fn test_fingerprint_determinism() {
    // Test implementation
}
```

## CI/CD Integration

### Required Checks Before Merge
```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test --all-features --workspace

- name: Check coverage
  run: |
    cargo tarpaulin --workspace --out Xml
    if [ $(grep -oP 'line-rate="\K[0-9.]+' cobertura.xml | head -1 | awk '{print ($1 >= 0.90)}') -eq 0 ]; then
      echo "Coverage below 90%"
      exit 1
    fi

- name: Run clippy
  run: cargo clippy --all-features --workspace -- -D warnings

- name: Check formatting
  run: cargo fmt --all -- --check

- name: Security audit
  run: cargo audit
```

## Test Maintenance

### Keep Tests Fast
- Unit tests should run in < 10ms each
- Integration tests should run in < 1s each
- Use `#[ignore]` for slow tests, run separately
- Mock external dependencies (databases, APIs)

### Avoid Flaky Tests
- No `sleep()` calls - use proper synchronization
- Don't rely on timing for correctness
- Use deterministic test data
- Avoid parallel tests that share mutable state

### Clean Up After Tests
```rust
#[tokio::test]
async fn test_with_cleanup() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Run test
    let result = run_test_with_db(&db_path).await;
    
    // Cleanup happens automatically when temp_dir drops
    assert!(result.is_ok());
}
```

## Quality Gates

Before committing code:
- [ ] All tests pass: `cargo test --all-features`
- [ ] Coverage ≥ 90%: `cargo tarpaulin`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt`
- [ ] No security issues: `cargo audit`
- [ ] Benchmarks meet performance targets
- [ ] Integration tests pass
- [ ] Property tests pass (if applicable)

## Test Review Checklist

When reviewing tests:
- [ ] Tests are clear and well-named
- [ ] Tests cover happy path and error cases
- [ ] Tests are deterministic (no flakiness)
- [ ] Tests are fast (<1s for integration, <10ms for unit)
- [ ] Mocks are used appropriately
- [ ] Test data is realistic
- [ ] Edge cases are covered
- [ ] Performance targets are verified
