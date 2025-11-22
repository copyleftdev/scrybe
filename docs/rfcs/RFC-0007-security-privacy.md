# RFC-0007: Security & Privacy

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Depends On**: All previous RFCs v0.2.0
- **Review**: Updated IP anonymization, added GDPR consent requirements, DPA templates, breach procedures
- **Style**: TigerStyle

## Summary

Scrybe is designed with privacy and security as foundational principles. While it collects browser signals for bot detection, it must:
1. **Never collect PII** (personally identifiable information)
2. **Respect user privacy** (opt-out mechanisms)
3. **Secure data in transit and at rest**
4. **Prevent abuse** (rate limiting, DDoS protection)
5. **Be transparent** (clear documentation, privacy policy)

This RFC defines all security and privacy safeguards.

## Privacy Philosophy

**Core Principle**: Scrybe observes *behavior patterns*, not *identities*.

We distinguish between:
- ✅ **Allowed**: Behavioral fingerprints, browser capabilities, timing patterns
- ❌ **Forbidden**: Input field values, form data, personal information, cookies

## PII Protection

### What We NEVER Collect

```rust
// ❌ NEVER collect these:
- Input field values (text fields, passwords, forms)
- Form submissions
- URLs with query parameters (may contain tokens/IDs)
- Cookies (except our own session cookie)
- localStorage/sessionStorage keys or values (except our own)
- Full user-agent (only parsed components)
- Exact geolocation (only city-level, anonymized)
- Email addresses
- Phone numbers
- Names
- Social security numbers
- Credit card numbers
```

### What We DO Collect

```rust
// ✅ Safe to collect (non-identifying):
- Canvas/WebGL/Audio fingerprints (hashes only)
- Browser capabilities (fonts, plugins, screen size)
- Network timing (connection speed, latency)
- Behavioral patterns (mouse entropy, scroll smoothness)
- IP address (anonymized: last octet masked)
- Generalized geo (country, city only)
- HTTP headers (standard headers only, no custom headers with tokens)
```

## Data Minimization

### JavaScript SDK

```typescript
// ❌ NEVER track input values
document.addEventListener('input', (e) => {
  // DO NOT: track e.target.value
  // DO NOT: track e.target.name
  
  // ✅ ONLY track interaction occurred (no value)
  interactionCount++;
});

// ❌ NEVER track which elements were clicked
document.addEventListener('click', (e) => {
  // DO NOT: track e.target.id
  // DO NOT: track e.target.className
  // DO NOT: track element content
  
  // ✅ ONLY track click coordinates and timing
  clicks.push({
    x: e.clientX,
    y: e.clientY,
    timestamp: Date.now(),
  });
});

// ❌ NEVER track URLs with parameters
function sanitizeUrl(url: string): string {
  const parsed = new URL(url);
  // Remove query params and hash
  return `${parsed.origin}${parsed.pathname}`;
}
```

### Rust Ingestion

```rust
// Hash IP addresses (GDPR-compliant anonymization)
fn anonymize_ip(ip: IpAddr, salt: &Secret<String>) -> String {
    // Use salted hash instead of masking
    // Per GDPR Article 29 WP: masking last octet is NOT anonymous
    
    let mut hasher = Sha256::new();
    hasher.update(salt.expose().as_bytes());
    hasher.update(ip.to_string().as_bytes());
    
    // Return hash (irreversible)
    format!("{:x}", hasher.finalize())
}

// Example: Store hash, not raw IP
pub struct AnonymizedSession {
    pub ip_hash: String,  // SHA-256(salt + IP)
    // NO raw IP stored
}

// Redact sensitive headers
fn sanitize_headers(headers: &HeaderMap) -> Vec<HttpHeader> {
    const SAFE_HEADERS: &[&str] = &[
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        // NOT: "authorization", "cookie", "x-api-key", etc.
    ];
    
    headers
        .iter()
        .filter(|(name, _)| SAFE_HEADERS.contains(&name.as_str()))
        .map(|(name, value)| HttpHeader {
            name: name.to_string(),
            value: value.to_str().unwrap_or("").to_string(),
        })
        .collect()
}
```

## GDPR Consent Requirements

### Legal Basis

**Critical**: Browser fingerprints are **personal data** under GDPR (Recital 30).

**Options for legal basis**:
1. **Consent** (Article 6(1)(a)) - User explicitly agrees
2. **Legitimate Interest** (Article 6(1)(f)) - Requires balancing test

