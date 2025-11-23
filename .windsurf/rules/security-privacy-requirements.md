---
trigger: always_on
---

# Security & Privacy Requirements

**Activation**: Always On (all files)

## Critical Security Rules

### 1. Zero Trust Input Validation
- **ALL external input MUST be validated at API boundaries**
- No assumptions about data format, size, or content
- Use bounded collections to prevent DoS attacks
- Validate before deserializing, not after

```rust
// ✅ GOOD - Validate before processing
pub fn ingest_telemetry(raw_data: &[u8]) -> Result<Telemetry, IngestionError> {
    // Validate size first
    if raw_data.len() > MAX_TELEMETRY_SIZE {
        return Err(IngestionError::PayloadTooLarge);
    }
    
    // Then deserialize
    let telemetry: Telemetry = serde_json::from_slice(raw_data)
        .map_err(|e| IngestionError::InvalidFormat(e))?;
    
    // Then validate content
    validate_telemetry(&telemetry)?;
    
    Ok(telemetry)
}
```

### 2. Cryptographic Standards
- **HMAC-SHA256** for API authentication (256-bit keys minimum)
- **TLS 1.3** only - no TLS 1.2 or earlier
- Use `ring` or `rustls` crates - never `openssl`
- Constant-time comparisons for secrets: `subtle::ConstantTimeEq`
- Nonce-based replay protection (max 5-minute window)

```rust
use ring::hmac;

pub fn verify_signature(
    message: &[u8],
    signature: &[u8],
    key: &hmac::Key,
) -> Result<(), SecurityError> {
    hmac::verify(key, message, signature)
        .map_err(|_| SecurityError::InvalidSignature)
}
```

### 3. No Secrets in Code
- **NEVER hardcode API keys, tokens, or secrets**
- Load from environment variables or secure vaults
- Use `.env` files for development (not committed)
- Rotate secrets regularly (document schedule)

```rust
// ✅ GOOD - Load from environment
pub fn load_hmac_key() -> Result<hmac::Key, ConfigError> {
    let key_hex = std::env::var("SCRYBE_HMAC_KEY")
        .map_err(|_| ConfigError::MissingHmacKey)?;
    
    let key_bytes = hex::decode(key_hex)
        .map_err(|_| ConfigError::InvalidHmacKey)?;
    
    Ok(hmac::Key::new(hmac::HMAC_SHA256, &key_bytes))
}

// ❌ BAD - Hardcoded secret
const HMAC_KEY: &[u8] = b"super-secret-key-12345"; // NEVER DO THIS!
```

### 4. Zero PII Collection
- **NO collection of personally identifiable information**
- No IP addresses stored in plain text (SHA-256 hash only)
- No user input data logged or stored
- No tracking of specific users across sessions

```rust
use ring::digest;

/// Hash IP address with salt for privacy-preserving storage
pub fn hash_ip(ip: &str, salt: &[u8]) -> String {
    let mut context = digest::Context::new(&digest::SHA256);
    context.update(ip.as_bytes());
    context.update(salt);
    let hash = context.finish();
    hex::encode(hash.as_ref())
}
```

### 5. GDPR Compliance (Article 6(1)(a))
- Explicit consent required for EU visitors
- Consent banner must be shown before collection
- Right to erasure: support deletion by fingerprint ID
- 90-day automatic data retention (TTL)
- Data Processing Agreement (DPA) template provided

```typescript
// JavaScript SDK - Consent check
export class ScrybeSDK {
    async init(config: ScrybeConfig): Promise<void> {
        // Check if EU visitor (via geo-IP or timezone heuristic)
        const isEU = await this.detectEUVisitor();
        
        if (isEU && !config.consentGiven) {
            console.warn('[Scrybe] GDPR consent required. Fingerprinting disabled.');
            return;
        }
        
        // Proceed with initialization
        await this.startFingerprinting();
    }
}
```

## Authentication & Authorization

