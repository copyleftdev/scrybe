# Testing & CI Workflow

**Description**: Comprehensive testing workflow for Scrybe - run before every commit and PR

## Workflow Steps

### 1. Pre-Commit Checks

Before starting, ensure working directory is clean:

```bash
# Check git status
git status

# Stash any unrelated changes
git stash save "Temporary stash for testing"
```

### 2. Rust Testing Suite

#### A. Run Unit Tests
```bash
# Run all unit tests
cargo test --workspace

# Run with verbose output to see test names
cargo test --workspace -- --nocapture

# Run specific test module
cargo test --package scrybe-core --lib fingerprint::tests

# Run test with name pattern
cargo test --workspace test_fingerprint
```

Expected: All tests pass ✅

#### B. Run Integration Tests
```bash
# Run integration tests in tests/ directory
cargo test --workspace --test '*'

# Run specific integration test
cargo test --test ingestion_api
```

Expected: All integration tests pass ✅

#### C. Run Doc Tests
```bash
# Test code examples in documentation
cargo test --doc --workspace
```

Expected: All doc examples compile and run ✅

#### D. Check Test Coverage
```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Check coverage percentage
cargo tarpaulin --workspace --out Json | jq '.coverage'

# Verify minimum coverage (90%)
COVERAGE=$(cargo tarpaulin --workspace --out Json | jq '.coverage')
if (( $(echo "$COVERAGE < 90" | bc -l) )); then
  echo "Coverage $COVERAGE% is below 90% threshold"
  exit 1
fi
```

Expected: Coverage ≥ 90% ✅

### 3. JavaScript/TypeScript Testing

#### A. Run Unit Tests
```bash
cd sdk/

# Run Jest tests
npm test

# Run with coverage
npm test -- --coverage

# Run specific test file
npm test -- fingerprint.test.ts

# Run in watch mode (for development)
npm test -- --watch
```

Expected: All tests pass, coverage ≥ 90% ✅

#### B. Type Checking
```bash
# Run TypeScript compiler checks
npm run type-check

# Or directly
npx tsc --noEmit
```

Expected: No type errors ✅

#### C. Linting
```bash
# Run ESLint
npm run lint

# Auto-fix issues
npm run lint -- --fix
```

Expected: No linting errors ✅

### 4. Code Quality Checks

#### A. Rust Formatting
```bash
# Check formatting (don't modify files)
cargo fmt --all -- --check

# Fix formatting issues
cargo fmt --all
```

Expected: All code properly formatted ✅

#### B. Rust Clippy (Linting)
```bash
# Run clippy with strictest settings
cargo clippy --all-features --workspace -- -D warnings

# Check specific package
cargo clippy --package scrybe-core -- -D warnings

# Check for pedantic warnings (more strict)
cargo clippy --all-features --workspace -- -W clippy::pedantic
```

Expected: No clippy warnings ✅

#### C. JavaScript/TypeScript Formatting
```bash
cd sdk/

# Check formatting with Prettier
npm run format:check

# Fix formatting
npm run format
```

Expected: All code properly formatted ✅

### 5. Security Audits

#### A. Rust Dependency Audit
```bash
# Check for known vulnerabilities
cargo audit

# Check with updated database
cargo audit update && cargo audit

# Fail on warnings
cargo audit --deny warnings
```

Expected: No security vulnerabilities ✅

#### B. JavaScript Dependency Audit
```bash
cd sdk/

# Run npm audit
npm audit

# Check for high/critical only
npm audit --audit-level=high

# Generate fix if available
npm audit fix
```

Expected: No high/critical vulnerabilities ✅

### 6. Build Verification

#### A. Rust Build (All Targets)
```bash
# Build in release mode
cargo build --release --workspace

# Build with all features
cargo build --all-features --workspace

# Check compilation (faster than build)
cargo check --workspace
```

Expected: Clean build with no errors ✅

#### B. JavaScript/TypeScript Build
```bash
cd sdk/

# Build production bundle
npm run build

# Verify output exists
ls -lh dist/

# Check bundle size (should be < 50KB gzipped)
gzip -c dist/scrybe-sdk.min.js | wc -c
```

Expected: Build succeeds, bundle size acceptable ✅

### 7. Performance Testing

#### A. Rust Benchmarks
```bash
# Run benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --bench fingerprint_bench

# Compare with baseline (if exists)
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

Expected: Performance meets targets (<5ms fingerprint generation) ✅

#### B. Load Testing (Integration)
```bash
# Start test server
cargo run --release &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Run load test with wrk or similar
wrk -t4 -c100 -d30s http://localhost:8080/health

# Cleanup
kill $SERVER_PID
```

Expected: Handles target load (100k req/sec) ✅

### 8. Documentation Verification

#### A. Check Rust Documentation
```bash
# Build documentation
cargo doc --workspace --no-deps

# Check for missing docs (warnings)
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Open docs in browser to verify
cargo doc --open
```

Expected: All public APIs documented ✅

#### B. Check Markdown Documentation
```bash
# Check for broken links in markdown
npm install -g markdown-link-check
find docs/ -name "*.md" -exec markdown-link-check {} \;

# Spell check (if tool available)
codespell docs/
```

Expected: No broken links, no major typos ✅

### 9. Database Integration Tests

#### A. ClickHouse Tests
```bash
# Start ClickHouse test container
docker run -d --name clickhouse-test -p 8123:8123 clickhouse/clickhouse-server

