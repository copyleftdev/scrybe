# Code Review Workflow

**Description**: Comprehensive code review process for Scrybe pull requests

## Workflow Steps

### 1. Identify Pull Request

```bash
# List open PRs
gh pr list

# Or get specific PR number from user
```

Ask user which PR to review, or use the PR number provided.

### 2. Checkout PR Branch

```bash
# Fetch and checkout PR branch
gh pr checkout [PR_NUMBER]

# Verify branch
git branch --show-current
```

### 3. Review PR Description

Read the PR description carefully:
- What is the goal of this change?
- Are there linked issues?
- Is there a design document or RFC?
- What are the acceptance criteria?

### 4. Code Analysis - Rust Files

For each Rust file changed:

#### A. Safety & Error Handling
```bash
# Search for unsafe patterns
rg "unwrap\(\)|panic!|expect\(" --type rust

# Check for proper error handling
rg "Result<|Option<" --type rust
```

Check:
- [ ] No `unwrap()` or `panic!()` in production code
- [ ] All errors use `Result` with proper context
- [ ] `expect()` only used with detailed messages
- [ ] Error types are well-defined

#### B. Type Safety
- [ ] Newtypes used for domain concepts
- [ ] No unnecessary `Clone` derivations
- [ ] Proper lifetime annotations
- [ ] `#[must_use]` on important return values

#### C. Testing
```bash
# Check for test coverage
rg "#\[test\]|#\[tokio::test\]" --type rust

# Run tests
cargo test --all-features

# Check coverage
cargo tarpaulin --workspace
```

Verify:
- [ ] New code has ≥ 90% test coverage
- [ ] Edge cases tested
- [ ] Error paths tested
- [ ] Tests are deterministic

#### D. Documentation
```bash
# Check for doc comments
rg "^///|^//!" --type rust
```

Verify:
- [ ] Public APIs documented with `///`
- [ ] Examples included in docs
- [ ] Errors section present
- [ ] Complex logic explained

#### E. Performance
```bash
# Check for common performance issues
rg "Vec::new\(\)|String::new\(\)" --type rust
```

Review:
- [ ] `Vec::with_capacity()` used when size known
- [ ] No unnecessary allocations
- [ ] Efficient algorithms chosen
- [ ] Hot paths optimized

#### F. Security
```bash
# Check for security patterns
rg "subtle::|ring::|rustls::" --type rust
```

Verify:
- [ ] Constant-time comparisons for secrets
- [ ] No hardcoded credentials
- [ ] Input validation at boundaries
- [ ] Cryptography uses approved libraries

### 5. Code Analysis - JavaScript/TypeScript Files

For each JS/TS file changed:

#### A. Type Safety
```bash
# Check for any types
rg ": any|as any" --type ts

# Verify strict mode
cat tsconfig.json | grep "strict"
```

Check:
- [ ] No `any` types (use `unknown` if needed)
- [ ] Strict mode enabled
- [ ] Proper interface definitions
- [ ] Type guards for runtime checks

#### B. Bounded Collections
```bash
# Look for unbounded arrays/collections
rg "\.push\(|\.unshift\(" --type ts
```

Verify:
- [ ] All collections have maximum size
- [ ] Event queues are bounded
- [ ] Memory exhaustion prevented

#### C. Error Handling
```bash
# Check for try-catch blocks
rg "try {|catch \(" --type ts
```

Review:
- [ ] Promises have .catch() handlers
- [ ] Async functions wrapped in try-catch
- [ ] Errors logged appropriately
- [ ] Graceful degradation implemented

#### D. Security
```bash
# Check for security patterns
rg "crypto\.|HMAC|SHA" --type ts
```

Verify:
- [ ] HMAC authentication implemented
- [ ] No secrets in code
- [ ] API calls authenticated
- [ ] Input sanitized

### 6. Run Quality Checks

#### Rust Quality Gates
```bash
# Format check
cargo fmt -- --check

# Clippy (no warnings allowed)
cargo clippy --all-features --workspace -- -D warnings

# Security audit
cargo audit

# Run tests
cargo test --all-features --workspace

# Run benchmarks (if applicable)
cargo bench
```

#### JavaScript/TypeScript Quality Gates
```bash
# Linting
npm run lint

# Type checking
npm run type-check

# Tests
npm test

# Build
npm run build

# Bundle size check
du -h dist/*.js
```

### 7. Architecture Review

Check if changes:
- [ ] Follow existing patterns
- [ ] Don't introduce new dependencies unnecessarily
- [ ] Maintain separation of concerns
- [ ] Don't create circular dependencies
- [ ] Scale appropriately

### 8. Security Review

Critical security checks:
- [ ] No PII collected or logged
- [ ] Input validation at API boundaries
- [ ] Rate limiting appropriate
- [ ] Authentication/authorization correct
- [ ] Crypto operations use approved libraries
- [ ] No timing attacks possible
- [ ] GDPR compliance maintained

### 9. Performance Review