**Recommendation**: Use **consent** for EU visitors to avoid legal risk.

### Consent Implementation

```typescript
// MUST obtain explicit consent in EU
if (isEUVisitor()) {
  // Show consent banner
  const consent = await showConsentBanner({
    purpose: 'Bot detection and security',
    data_collected: [
      'Browser fingerprint (anonymized)',
      'Mouse movement patterns',
      'Session interactions'
    ],
    retention: '90 days',
    processor: 'Scrybe Bot Detection',
  });
  
  if (!consent) {
    // Don't initialize SDK
    return;
  }
}

// Initialize SDK only after consent
Scrybe.init({ ... });
```

### Geolocation-Based Consent

```rust
// Server-side: Check if visitor from EU
fn requires_consent(ip: IpAddr, geoip: &GeoIpDatabase) -> bool {
    const EU_COUNTRIES: &[&str] = &[
        "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR",
        "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL",
        "PL", "PT", "RO", "SK", "SI", "ES", "SE"
    ];
    
    if let Ok(geo) = geoip.lookup(ip) {
        EU_COUNTRIES.contains(&geo.country.as_str())
    } else {
        // If unknown, assume requires consent (safe default)
        true
    }
}
```

## Opt-Out Mechanisms

### Client-Side Opt-Out

```typescript
// Check multiple opt-out signals
function shouldOptOut(): boolean {
  // 1. Global JavaScript flag
  if (window.scrybeOptOut === true) {
    return true;
  }
  
  // 2. Do Not Track header
  if (navigator.doNotTrack === '1') {
    return true;
  }
  
  // 3. Meta tag
  if (document.querySelector('meta[name="scrybe-opt-out"]')) {
    return true;
  }
  
  // 4. localStorage opt-out
  if (localStorage.getItem('scrybe_opted_out') === 'true') {
    return true;
  }
  
  return false;
}

// Respect opt-out immediately
if (shouldOptOut()) {
  console.log('Scrybe: User opted out, not collecting data');
  return;  // Exit SDK initialization
}
```

### Server-Side Opt-Out

```rust
fn check_opt_out(headers: &HeaderMap) -> bool {
    // Check DNT header
    if let Some(dnt) = headers.get("dnt") {
        if dnt == "1" {
            return true;
        }
    }
    
    // Check custom opt-out header
    if let Some(optout) = headers.get("x-scrybe-opt-out") {
        if optout == "true" {
            return true;
        }
    }
    
    false
}

// In ingestion handler
if check_opt_out(&headers) {
    return Ok(StatusCode::NO_CONTENT);  // 204 No Content
}
```

### Opt-Out Storage

```sql
-- ClickHouse: Don't store opted-out sessions
CREATE TABLE scrybe.opt_outs (
    ip_hash FixedString(32),        -- SHA-256 of IP
    opted_out_at DateTime,
    user_agent_hash FixedString(32) -- For correlation
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(opted_out_at)
ORDER BY (ip_hash, opted_out_at)
TTL opted_out_at + INTERVAL 365 DAY;
```

## Cryptographic Hashing

### Salted Hashes for Identifiers

```rust
use sha2::{Sha256, Digest};

pub struct HashSalt {
    salt: String,
}

impl HashSalt {
    /// Generate new salt (rotate every 30 days).
    pub fn new() -> Self {
        Self {
            salt: Uuid::new_v4().to_string(),
        }
    }
    
    /// Hash identifier with salt.
    pub fn hash(&self, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.salt.as_bytes());
        hasher.update(value.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Rotate salt (new salt = old hashes invalid).
    pub fn rotate(&mut self) {
        self.salt = Uuid::new_v4().to_string();
    }
}

// Usage
let salt = HashSalt::new();
let ip_hash = salt.hash(&ip.to_string());           // Store hash, not IP
let fp_hash = salt.hash(&fingerprint.to_string());  // Store hash, not raw FP
```

### Salt Rotation

```rust
// Rotate salts every 30 days (cron job)
pub async fn rotate_salts() -> Result<(), ScrybeError> {
    let mut salt = load_current_salt()?;
    
    tracing::info!("Rotating hash salt");
    
    salt.rotate();
    save_salt(&salt)?;
    
    // Note: Old hashes become unrecognizable (privacy-preserving)
    // This breaks long-term tracking (intentional)
    
    Ok(())
}
```

