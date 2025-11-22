# Scrybe RFC Index

**Project**: Scrybe - Browser Behavior Intelligence System  
**Style**: TigerStyle (following TigerBeetle principles)  
**Version**: 0.2.0  
**Status**: Post-Review Refinement → Ready for Implementation  
**Last Updated**: 2025-01-22

---

## 🎉 Version 0.2.0 Update

**All RFCs have been refined based on comprehensive multi-disciplinary review!**

**Critical blockers addressed**:
- ✅ API authentication (HMAC signatures)
- ✅ Replay attack prevention (nonce validation)
- ✅ DoS protection (bounded collections)
- ✅ GDPR compliance (cookie consent)
- ✅ Graceful shutdown
- ✅ Secrets management

**See [CHANGELOG-v0.2.0.md](./CHANGELOG-v0.2.0.md) for complete details.**

---

## Project Overview

Scrybe is a high-fidelity browser behavior intelligence system designed to detect and understand automation with forensic granularity. It acts as both a data collector and behavior profiler, enabling real-time bot detection, historical session analysis, and ML model training.

### Core Components

1. **JavaScript SDK** - Browser-side signal collection agent
2. **Rust Ingestion Gateway** - High-performance HTTP API for receiving telemetry
3. **Enrichment Pipeline** - Fingerprinting, geo-resolution, anomaly detection
4. **ClickHouse Storage** - Analytical database for session telemetry
5. **Redis Cache** - Fast session correlation and rate limiting
6. **Analyst UI** - React dashboard for visualization and analysis

---

## Completed RFCs