### API Authentication Pattern
```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// HMAC authentication middleware
pub async fn authenticate(
    req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract timestamp and signature from headers
    let timestamp = extract_timestamp(&req)?;
    let signature = extract_signature(&req)?;
    
    // Verify timestamp freshness (max 5 minutes)
    verify_timestamp(timestamp)?;
    
    // Verify HMAC signature
    let message = build_message(&req, timestamp);
    verify_signature(&message, &signature, &HMAC_KEY)?;
    
    // Check nonce for replay protection
    verify_nonce(&req)?;
    
    Ok(next.run(req).await)
}
```

## Rate Limiting

### Per-IP Rate Limits
- **100 requests per second per IP** (ingestion endpoint)
- **10 requests per second per IP** (query endpoint)
- Use token bucket algorithm
- Return 429 Too Many Requests with Retry-After header

```rust
use governor::{Quota, RateLimiter};

pub struct RateLimitConfig {
    pub ingestion_limit: Quota,
    pub query_limit: Quota,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        use std::num::NonZeroU32;
        
        Self {
            ingestion_limit: Quota::per_second(NonZeroU32::new(100).unwrap()),
            query_limit: Quota::per_second(NonZeroU32::new(10).unwrap()),
        }
    }
}
```

## Security Headers

Always include these headers in HTTP responses:
```rust
use axum::response::Response;

pub fn add_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'none'".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    response
}
```

## Data Sanitization

### SQL Injection Prevention
- Use parameterized queries ONLY
- Never construct SQL with string concatenation
- Use `clickhouse-rs` prepared statements

```rust
// ✅ GOOD - Parameterized query
let query = "SELECT * FROM sessions WHERE fingerprint_hash = ?";
client.query(query).bind(&fingerprint_hash).fetch_all().await?;

// ❌ BAD - SQL injection risk
let query = format!("SELECT * FROM sessions WHERE fingerprint_hash = '{}'", fingerprint_hash);
```

### XSS Prevention
- Escape all user-provided data in UI
- Use React's built-in escaping (don't use `dangerouslySetInnerHTML`)
- Content-Type: application/json for API responses

## Logging & Monitoring

### Secure Logging
- **NEVER log sensitive data**: API keys, signatures, full IPs
- Log hashed identifiers only
- Use structured logging: `tracing` crate
- Rotate logs daily, retain for 30 days

```rust
use tracing::{info, warn, error};

// ✅ GOOD - Safe logging
info!(
    session_id = %session.id,
    fingerprint_hash = %session.fingerprint_hash,
    "Session created"
);

// ❌ BAD - Logging sensitive data
info!("Session created with IP: {}", client_ip); // Don't log IPs!
```

## Vulnerability Management

### Regular Security Audits
- Run `cargo audit` before every release
- Update dependencies monthly (security patches immediately)
- Monitor CVE databases for Rust ecosystem
- Conduct penetration testing before production

### Incident Response
- Security incident playbook documented
- Contact: security@scrybe.io (create this email)
- 24-hour response time for critical vulnerabilities
- Public disclosure after 90 days or fix deployment

## Threat Model Awareness

### Known Attack Vectors
1. **Replay attacks** - Mitigated by nonce validation
2. **DDoS via large payloads** - Mitigated by size limits
3. **Timing attacks** - Mitigated by constant-time comparisons
4. **Enumeration attacks** - Mitigated by rate limiting
5. **Session fixation** - Mitigated by secure random session IDs

### Defense in Depth
- Multiple layers of validation
- Graceful degradation under attack
- Circuit breakers for external services
- Automatic backoff and retry logic

## Privacy Policy Compliance

When implementing features:
- Consider GDPR, CCPA, LGPD implications
- Document what data is collected and why
- Provide opt-out mechanisms
- Honor Do Not Track headers (optional but recommended)
- Support data export (JSON format)
- Support data deletion (by fingerprint ID)

## Code Review Checklist

Before approving code:
- [ ] No hardcoded secrets or credentials
- [ ] All input validated and bounded
- [ ] Cryptography uses approved libraries
- [ ] Error messages don't leak sensitive info
- [ ] Logging doesn't contain PII
- [ ] Rate limiting applied to public endpoints
- [ ] Security headers set correctly
- [ ] SQL queries parameterized
- [ ] TLS enforced for all connections
- [ ] Tests cover security edge cases