## Data Processing Agreement (DPA)

### DPA Template

If Scrybe is embedded in customer websites, both parties are **joint controllers** under GDPR.

**Required DPA clauses**:

```markdown
# Data Processing Agreement - Scrybe Bot Detection

## 1. Definitions
- **Controller**: Website operator (Customer)
- **Processor**: Scrybe Bot Detection Service
- **Personal Data**: Browser fingerprints, behavioral patterns

## 2. Scope of Processing
- **Purpose**: Bot detection and security
- **Data Categories**: Browser fingerprints, mouse movements, session metadata
- **Data Subjects**: Website visitors
- **Retention**: 90 days maximum

## 3. Processor Obligations
- Process data only on documented instructions
- Ensure confidentiality of personnel
- Implement appropriate technical measures (encryption, access controls)
- Assist with data subject requests (within 48 hours)
- Delete or return data on termination

## 4. Sub-Processors
- Cloud provider: [AWS/GCP]
- Prior notification required for new sub-processors

## 5. Data Subject Rights
- Right to access (export JSON)
- Right to erasure (delete by fingerprint hash)
- Right to data portability
- Response time: 30 days maximum

## 6. Security Measures
- TLS 1.3 for data in transit
- Encryption at rest (AES-256)
- Access controls (least privilege)
- Annual security audits

## 7. Data Breach Notification
- Processor notifies Controller within 24 hours
- Controller notifies supervisory authority within 72 hours
- Processor assists with breach notification

## 8. Liability
- Each party liable for its own GDPR violations
- Indemnification for breaches caused by other party
```

## Cross-Border Data Transfers

### Standard Contractual Clauses (SCCs)

If ClickHouse servers are outside EU:

```markdown
# Standard Contractual Clauses (2021 version)

## Module Two: Controller to Processor

### Clause 1: Purpose and scope
Data exporter: EU-based website operator
Data importer: Scrybe (US-based processor)

### Clause 7: Docking clause
Third parties may accede to these Clauses.

### Clause 8: Data protection safeguards
- Data minimization: Only fingerprints, no PII
- Encryption: TLS 1.3 in transit, AES-256 at rest
- Access controls: Role-based access (RBAC)
- Logging: All access logged and monitored

### Clause 13: Supervision
Supervising authority: [Irish DPC if EU subsidiary]

### Clause 14: Local laws and practices
Data importer certifies no conflicting US obligations
Transparency report if government requests received
```

### Alternative: EU Hosting

```
Recommendation: Host ClickHouse in EU region
- AWS: eu-central-1 (Frankfurt) or eu-west-1 (Ireland)
- GCP: europe-west1 (Belgium) or europe-west3 (Frankfurt)

Benefit: Avoid SCC complexity, data stays in EU
```

## Data Breach Notification

### Procedure (GDPR Article 33/34)

```rust
/// Data breach response procedure
pub async fn handle_data_breach(
    breach: &BreachIncident,
) -> Result<(), ScrybeError> {
    // 1. Contain breach immediately
    disable_affected_systems().await?;
    
    // 2. Assess severity
    let severity = assess_breach_severity(breach);
    
    // 3. Notify controller (customer) within 24 hours
    notify_controller(breach, severity).await?;
    
    // 4. Document breach
    log_breach_to_register(breach).await?;
    
    // 5. If high risk: Help controller notify supervisory authority (within 72h)
    if severity == BreachSeverity::HighRisk {
        prepare_authority_notification(breach).await?;
    }
    
    // 6. If very high risk: Help notify affected data subjects
    if severity == BreachSeverity::VeryHighRisk {
        prepare_subject_notification(breach).await?;
    }
    
    Ok(())
}

#[derive(Debug)]
pub enum BreachSeverity {
    Low,       // No notification required
    HighRisk,  // Notify authority within 72h
    VeryHighRisk, // Also notify data subjects
}

fn assess_breach_severity(breach: &BreachIncident) -> BreachSeverity {
    // Factors:
    // - Type of data (fingerprints vs PII)
    // - Number of affected individuals
    // - Consequences (identity theft risk?)
    // - Special categories of data (none in our case)
    
    if breach.affected_count > 10_000 {
        BreachSeverity::VeryHighRisk
    } else if breach.data_exposed.contains("raw_ip") {
        BreachSeverity::HighRisk
    } else {
        BreachSeverity::Low
    }
}
```