### ✅ RFC-0001: Core Architecture
**File**: `RFC-0001-architecture.md` (10KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines the overall system architecture for Scrybe:
- **System design**: Multi-layer signal collection (network, browser, behavioral)
- **Module structure**: Rust crates organization (core, ingestion, enrichment, storage, cache)
- **Data flow**: Browser → Ingestion → Enrichment → Storage
- **Core types**: Session, NetworkSignals, BrowserSignals, BehavioralSignals, Fingerprint
- **Performance targets**: < 5ms ingestion, < 50ms enrichment
- **Dependencies**: Axum, ClickHouse, Redis, serde, chrono
- **TigerStyle compliance**: Safety, simplicity, correctness, performance

**Key Decisions**:
- Rust for performance and safety
- ClickHouse for analytical queries
- Redis for real-time session cache
- JavaScript SDK for client-side collection

---

### ✅ RFC-0002: JavaScript SDK (Browser Collection Agent)
**File**: `RFC-0002-javascript-sdk.md` (15KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines the browser-side collection agent:
- **Signal categories**: Network, browser, behavioral
- **Canvas fingerprinting**: SHA-256 hash of canvas rendering
- **WebGL fingerprinting**: Vendor, renderer, extensions
- **Audio fingerprinting**: Audio context signature
- **Behavioral collection**: Mouse entropy, scroll smoothness, timing patterns
- **Transport**: Beacon API (primary), Fetch API (fallback)
- **Privacy safeguards**: No PII, no input values, opt-out support
- **Performance**: < 20ms initialization, < 30KB bundle size
- **Browser support**: Chrome 90+, Firefox 88+, Safari 14+

**Key Features**:
- Non-blocking async collection
- Comprehensive signal coverage
- Privacy-aware (no PII tracking)
- Lightweight and fast

---

### ✅ RFC-0003: Rust Ingestion Gateway
**File**: `RFC-0003-ingestion-gateway.md` (14KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines the HTTP ingestion service built with Axum:
- **API endpoint**: POST /api/v1/ingest
- **Server-side signals**: IP, TLS (JA3/JA4), HTTP headers
- **Middleware stack**: Tracing, CORS, rate limiting, request ID
- **Validation**: Payload schema, timestamp checks, size limits
- **Error handling**: Type-safe errors, appropriate status codes
- **Rate limiting**: 100 req/min per IP, 1000 req/min per session
- **Performance**: < 5ms latency (p99), 10k req/sec throughput
- **Observability**: Structured logging, Prometheus metrics

**Key Technologies**:
- Axum (web framework)
- Tower (middleware)
- tokio (async runtime)
- redis (session cache)

---

### ✅ RFC-0004: Fingerprinting & Enrichment Pipeline
**File**: `RFC-0004-fingerprinting-enrichment.md` (17KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines the enrichment pipeline:
- **Composite fingerprinting**: Deterministic device identification (SHA-256)
- **Component hashes**: Network, browser, behavioral, device
- **Geo/ASN resolution**: MaxMind GeoIP2 integration
- **MinHash similarity**: Fingerprint clustering via LSH
- **Anomaly detection**: Behavioral, timing, header, fingerprint anomalies
- **Bot probability scoring**: Weighted combination of anomaly scores
- **Pipeline stages**: Fingerprint → Geo → Similarity → Anomaly → Assembly
- **Performance**: < 50ms enrichment (p99)
- **Caching**: GeoIP lookups cached in memory

**Key Algorithms**:
- SHA-256 for deterministic hashing
- MinHash for similarity detection
- Jaccard similarity for clustering
- ML-based anomaly detection

---

### ✅ RFC-0005: Storage Schema (ClickHouse)
**File**: `RFC-0005-storage-schema.md` (16KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines the ClickHouse schema and queries:
- **Main table**: `sessions` (partitioned by date, TTL 90 days)
- **Primary key**: (timestamp, session_id)
- **Indexes**: Bloom filter (fingerprint), token bloom (IP), minmax (bot_probability)
- **Materialized views**: hourly_stats, fingerprint_clusters, anomaly_events
- **Rust integration**: clickhouse-rs client, batch writes
- **Query patterns**: Time-range scans, fingerprint lookups, anomaly aggregations
- **Performance**: 100k writes/sec, < 100ms queries (p99)
- **Compression**: > 50:1 ratio with zstd

**Key Features**:
- Columnar storage for fast analytics
- Automatic TTL-based cleanup
- Materialized views for common queries
- High-cardinality optimization

---

### ✅ RFC-0006: Session Management (Redis)
**File**: `RFC-0006-session-management.md` (14KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines Redis-based session cache:
- **Data structures**: Hash (session metadata), Set (fingerprint → sessions), String (rate limits)
- **Key naming**: `session:{uuid}`, `fingerprint:{hash}`, `ratelimit:{ip}:{window}`
- **TTL strategy**: 24 hours for sessions, 60 seconds for rate limits
- **Session correlation**: Fingerprint + IP matching
- **Rate limiting**: Per-IP counters with sliding windows
- **Anomaly feed**: Sorted sets for real-time alerts
- **Performance**: < 1ms latency (p99)
- **Connection pooling**: deadpool-redis

**Key Operations**:
- Fast session lookups
- Real-time fingerprint correlation
- Rate limit enforcement
- Anomaly event streaming

---

### ✅ RFC-0007: Security & Privacy
**File**: `RFC-0007-security-privacy.md` (12KB)  
**Status**: Complete  
**Created**: 2025-01-22

Defines all security and privacy safeguards:
- **PII protection**: Never collect input values, form data, personal info
- **Opt-out mechanisms**: DNT header, JavaScript flag, meta tag, localStorage
- **Data minimization**: Only collect behavioral patterns, not identities
- **Cryptographic hashing**: Salted SHA-256 for all identifiers
- **Salt rotation**: Every 30 days (breaks long-term tracking)
- **Data retention**: Auto-delete after 90 days (ClickHouse TTL)
- **GDPR compliance**: Right to access, erasure, portability
- **Transport security**: TLS required, secure cookies
- **Rate limiting**: Multi-layer (IP, session, global)
- **Input validation**: Strict schema validation, size limits
- **Audit logging**: Security events tracked

**Privacy Principles**:
- Observe behavior, not identity
- No PII collection
- Transparent data practices
- User control (opt-out)

---

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)
**Goal**: Working end-to-end pipeline

- [ ] Set up Rust workspace
- [ ] Implement core types (Session, Fingerprint, etc.)
- [ ] Build ingestion gateway (Axum)
- [ ] Redis cache integration
- [ ] ClickHouse schema setup

**Deliverables**:
- Ingestion API accepting session data
- Redis session cache working
- ClickHouse writes successful

---

### Phase 2: Enrichment Pipeline (Weeks 3-4)
**Goal**: Fingerprinting and anomaly detection

- [ ] Implement composite fingerprinting
- [ ] GeoIP integration (MaxMind)
- [ ] MinHash similarity engine
- [ ] Anomaly detection algorithms
- [ ] Enrichment pipeline executor

**Deliverables**:
- Deterministic fingerprints generated
- Geo/ASN enrichment working
- Anomaly scores calculated

---

### Phase 3: JavaScript SDK (Weeks 5-6)
**Goal**: Browser signal collection

- [ ] Canvas fingerprinting
- [ ] WebGL fingerprinting
- [ ] Audio fingerprinting
- [ ] Behavioral collectors (mouse, scroll)
- [ ] Transport layer (Beacon API)
- [ ] Bundle and optimize (< 30KB)

**Deliverables**:
- Working JavaScript SDK
- Signal collection functional
- Privacy safeguards implemented

---

### Phase 4: Analysis & UI (Weeks 7-8)
**Goal**: Analyst dashboard and queries

- [ ] React UI setup
- [ ] ClickHouse query interface
- [ ] Session visualization
- [ ] Fingerprint comparison
- [ ] Anomaly filtering
- [ ] Real-time event feed

**Deliverables**:
- Functional analyst dashboard
- Common queries optimized
- Visualization components

---

### Phase 5: Production Hardening (Weeks 9-10)
**Goal**: Production-ready deployment

- [ ] Load testing (10k req/sec)
- [ ] Security audit
- [ ] Performance optimization
- [ ] Monitoring and alerting
- [ ] Documentation
- [ ] Deployment automation

**Deliverables**:
- Performance benchmarks met
- Security audit passed
- Monitoring in place
- Deployment scripts

---

## TigerStyle Compliance Checklist

All code must adhere to TigerStyle principles:

### Safety
- [x] No `panic!` in production code
- [x] No `.unwrap()` or `.expect()` (except in tests)
- [x] No `unsafe` (or explicit justification)
- [x] All invariants validated at boundaries

### Simplicity
- [x] Explicit > implicit
- [x] Clear > clever
- [x] Minimal abstractions
- [x] Boring solutions preferred

### Correctness
- [x] Type-driven design
- [x] Invalid states unrepresentable
- [x] Comprehensive tests (> 90% coverage)
- [x] Deterministic behavior

### Performance
- [x] Fast by default
- [x] Benchmarked (not guessed)
- [x] Zero-copy where possible
- [x] Pre-allocation for known sizes

### Dependencies
- [x] Minimal, well-vetted crates
- [x] Pinned versions
- [x] Regular audits (`cargo audit`)
- [x] No unnecessary features

---

## Performance Targets

| Component | Metric | Target | Acceptable | Unacceptable |
|-----------|--------|--------|------------|--------------|
| **JS SDK** | Init time | < 10ms | < 20ms | > 20ms |
| **JS SDK** | Bundle size | < 20KB | < 30KB | > 30KB |
| **Ingestion** | Latency (p99) | < 5ms | < 10ms | > 10ms |
| **Ingestion** | Throughput | 10k/s | 5k/s | < 1k/s |
| **Enrichment** | Latency (p99) | < 50ms | < 100ms | > 100ms |
| **Redis** | Latency (p99) | < 1ms | < 2ms | > 2ms |
| **ClickHouse** | Write (p99) | < 100ms | < 500ms | > 500ms |
| **ClickHouse** | Query (p99) | < 100ms | < 500ms | > 500ms |

---

## Testing Requirements

### Unit Tests
- Each module > 90% coverage
- Fast (< 10s total)
- Deterministic (no flakes)
- Mock external services

### Integration Tests
- End-to-end flows
- Real Redis/ClickHouse (test containers)
- Cross-browser (Playwright)
- Performance benchmarks

### Load Tests
- 10k req/sec sustained
- Measure p50, p99, p999
- Memory usage under load
- Graceful degradation

---

## Security Requirements

### Pre-Deployment Checklist
- [ ] No PII collected
- [ ] Opt-out mechanisms tested
- [ ] TLS enforced (HTTPS only)
- [ ] Rate limiting functional
- [ ] Input validation comprehensive
- [ ] SQL injection tests passed
- [ ] XSS prevention verified
- [ ] Dependency audit clean (`cargo audit`)
- [ ] Security headers configured
- [ ] GDPR compliance verified

---

## Dependencies

### Rust Crates

```toml
[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

# Web framework
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }

# Storage
clickhouse = "0.11"
redis = { version = "0.24", features = ["tokio-comp"] }
deadpool-redis = "0.14"

# Cryptography
sha2 = "0.10"
blake3 = "1.5"
md5 = "0.7"

# GeoIP
maxminddb = "0.24"

# Telemetry
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Testing
proptest = "1.4"
criterion = "0.5"
```

### JavaScript Dependencies

```json
{
  "dependencies": {},
  "devDependencies": {
    "typescript": "^5.3.0",
    "esbuild": "^0.19.0",
    "@playwright/test": "^1.40.0"
  }
}
```

---

## Monitoring & Observability

### Metrics to Track

**Ingestion**:
- Requests per second
- Latency (p50, p95, p99)
- Error rate
- Payload size distribution

**Enrichment**:
- Enrichment time per stage
- GeoIP cache hit rate
- Fingerprint similarity matches
- Anomaly detection rate

**Storage**:
- ClickHouse write throughput
- Query latency
- Disk usage
- Compression ratio

**Redis**:
- Cache hit rate
- Memory usage
- Connection pool utilization
- Rate limit triggers

### Alerts

- Ingestion latency > 10ms (p99) → Page on-call
- Error rate > 1% → Alert Slack
- Disk usage > 80% → Alert ops
- Rate limit exceeded 10x → Investigate potential attack

---

## References

### External Documentation
- **TigerStyle**: https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md
- **JA3/JA4**: https://engineering.salesforce.com/tls-fingerprinting-with-ja3-and-ja3s-247362855ced/
- **Cloudflare Bot Management**: https://www.cloudflare.com/products/bot-management/
- **FingerprintJS**: https://github.com/fingerprintjs/fingerprintjs
- **ClickHouse**: https://clickhouse.com/docs/
- **Redis**: https://redis.io/documentation
- **Axum**: https://docs.rs/axum/latest/axum/
- **GDPR**: https://gdpr.eu/
- **OWASP Top 10**: https://owasp.org/www-project-top-ten/

### Internal Documents
- Vision: `../vision.md`
- Development Rules: `/home/ops/Project/mimicron/DEVELOPMENT_RULES.md`
- Security Rules: Memory (security.md)
- Testing Rules: Memory (testing.md)
- TigerStyle Rules: Memory (tigerstyle.md)

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-01-22 | Use TigerStyle principles | Safety, simplicity, correctness aligned with project needs |
| 2025-01-22 | Rust + Axum for ingestion | Performance, safety, excellent async ecosystem |
| 2025-01-22 | ClickHouse for analytics | Optimized for time-series, high-cardinality data |
| 2025-01-22 | Redis for session cache | < 1ms latency, simple key-value model |
| 2025-01-22 | MinHash for similarity | Fast approximate matching, scalable |
| 2025-01-22 | No PII collection | Privacy-first design, GDPR compliance |
| 2025-01-22 | 90-day retention | Balance between analysis needs and privacy |
| 2025-01-22 | Salted hashes rotated monthly | Prevent long-term tracking |

---

## Next Steps

1. ✅ **Complete RFCs** - All 7 RFCs written
2. ⏳ **Review & approval** - Team review of RFC designs
3. ⏳ **Set up workspace** - Initialize Rust workspace
4. ⏳ **Begin Phase 1** - Core infrastructure implementation
5. ⏳ **Iterate on design** - Refine based on implementation learnings

---

**Last Updated**: 2025-01-22  
**Maintainer**: Zuub Engineering  
**Status**: Ready for implementation review
