# 🦉 Scrybe

<p align="center">
  <img src="media/image.png" alt="Scrybe - The Vigilant Observer" width="400"/>
</p>

<p align="center">
  <em>"Welcome, traveler. I am Scrybe. You have just gifted me a fingerprint.<br/>
  My task is to remember it, enrich it, and test its truth."</em>
</p>

---

## 🎯 Vision

**Scrybe** is a high-fidelity, Rust-powered browser observation system designed to detect and understand automation with **forensic granularity**. It is equal parts data collector, behavior profiler, and session fingerprint historian—engineered to act as a sophisticated anti-bot detection engine and training ground for resilient bot defenses.

More than a passive observer, Scrybe is a **vigilant system** that watches browsers with contextual memory and scientific rigor. Its mission is not just to block bots—it's to **understand them, adapt to them, and learn from every interaction**.

---

## 🦉 Meet Scrybe

**Species**: Autonomous Rust Intelligence  
**Personality**: Scholarly, curious, and unflinchingly meticulous

Scrybe documents all who visit its domain—not to judge, but to remember. Every movement, header, and anomaly becomes a piece of a broader behavioral mosaic.

- **Humans** find Scrybe charming
- **Bots** find it uncanny

---

## ✨ Key Features

Canvas, WebGL, and audio fingerprinting:
- Multi-layer canvas tests (anti-spoofing)
- Font enumeration patterns
- DOM feature detection
- WebDriver presence analysis

### 🎯 Per-Session Anomaly Detection
ML-driven behavioral baselines:
- Percentile-based thresholds (adaptive)
- Deviation vector flagging
- Fingerprint similarity clustering (MinHash)
- Real-time anomaly scoring

### 🔐 Privacy by Design
GDPR-compliant from the ground up:
- **Zero PII collection**
- Salted hash fingerprints
- Explicit consent for EU visitors
- Data Processing Agreement templates
- 90-day automatic retention

---

## 🏗️ Architecture

```
┌────────────┐     ┌───────────────┐     ┌──────────────────┐     ┌───────────────┐
│  Browser   │ ──> │  Ingestion    │ ──> │  Enrichment & ML │ ──> │  ClickHouse   │
│  (JS SDK)  │     │  Gateway/API  │     │  Fingerprinting  │     │   Storage     │
└────────────┘     └───────────────┘     └──────────────────┘     └───────────────┘
                             │                      │
                             ▼                      ▼
                   ┌────────────────┐     ┌────────────────┐
                   │ Session Cache  │     │  Analyst UI    │
                   │   (Redis)      │     │  Dashboard     │
                   └────────────────┘     └────────────────┘
```

### Tech Stack
- **Core Engine**: Rust (TigerStyle compliant)
- **JavaScript SDK**: TypeScript with bounded collections
- **Storage**: ClickHouse (columnar analytics)
- **Session Cache**: Redis (sub-millisecond lookups)
- **ML Pipeline**: Percentile-based anomaly detection
- **Security**: HMAC-SHA256 auth, TLS 1.3, nonce validation

---

## 📊 Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Ingestion throughput | 100k sessions/sec | 🎯 Designed |
| Query latency (p99) | < 100ms | 🎯 Designed |
| Fingerprint generation | < 5ms | 🎯 Designed |
| Redis lookup | < 1ms | 🎯 Designed |
| Storage compression | 10-20:1 ratio | 🎯 Designed |

---

## 🛡️ Security & Privacy

### Security First
- ✅ HMAC-SHA256 API authentication
- ✅ Anti-replay protection (nonce validation)
- ✅ Bounded collections (DoS prevention)
- ✅ Rate limiting per IP and session
- ✅ Security headers (HSTS, CSP, X-Frame-Options)
- ✅ Graceful degradation (circuit breakers)

### Privacy by Default
- ✅ IP hashing (SHA-256 salted)
- ✅ No PII collection
- ✅ GDPR Article 6(1)(a) compliance
- ✅ Explicit consent for EU visitors
- ✅ Data Processing Agreement templates
- ✅ Right to erasure (delete by fingerprint)
- ✅ 90-day TTL with automatic cleanup