### Breach Register

```sql
-- Maintain breach register (GDPR Article 33(5))
CREATE TABLE scrybe.breach_register (
    breach_id UUID PRIMARY KEY,
    detected_at TIMESTAMP,
    contained_at TIMESTAMP,
    breach_type VARCHAR,           -- 'unauthorized_access', 'data_leak', etc.
    affected_records BIGINT,
    data_categories VARCHAR[],     -- ['fingerprints', 'session_metadata']
    root_cause TEXT,
    remediation TEXT,
    controller_notified_at TIMESTAMP,
    authority_notified_at TIMESTAMP,
    subjects_notified_at TIMESTAMP,
    status VARCHAR                 -- 'contained', 'investigating', 'resolved'
);
```

## Data Retention & Deletion

### Automatic TTL

```sql
-- ClickHouse: Auto-delete old sessions
ALTER TABLE scrybe.sessions 
MODIFY TTL timestamp + INTERVAL 90 DAY;

-- Redis: Auto-expire cached sessions
EXPIRE session:123e4567... 86400  -- 24 hours
```

### Manual Deletion (GDPR Right to Erasure)

```rust
pub async fn delete_user_data(
    fingerprint_hash: &str,
    store: &SessionStore,
) -> Result<(), ScrybeError> {
    // Delete from ClickHouse
    store.client
        .query("DELETE FROM scrybe.sessions WHERE fingerprint_hash = ?")
        .bind(fingerprint_hash)
        .execute()
        .await?;
    
    // Delete from Redis
    let mut conn = store.cache.pool.get().await?;
    let key = format!("fingerprint:{}", fingerprint_hash);
    conn.del(&key).await?;
    
    tracing::info!("Deleted user data for fingerprint: {}", fingerprint_hash);
    
    Ok(())
}
```

## Transport Security

### TLS Requirements

```rust
// Require TLS for all connections
#[derive(Clone)]
pub struct TlsConfig {
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl TlsConfig {
    pub fn load(&self) -> Result<ServerConfig, ScrybeError> {
        let certs = load_certs(&self.cert_path)?;
        let key = load_private_key(&self.key_path)?;
        
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| ScrybeError::TlsError(e.to_string()))?;
        
        Ok(config)
    }
}

// Reject non-HTTPS in production
async fn enforce_https(req: Request<Body>) -> Result<Request<Body>, StatusCode> {
    if req.uri().scheme_str() != Some("https") {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(req)
}
```

### Secure Cookies

```typescript
// JavaScript SDK: Secure session cookie
function setSessionCookie(sessionId: string) {
  document.cookie = `scrybe_session=${sessionId}; ` +
    `Secure; ` +                    // Only over HTTPS
    `HttpOnly; ` +                  // Not accessible via JS
    `SameSite=Strict; ` +           // CSRF protection
    `Max-Age=86400; ` +             // 24 hours
    `Path=/`;
}
```

## Rate Limiting & DDoS Protection

### Multi-Layer Rate Limiting

```rust
pub struct RateLimitConfig {
    pub per_ip_limit: u32,           // 100 req/min
    pub per_session_limit: u32,      // 1000 req/min
    pub global_limit: u32,           // 100k req/min
}

impl RateLimiter {
    pub async fn check_all(&self, ip: IpAddr, session_id: Uuid) -> Result<(), ScrybeError> {
        // Check per-IP limit (prevent single-IP floods)
        self.check_ip_limit(ip).await?;
        
        // Check per-session limit (prevent session abuse)
        self.check_session_limit(session_id).await?;
        
        // Check global limit (prevent total overload)
        self.check_global_limit().await?;
        
        Ok(())
    }
}
```

### IP Blacklisting

```rust
pub struct IpBlacklist {
    cache: Arc<RedisCache>,
}

impl IpBlacklist {
    pub async fn is_blacklisted(&self, ip: IpAddr) -> Result<bool, ScrybeError> {
        let mut conn = self.cache.pool.get().await?;
        let key = format!("blacklist:ip:{}", ip);
        
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }
    
    pub async fn add(&self, ip: IpAddr, reason: &str, duration: Duration) -> Result<(), ScrybeError> {
        let mut conn = self.cache.pool.get().await?;
        let key = format!("blacklist:ip:{}", ip);
        
        conn.set_ex(&key, reason, duration.as_secs() as usize).await?;
        
        tracing::warn!("Blacklisted IP {} for {}: {}", ip, duration.as_secs(), reason);
        
        Ok(())
    }
}
```