# Run database integration tests
cargo test --test clickhouse_integration

# Cleanup
docker stop clickhouse-test && docker rm clickhouse-test
```

Expected: All database tests pass ✅

#### B. Redis Tests
```bash
# Start Redis test container
docker run -d --name redis-test -p 6379:6379 redis:alpine

# Run cache integration tests
cargo test --test redis_integration

# Cleanup
docker stop redis-test && docker rm redis-test
```

Expected: All cache tests pass ✅

### 10. End-to-End Tests

#### A. API End-to-End Tests
```bash
# Start all services
docker-compose -f docker-compose.test.yml up -d

# Wait for services to be ready
./scripts/wait-for-services.sh

# Run E2E tests
cargo test --test e2e_full_flow

# Cleanup
docker-compose -f docker-compose.test.yml down
```

Expected: Full flow works end-to-end ✅

### 11. Generate Test Report

Create summary of test results:

```bash
# Generate test report
cat > test-report.md << 'EOF'
# Test Report

**Date**: $(date +%Y-%m-%d)
**Branch**: $(git branch --show-current)
**Commit**: $(git rev-parse --short HEAD)

## Test Results

| Test Suite | Status | Coverage | Time |
|------------|--------|----------|------|
| Rust Unit Tests | ✅ PASS | 92% | 5.2s |
| Rust Integration Tests | ✅ PASS | - | 12.8s |
| JavaScript Unit Tests | ✅ PASS | 94% | 3.1s |
| E2E Tests | ✅ PASS | - | 45.3s |

## Quality Checks

| Check | Status | Issues |
|-------|--------|--------|
| Cargo Fmt | ✅ PASS | 0 |
| Clippy | ✅ PASS | 0 |
| ESLint | ✅ PASS | 0 |
| Security Audit | ✅ PASS | 0 |

## Performance

| Benchmark | Target | Actual | Status |
|-----------|--------|--------|--------|
| Fingerprint Generation | <5ms | 2.3ms | ✅ PASS |
| API Latency (p99) | <10ms | 7.8ms | ✅ PASS |
| Throughput | >100k req/s | 142k req/s | ✅ PASS |

## Summary

All tests passing ✅
Ready for merge ✅
EOF
```

### 12. CI/CD Simulation (Local)

Simulate what CI will run:

```bash
#!/bin/bash
# ci-check.sh - Run all CI checks locally

set -e

echo "🧪 Running CI checks locally..."

echo "📝 Formatting check..."
cargo fmt --all -- --check
cd sdk && npm run format:check && cd ..

echo "🔍 Linting..."
cargo clippy --all-features --workspace -- -D warnings
cd sdk && npm run lint && cd ..

echo "🧪 Unit tests..."
cargo test --workspace
cd sdk && npm test && cd ..

echo "📊 Coverage check..."
cargo tarpaulin --workspace --out Json | jq '.coverage'

echo "🔒 Security audit..."
cargo audit
cd sdk && npm audit --audit-level=high && cd ..

echo "🏗️ Build check..."
cargo build --release --workspace
cd sdk && npm run build && cd ..

echo "✅ All CI checks passed!"
```

Run:
```bash
chmod +x ci-check.sh
./ci-check.sh
```

### 13. Pre-Push Checklist

Before pushing to remote:

- [ ] All tests pass locally
- [ ] Code formatted (`cargo fmt`, `npm run format`)
- [ ] No linting warnings (`clippy`, `eslint`)
- [ ] Coverage ≥ 90%
- [ ] No security vulnerabilities
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if needed)
- [ ] Commit message follows convention
- [ ] Branch up to date with main

### 14. GitHub Actions Verification

After push, monitor CI:

```bash
# Check CI status
gh run list --branch $(git branch --show-current)

# Watch specific run
gh run watch

# View logs if failed
gh run view --log
```

Expected: All CI jobs pass ✅

### 15. Failed Test Debugging

If tests fail:

#### A. Identify Failing Test
```bash
# Run with backtrace for more info
RUST_BACKTRACE=1 cargo test --workspace

# Run specific failing test
cargo test --package scrybe-core test_name -- --exact --nocapture
```

#### B. Debug with Print Statements
```rust
#[test]
fn test_failing() {
    println!("Debug: value = {:?}", value);
    assert_eq!(expected, actual);
}
```

#### C. Run in Isolation
```bash
# Prevent parallel test execution
cargo test -- --test-threads=1

# Run single test
cargo test test_name -- --exact
```

#### D. Check for Flaky Tests
```bash
# Run test multiple times
for i in {1..10}; do
  cargo test test_name --exact || break
done
```

## Continuous Integration Setup

Ensure `.github/workflows/ci.yml` includes:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: cargo test --workspace
        
      - name: Check coverage
        run: |
          cargo tarpaulin --workspace --out Xml
          bash <(curl -s https://codecov.io/bash)
          
      - name: Lint
        run: cargo clippy --all-features --workspace -- -D warnings
        
      - name: Security audit
        run: cargo audit
```

## Quality Gates

Tests must meet these criteria:
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Coverage ≥ 90%
- ✅ No clippy warnings
- ✅ No security vulnerabilities
- ✅ Proper formatting
- ✅ Documentation complete
- ✅ Performance benchmarks pass

Remember: Comprehensive testing prevents bugs in production. Don't skip tests!
