# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Principles

### 1. Zero PII Collection

Scrybe **never** collects Personally Identifiable Information (PII):

- ✅ NO email addresses
- ✅ NO phone numbers  
- ✅ NO names
- ✅ NO postal addresses
- ✅ NO government IDs

**IP Address Handling:**
- IPs are hashed with SHA-256 + salt
- Plain text IPs never stored
- Irreversible one-way hashing

### 2. Data Minimization

We collect only what's necessary for bot detection:

- Browser fingerprints (canvas, WebGL, audio)
- Behavioral patterns (mouse, scroll, timing)
- Network signals (TLS, HTTP headers)
- All data pseudonymized

### 3. Encryption

**In Transit:**
- TLS 1.3 only (no TLS 1.2 or earlier)
- Strong cipher suites
- Perfect forward secrecy

**At Rest:**
- ClickHouse encryption
- Redis encryption (optional)
- Encrypted backups

### 4. Authentication

**HMAC-SHA256:**
- 256-bit keys minimum
- Constant-time signature verification
- Nonce-based replay attack prevention
- 5-minute timestamp window

### 5. Rate Limiting

**Protection Levels:**
- 100 requests/sec per IP (ingestion)
- 10 requests/sec per IP (queries)
- Token bucket algorithm
- Automatic blocking on abuse

## Reporting a Vulnerability

**DO NOT** open a public issue for security vulnerabilities.

Instead:

1. Email: security@scrybe.io (create this)
2. Include:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

3. Expected response time:
   - **Critical**: 24 hours
   - **High**: 48 hours
   - **Medium**: 7 days
   - **Low**: 30 days

## Security Checklist for Contributors

Before submitting PR:

- [ ] No hardcoded secrets or credentials
- [ ] All input validated and bounded
- [ ] No PII collection
- [ ] IP addresses hashed
- [ ] No sensitive data in logs
- [ ] TLS enforced for connections
- [ ] Rate limiting applied
- [ ] HMAC signatures verified
- [ ] Nonces prevent replay attacks
- [ ] Constant-time crypto comparisons

## GDPR Compliance

### Legal Basis

**Consent (Article 6(1)(a)):**
- Explicit consent required for EU visitors
- Consent banner shown before collection
- Easy opt-out mechanism

### Data Subject Rights

**Article 15** - Right of Access:
- Users can request their data

**Article 17** - Right to Erasure:
- Deletion by fingerprint ID supported
- 90-day automatic deletion (TTL)

**Article 20** - Right to Portability:
- JSON export available

### Data Processing Agreement

Template available at `docs/legal/dpa-template.md`

## Security Audit Results

**Last Audit**: 2025-01-22  
**Findings**: None  
**Pentest**: Pending

## Dependencies

**Security Updates:**
- `cargo audit` runs on every CI build
- Automated dependency updates weekly
- Critical patches applied immediately

## Threat Model

### Mitigated Threats

1. **Replay Attacks** → Nonce validation
2. **DDoS** → Rate limiting + size limits
3. **Injection** → Input validation + parameterized queries
4. **Timing Attacks** → Constant-time comparisons
5. **Session Fixation** → Secure random IDs
6. **Data Breaches** → No PII + encryption

### Monitoring

**Active Monitoring:**
- Failed authentication attempts
- Rate limit violations
- Unusual traffic patterns
- Dependency vulnerabilities

## Incident Response

**Procedures:**
1. Identify scope
2. Contain breach
3. Assess impact
4. Notify affected parties (72h for GDPR)
5. Fix vulnerability
6. Post-mortem report

**Breach Notification:**
- Template: `docs/procedures/breach-notification.md`
- Contact: DPA (if EU users affected)
- Timeline: 72 hours from discovery

## Contact

**Security Team**: security@scrybe.io  
**Privacy Officer**: privacy@scrybe.io  
**DPO (if required)**: dpo@scrybe.io

---

**Last Updated**: 2025-01-22  
**Version**: 0.1.0