---

## 📚 Documentation

This repository contains comprehensive RFC documentation (v0.2.0):

- **[RFC-0001](docs/rfcs/RFC-0001-architecture.md)** - Core Architecture
- **[RFC-0002](docs/rfcs/RFC-0002-javascript-sdk.md)** - JavaScript SDK (Browser Agent)
- **[RFC-0003](docs/rfcs/RFC-0003-ingestion-gateway.md)** - Rust Ingestion Gateway
- **[RFC-0004](docs/rfcs/RFC-0004-fingerprinting-enrichment.md)** - Fingerprinting & Enrichment
- **[RFC-0005](docs/rfcs/RFC-0005-storage-schema.md)** - ClickHouse Storage Schema
- **[RFC-0006](docs/rfcs/RFC-0006-session-management.md)** - Redis Session Management
- **[RFC-0007](docs/rfcs/RFC-0007-security-privacy.md)** - Security & Privacy

**Additional Resources**:
- [Vision Document](docs/vision.md) - Complete product vision
- [RFC Index](docs/rfcs/RFC-INDEX.md) - Master index
- [Changelog v0.2.0](docs/rfcs/CHANGELOG-v0.2.0.md) - Recent updates
- [Review Summary](docs/rfcs/REVIEW-SUMMARY.md) - Design review findings

---

## 🎨 Design Philosophy: TigerStyle

Scrybe follows **TigerStyle** principles:

1. **Safety First** - No panics, all errors via `Result`
2. **Simplicity** - Clear over clever, explicit over implicit
3. **Correctness** - Type-driven design, >90% test coverage
4. **Performance** - Fast by default, profile before optimizing
5. **Minimal Dependencies** - Each dependency justified

---

## 💰 Cost Model

At **10,000 requests/second** sustained:

| Component | Monthly Cost | Optimization Potential |
|-----------|--------------|------------------------|
| ClickHouse (90-day retention) | $3,200 | 66% with 30-day retention |
| Redis (1-hour session cache) | $1,200 | Optimized |
| Data Transfer | $270 | 90% with 10% sampling |
| Backups (S3) | $700 | - |
| **Total** | **$7,264/month** | **$2,200/month** (optimized) |

---

## 🚀 Current Status

**Version**: v0.2.0 (RFC Phase)  
**Status**: 🎯 **Design Complete** - Ready for Implementation

### Completed
- ✅ Complete RFC suite (7 documents)
- ✅ Multi-disciplinary review (10 expert perspectives)
- ✅ All critical blockers addressed
- ✅ Security hardening (authentication, replay protection)
- ✅ GDPR compliance (consent, DPA templates)
- ✅ Production readiness (health checks, disaster recovery)

### Next Steps
- 🔨 Phase 1: Core infrastructure (Weeks 1-2)
- 🔐 Phase 2: Security features (Weeks 3-4)
- 🧪 Phase 3: SDK & enrichment (Weeks 5-6)
- 💾 Phase 4: Storage & reliability (Weeks 7-8)
- ✅ Phase 5: Testing & hardening (Weeks 9-10)

**Timeline**: 10 weeks to production-ready system

---

## 🤝 Contributing

This is a private repository. Contributions are welcome from authorized collaborators.

### Development Principles
- Follow TigerStyle guidelines
- Maintain >90% test coverage
- Document all public APIs
- No `unwrap()` or `panic!()` in production code
- Explicit error handling with context

---

## 📜 License

Private & Proprietary

---

## 🦉 Philosophy

> "The best defense is not to be invisible, but to be **understood**."

Scrybe doesn't just detect bots—it studies them. Every fingerprint, every behavioral anomaly, every timing quirk becomes part of a living knowledge base. The system learns, adapts, and evolves.

Like its namesake suggests, Scrybe is both **scribe** (recorder of truth) and **scrying** (diviner of hidden meaning). It sees not just what browsers do, but what they *are*.

---

<p align="center">
  <strong>Built with Rust 🦀 | Powered by Curiosity 🦉 | Guided by TigerStyle 🐯</strong>
</p>