Performance considerations:
- [ ] No obvious performance regressions
- [ ] Algorithms are efficient (check Big O)
- [ ] Database queries optimized
- [ ] Memory usage reasonable
- [ ] Latency targets met (<10ms p99)

### 10. Documentation Review

Check documentation updates:
```bash
# Look for changed markdown files
git diff --name-only origin/main | grep "\.md$"

# Check if CHANGELOG updated
git diff CHANGELOG.md
```

Verify:
- [ ] CHANGELOG.md updated (if user-facing)
- [ ] README updated (if needed)
- [ ] API docs updated (if API changed)
- [ ] Code comments adequate

### 11. Create Review Comments

Structure your review:

```markdown
## Code Review: PR #[NUMBER]

### Summary
[Overall assessment of the changes]

### Strengths
- [Positive aspects of the implementation]

### Issues Found

#### 🔴 Critical (Must Fix Before Merge)
- [ ] **File**: `path/to/file.rs`, **Line**: 42
  **Issue**: Using `unwrap()` in production code
  **Fix**: Replace with proper error handling using `?` operator

#### 🟡 Major (Should Fix Before Merge)
- [ ] **File**: `path/to/file.ts`, **Line**: 123
  **Issue**: Unbounded array could cause memory exhaustion
  **Fix**: Use BoundedArray class with appropriate limit

#### 🟢 Minor (Consider Addressing)
- **File**: `path/to/file.rs`, **Line**: 200
  **Issue**: Variable name `x` is not descriptive
  **Suggestion**: Rename to `session_id` for clarity

### Quality Checks
- [x] Tests pass
- [x] Coverage ≥ 90%
- [x] Clippy/Lint clean
- [x] Formatted correctly
- [ ] Security audit clean (found 1 issue - see critical items)

### Performance Analysis
[Impact on performance, if any]

### Security Analysis
[Security implications and verification]

### Recommendation
**[APPROVE / REQUEST CHANGES / REJECT]**

[Justification for recommendation]
```

### 12. Submit Review on GitHub

```bash
# For approval
gh pr review [PR_NUMBER] --approve -b "LGTM! [Your review summary]"

# For requesting changes
gh pr review [PR_NUMBER] --request-changes -b "[Your review comments]"

# For comments only
gh pr review [PR_NUMBER] --comment -b "[Your review comments]"
```

### 13. Follow-Up Actions

If changes requested:
- Add inline comments on specific lines
- Suggest code improvements
- Link to relevant documentation
- Offer to pair program if complex

If approved:
- Verify CI passes
- Check merge conflicts
- Ensure branch is up to date
- Approve merge when ready

## Review Principles

### Be Constructive
- Explain *why* something should change
- Suggest alternatives
- Praise good code
- Be respectful and professional

### Be Thorough
- Read every changed line
- Run the code locally
- Test edge cases
- Check for side effects

### Be Consistent
- Apply standards uniformly
- Reference style guides
- Use same criteria for everyone

### Be Timely
- Review within 24 hours
- Don't block unnecessarily
- Prioritize critical PRs

## Common Review Patterns

### Good Patterns to Recognize
```rust
// ✅ Proper error handling with context
pub fn process_data(data: &[u8]) -> Result<Output, ProcessError> {
    validate_input(data)
        .map_err(|e| ProcessError::InvalidInput(e))?;
    
    let parsed = parse_data(data)
        .map_err(|e| ProcessError::ParseFailed(e))?;
    
    Ok(transform(parsed))
}

// ✅ Proper documentation
/// Generates a browser fingerprint from telemetry data.
///
/// # Arguments
///
/// * `telemetry` - Browser telemetry data
///
/// # Returns
///
/// A 32-byte SHA-256 hash
///
/// # Errors
///
/// Returns `FingerprintError::InvalidData` if telemetry is malformed
pub fn generate_fingerprint(
    telemetry: &BrowserTelemetry
) -> Result<[u8; 32], FingerprintError> {
    // ...
}
```

### Anti-Patterns to Catch
```rust
// ❌ Using unwrap
let value = maybe_value.unwrap(); // Stop! Request changes

// ❌ Hardcoded secrets
const API_KEY: &str = "secret-key-123"; // Stop! Security issue

// ❌ Unbounded collections
let mut events = Vec::new();
loop {
    events.push(event); // Could grow infinitely
}

// ❌ Missing error context
Err(Error::Failed) // Not helpful - what failed?
```

## Emergency Fast-Track Reviews

For urgent production fixes:
1. Focus on critical path only
2. Verify fix addresses the issue
3. Check for no regressions
4. Approve quickly if safe
5. Schedule follow-up review for quality improvements

## Pair Review for Complex Changes

For major architectural changes:
1. Schedule synchronous review session
2. Screen share and walk through code
3. Discuss design decisions
4. Identify improvements together
5. Document decisions

Remember: Code review is about ensuring quality, not finding fault. Be helpful, be thorough, and help the team ship excellent code.