## Input Validation

### Strict Validation Rules

```rust
impl IngestRequest {
    pub fn validate(&self) -> Result<(), ScrybeError> {
        // Timestamp validation
        if self.timestamp > Utc::now() {
            return Err(ScrybeError::InvalidSession {
                field: "timestamp".to_string(),
                reason: "Timestamp cannot be in future".to_string(),
            });
        }
        
        if (Utc::now() - self.timestamp).num_hours() > 24 {
            return Err(ScrybeError::InvalidSession {
                field: "timestamp".to_string(),
                reason: "Timestamp too old (> 24 hours)".to_string(),
            });
        }
        
        // Payload size validation
        let serialized = serde_json::to_string(self)
            .map_err(|e| ScrybeError::ValidationError(e.to_string()))?;
        
        if serialized.len() > 1_000_000 {  // 1MB max
            return Err(ScrybeError::InvalidSession {
                field: "payload".to_string(),
                reason: "Payload too large (> 1MB)".to_string(),
            });
        }
        
        // Validate all nested structures
        self.network.validate()?;
        self.browser.validate()?;
        self.behavioral.validate()?;
        
        Ok(())
    }
}
```

### Sanitization

```rust
fn sanitize_string(s: &str, max_len: usize) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .take(max_len)
        .collect()
}

fn sanitize_user_agent(ua: &str) -> String {
    // Remove potentially identifying tokens
    let sanitized = sanitize_string(ua, 500);
    
    // Parse and reconstruct (removes malformed parts)
    // Using user-agent parser crate
    sanitized
}
```

## Audit Logging

### Security Events

```rust
pub enum SecurityEvent {
    RateLimitExceeded { ip: IpAddr },
    InvalidRequest { ip: IpAddr, reason: String },
    OptOutRequested { ip_hash: String },
    DataDeletionRequested { fingerprint_hash: String },
    SuspiciousActivity { ip: IpAddr, description: String },
}

impl SecurityEvent {
    pub async fn log(&self, store: &AuditLog) -> Result<(), ScrybeError> {
        match self {
            SecurityEvent::RateLimitExceeded { ip } => {
                tracing::warn!("Rate limit exceeded: {}", ip);
                store.insert("rate_limit", &json!({ "ip": ip.to_string() })).await?;
            }
            SecurityEvent::SuspiciousActivity { ip, description } => {
                tracing::error!("Suspicious activity from {}: {}", ip, description);
                store.insert("suspicious", &json!({ 
                    "ip": ip.to_string(),
                    "description": description 
                })).await?;
            }
            // ... other events
        }
        Ok(())
    }
}
```

### Audit Trail

```sql
-- ClickHouse: Security audit log
CREATE TABLE scrybe.audit_log (
    event_time DateTime,
    event_type LowCardinality(String),
    ip_hash FixedString(32),
    details String,                  -- JSON
    severity LowCardinality(String)  -- "info", "warning", "error"
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_time)
ORDER BY (event_time, event_type)
TTL event_time + INTERVAL 365 DAY;
```

## Compliance

### GDPR Compliance

```rust
// Right to Access
pub async fn export_user_data(
    fingerprint_hash: &str,
    store: &SessionStore,
) -> Result<Vec<Session>, ScrybeError> {
    let sessions = store.query_by_fingerprint(fingerprint_hash).await?;
    Ok(sessions)
}

// Right to Erasure
pub async fn delete_user_data(
    fingerprint_hash: &str,
    store: &SessionStore,
) -> Result<(), ScrybeError> {
    store.delete_by_fingerprint(fingerprint_hash).await?;
    Ok(())
}

// Right to Portability
pub async fn export_json(
    fingerprint_hash: &str,
    store: &SessionStore,
) -> Result<String, ScrybeError> {
    let sessions = export_user_data(fingerprint_hash, store).await?;
    let json = serde_json::to_string_pretty(&sessions)?;
    Ok(json)
}
```

### Privacy Policy

```markdown
# Scrybe Privacy Policy

## What We Collect
- Browser fingerprints (hashes only, not reversible)
- Behavioral patterns (mouse movement entropy, scroll patterns)
- Network metadata (anonymized IP, connection timing)
- Device capabilities (screen size, available fonts)

## What We Don't Collect
- Input field values or form submissions
- Personal information (names, emails, phone numbers)
- Exact geolocation (only city-level)
- Cookies from other websites
- Browsing history or URLs with parameters

## How to Opt Out
Set `window.scrybeOptOut = true` before our script loads, or
Enable "Do Not Track" in your browser settings.

## Data Retention
Session data is automatically deleted after 90 days.
Aggregated statistics are kept for 1 year.

## Your Rights (GDPR)
- Right to access your data
- Right to delete your data
- Right to data portability
Contact: privacy@scrybe.example.com
```

## Security Testing

### Penetration Testing Checklist

```bash
# SQL Injection
curl -X POST https://api.scrybe.com/v1/ingest \
  -d '{"sessionId": "123; DROP TABLE sessions;--"}'

# XSS
curl -X POST https://api.scrybe.com/v1/ingest \
  -d '{"sessionId": "<script>alert(1)</script>"}'

# Oversized Payload
dd if=/dev/zero bs=1M count=10 | curl -X POST \
  https://api.scrybe.com/v1/ingest --data-binary @-

# Rate Limit Bypass
for i in {1..1000}; do
  curl -X POST https://api.scrybe.com/v1/ingest \
    -H "X-Forwarded-For: 1.2.3.$i" -d '{}'
done
```

### Automated Security Scanning

```bash
# Dependency vulnerability scanning
cargo audit

# SAST (Static Analysis Security Testing)
cargo clippy -- -D warnings

# Container scanning (if using Docker)
trivy image scrybe:latest
```

## Incident Response

### Security Incident Procedure

```rust
pub enum IncidentSeverity {
    Low,      // Rate limit exceeded
    Medium,   // Suspicious pattern detected
    High,     // Potential data breach
    Critical, // Active attack in progress
}

pub async fn handle_security_incident(
    severity: IncidentSeverity,
    description: &str,
) -> Result<(), ScrybeError> {
    match severity {
        IncidentSeverity::Critical => {
            // 1. Alert on-call team
            alert_pagerduty("CRITICAL: {}", description).await?;
            
            // 2. Enable aggressive rate limiting
            enable_emergency_mode().await?;
            
            // 3. Log detailed forensics
            tracing::error!("CRITICAL INCIDENT: {}", description);
        }
        
        IncidentSeverity::High => {
            // Alert team during business hours
            alert_slack("HIGH: {}", description).await?;
            tracing::error!("HIGH INCIDENT: {}", description);
        }
        
        _ => {
            tracing::warn!("INCIDENT ({}): {}", severity, description);
        }
    }
    
    Ok(())
}
```

## Pseudonymization vs Anonymization

**GDPR Definitions**:
- **Anonymization**: Irreversible, data no longer "personal data"
- **Pseudonymization**: Reversible with additional information, still "personal data"

**Scrybe Approach**:
```
Fingerprint hashes: Pseudonymization (could re-identify with browser)
IP hashes (salted): Pseudonymization (salt rotation → anonymization)
Behavioral patterns: Pseudonymization

→ Still subject to GDPR even with hashing
```

**Path to True Anonymization**:
1. Salt rotation every 30 days (breaks re-identification)
2. Delete correlation data (fingerprint → user mapping)
3. Aggregate to population level (remove individual patterns)

## Success Criteria

1. ✅ Zero PII collected
2. ✅ Opt-out mechanisms working
3. ✅ All data encrypted in transit (TLS 1.3)
4. ✅ Data retention enforced (TTL)
5. ✅ Rate limiting prevents abuse
6. ✅ Input validation prevents injection
7. ✅ Audit log captures security events
8. ✅ GDPR compliance (access, erasure, portability)
9. ✅ **NEW**: IP hashing (not masking)
10. ✅ **NEW**: Explicit consent for EU visitors
11. ✅ **NEW**: DPA template provided
12. ✅ **NEW**: Cross-border transfer safeguards (SCCs or EU hosting)
13. ✅ **NEW**: Data breach procedure documented
14. ✅ **NEW**: Breach register maintained

## References

- RFC-0001: Core Architecture
- RFC-0002: JavaScript SDK
- RFC-0003: Ingestion Gateway
- GDPR: https://gdpr.eu/
- OWASP Top 10: https://owasp.org/www-project-top-ten/
- Web Privacy: https://www.w3.org/TR/fingerprinting-guidance/
