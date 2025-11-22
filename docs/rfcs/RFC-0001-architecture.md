# RFC-0001: Scrybe Core Architecture

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Style**: TigerStyle
- **Review**: Addressed critical blockers from multi-disciplinary review

## Summary

Scrybe is a high-fidelity browser behavior intelligence system designed to detect and understand automation with forensic granularity. Built in Rust with a JavaScript SDK, it captures multi-layer signals (network, browser, behavioral) and stores them in ClickHouse for analysis and ML-based bot detection.

## Motivation

Modern bot detection requires comprehensive signal collection across multiple layers: TLS fingerprints, HTTP headers, browser APIs, and human behavioral patterns. Scrybe acts as both a data collector and behavior profiler, enabling:

1. Real-time bot detection
2. Historical session analysis
3. ML model training on behavioral data
4. Forensic investigation of suspicious traffic

## Design Principles (TigerStyle)

1. **Safety**: No panics, no unwrap, no unsafe
2. **Simplicity**: Explicit over implicit, clear over clever
3. **Correctness**: Type-driven, schema validation, deterministic
4. **Performance**: < 5ms ingestion latency (p99), < 50ms enrichment
5. **Privacy**: No PII collection, salted hashes, opt-out support

## System Architecture

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────────┐
│   Browser       │─────▶│  Rust Ingestion  │─────▶│   Enrichment &      │
│   (JS SDK)      │ POST │  Gateway (Axum)  │      │   Fingerprinting    │
└─────────────────┘      └──────────────────┘      └─────────────────────┘
                                  │                           │
                                  │                           │
                                  ▼                           ▼
                         ┌─────────────────┐        ┌─────────────────┐
                         │  Session Cache  │        │   ClickHouse    │
                         │  (Redis)        │◀───────│   (Analytics)   │
                         └─────────────────┘        └─────────────────┘
                                  │
                                  ▼
                         ┌─────────────────┐
                         │   Analyst UI    │
                         │   (React)       │
                         └─────────────────┘
```

## Module Structure

```
scrybe/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── scrybe-core/                    # Core types and validation
│   │   ├── src/
│   │   │   ├── lib.rs                  # Public API
│   │   │   ├── error.rs                # Error types
│   │   │   ├── types/                  # Signal types
│   │   │   │   ├── network.rs          # TLS, IP, headers
│   │   │   │   ├── browser.rs          # Canvas, WebGL, fonts
│   │   │   │   ├── behavioral.rs       # Mouse, scroll, timing
│   │   │   │   └── session.rs          # Session metadata
│   │   │   └── validation/             # Input validation
│   │   └── tests/                      # Integration tests
│   │
│   ├── scrybe-ingestion/               # HTTP ingestion gateway
│   │   ├── src/
│   │   │   ├── main.rs                 # Axum server
│   │   │   ├── handlers.rs             # HTTP handlers
│   │   │   ├── middleware.rs           # Auth, rate limiting
│   │   │   └── validation.rs           # Request validation
│   │   └── tests/
│   │
│   ├── scrybe-enrichment/              # Fingerprinting & enrichment
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── fingerprint.rs          # Composite fingerprints
│   │   │   ├── geo.rs                  # IP → geo/ASN
│   │   │   ├── similarity.rs           # MinHash clustering
│   │   │   └── anomaly.rs              # Anomaly detection
│   │   └── tests/
│   │
│   ├── scrybe-storage/                 # ClickHouse interface
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs               # Table definitions
│   │   │   ├── writer.rs               # Batch writes
│   │   │   └── query.rs                # Query interface
│   │   └── migrations/
│   │
│   └── scrybe-cache/                   # Redis session cache
│       ├── src/
│       │   ├── lib.rs
│       │   ├── session.rs              # Session tracking
│       │   └── correlation.rs          # Fingerprint correlation
│       └── tests/
│
├── js-sdk/                             # Browser collection agent
│   ├── src/
│   │   ├── index.ts                    # Main entry
│   │   ├── collectors/
│   │   │   ├── network.ts              # Network signals
│   │   │   ├── browser.ts              # Browser APIs
│   │   │   └── behavioral.ts           # User behavior
│   │   ├── fingerprint.ts              # Client-side fingerprint
│   │   └── transport.ts                # Beacon API
│   ├── package.json
│   └── tsconfig.json
│
├── analyst-ui/                         # React dashboard
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   └── api/
│   └── package.json
│
└── scripts/
    ├── deploy.sh
    └── migrate-db.sh
```

## Data Flow

### 1. Browser Collection (JS SDK)
- Load on page visit
- Collect network/browser/behavioral signals
- Generate client-side fingerprint
- Send via Beacon API (non-blocking)

### 2. Ingestion Gateway (Rust/Axum)
- Receive POST /ingest
- Validate payload schema
- Extract server-side signals (IP, TLS, headers)
- Write to Redis (session cache)
- Enqueue for enrichment

### 3. Enrichment Pipeline (Rust)
- Resolve IP → geo/ASN
- Compute composite fingerprint
- Check fingerprint similarity (clustering)
- Detect anomalies (ML models)
- Write enriched data to ClickHouse

### 4. Storage (ClickHouse)
- Immutable session telemetry
- High-cardinality indexes
- Time-series optimization
- Analytics queries

### 5. Analysis (React UI)
- Query ClickHouse
- Visualize session replays
- Filter by anomaly type
- Compare fingerprints

## Core Types

```rust
/// Browser session with all collected signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub timestamp: DateTime<Utc>,
    pub network: NetworkSignals,
    pub browser: BrowserSignals,
    pub behavioral: BehavioralSignals,
    pub fingerprint: Fingerprint,
}

/// Network-layer signals (TLS, IP, headers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSignals {
    pub ip: IpAddr,
    pub ja3: Option<String>,           // TLS fingerprint
    pub ja4: Option<String>,
    pub headers: Vec<Header>,
    pub http_version: HttpVersion,
}

/// Browser environment signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSignals {
    pub canvas_hash: Option<String>,
    pub webgl_hash: Option<String>,
    pub audio_hash: Option<String>,
    pub fonts: Vec<String>,
    pub plugins: Vec<String>,
    pub timezone: String,
    pub language: String,
    pub screen: ScreenInfo,
}

/// User behavioral patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralSignals {
    pub mouse_events: Vec<MouseEvent>,
    pub scroll_events: Vec<ScrollEvent>,
    pub click_events: Vec<ClickEvent>,
    pub timing: TimingMetrics,
}

/// Composite fingerprint (deterministic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    pub hash: String,                  // SHA-256 of all signals
    pub components: FingerprintComponents,
    pub confidence: f64,               // 0.0-1.0
}
```

## Error Types (TigerStyle: Explicit)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ScrybeError {
    #[error("Invalid session data: {field} - {reason}")]
    InvalidSession { field: String, reason: String },
    
    #[error("Storage error: {0}")]
    Storage(#[from] clickhouse::error::Error),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Enrichment failed: {reason}")]
    Enrichment { reason: String },
    
    #[error("Rate limit exceeded: {limit} requests/min")]
    RateLimit { limit: u32 },
}
```

## API Endpoints

### Ingestion Gateway

```rust
// POST /api/v1/ingest
// Accepts browser session data
POST /api/v1/ingest
Content-Type: application/json

{
  "sessionId": "uuid",
  "timestamp": "2025-01-22T10:00:00Z",
  "network": { ... },
  "browser": { ... },
  "behavioral": { ... }
}

Response: 202 Accepted
```

### Query API

```rust
// GET /api/v1/sessions?filter=...
// Query sessions by various criteria
GET /api/v1/sessions?anomaly=true&limit=100

Response: 200 OK
{
  "sessions": [...],
  "total": 1523,
  "page": 1
}
```

## Health Checks

```rust
// GET /health - Liveness probe
async fn health_check() -> StatusCode {
    StatusCode::OK
}

// GET /health/ready - Readiness probe
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    // Check dependencies
    if !state.cache.is_healthy().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    if !state.storage.is_healthy().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(StatusCode::OK)
}
```

## Graceful Shutdown

```rust
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

// Server with graceful shutdown
axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

## Secrets Management

```rust
use std::env;

pub struct SecretConfig {
    pub clickhouse_url: Secret<String>,
    pub clickhouse_password: Secret<String>,
    pub redis_url: Secret<String>,
    pub api_key_salt: Secret<String>,
    pub tls_key_path: Secret<PathBuf>,
}

impl SecretConfig {
    pub fn from_env() -> Result<Self, ScrybeError> {
        Ok(Self {
            clickhouse_url: Secret::new(
                env::var("CLICKHOUSE_URL")
                    .map_err(|_| ScrybeError::ConfigError("Missing CLICKHOUSE_URL".into()))?
            ),
            clickhouse_password: Secret::new(
                env::var("CLICKHOUSE_PASSWORD")
                    .map_err(|_| ScrybeError::ConfigError("Missing CLICKHOUSE_PASSWORD".into()))?
            ),
            redis_url: Secret::new(
                env::var("REDIS_URL")
                    .map_err(|_| ScrybeError::ConfigError("Missing REDIS_URL".into()))?
            ),
            api_key_salt: Secret::new(
                env::var("API_KEY_SALT")
                    .map_err(|_| ScrybeError::ConfigError("Missing API_KEY_SALT".into()))?
            ),
            tls_key_path: Secret::new(
                PathBuf::from(env::var("TLS_KEY_PATH")
                    .map_err(|_| ScrybeError::ConfigError("Missing TLS_KEY_PATH".into()))?)
            ),
        })
    }
}

// Secret wrapper that doesn't print/log values
pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
```

## Cost Modeling

### Infrastructure Cost Estimates (10k req/sec sustained)

**ClickHouse** (r6i.4xlarge):
- Compute: $1,200/month
- Storage (20TB @ $0.08/GB): $1,600/month
- IOPS provisioned: $400/month
- **Subtotal**: $3,200/month

**Redis** (cache.r6g.4xlarge cluster, 4 nodes):
- Memory (256GB total): $2,400/month
- **Subtotal**: $2,400/month

**Data Transfer**:
- Ingress (4.3TB/day): Free
- Egress (100GB/day avg): $270/month
- **Subtotal**: $270/month

**Backups & Monitoring**:
- ClickHouse backups: $500/month
- Monitoring (Datadog): $200/month
- **Subtotal**: $700/month

**Total Monthly Cost**: ~$6,570/month

**Cost per Million Sessions**: $7.60

### Cost Optimization Strategies

1. **Reduce retention**: 90d → 30d = 66% storage savings
2. **Sampling**: 10% sample for ML = 90% reduction in training costs
3. **Reserved instances**: 40% compute savings if predictable
4. **Data tiering**: Hot (7d) → Warm (23d) → Archive (60d)
5. **Query limits**: Cap analyst query complexity

## Performance Requirements

| Operation | Target | Acceptable | Unacceptable |
|-----------|--------|------------|--------------|
| Ingestion (p99) | < 5ms | < 10ms | > 10ms |
| Enrichment (p99) | < 50ms | < 100ms | > 100ms |
| ClickHouse write | < 100ms | < 500ms | > 500ms |
| Redis read | < 1ms | < 5ms | > 5ms |
| JS SDK overhead | < 20ms | < 50ms | > 50ms |

## Security

### PII Protection
- **No input field values** collected
- **Salted hashes** for all identifiers
- **Rotate salts** every 30 days
- **Anonymize IPs** (mask last octet)

### Opt-Out
- Respect `DNT: 1` header
- Custom `X-Scrybe-Opt-Out` header
- JavaScript `window.scrybeOptOut = true`

### Rate Limiting
- 100 requests/min per IP
- 1000 requests/min per session
- Exponential backoff on violations

## Dependencies

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
subtle = "2.5"  # Constant-time comparisons

# Security
hmac = "0.12"
ring = "0.17"  # Cryptographic operations

# Circuit breaker
tower = { version = "0.4", features = ["limit", "timeout", "buffer"] }

# Bounded collections
arrayvec = "0.7"  # Stack-allocated vecs

# Telemetry
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Testing Strategy

### Unit Tests
- Each module > 90% coverage
- Property tests for fingerprinting
- Mock Redis/ClickHouse in tests

### Integration Tests
- End-to-end ingestion → storage
- Concurrent request handling
- Rate limiting behavior

### Load Tests
- 10,000 requests/sec sustained
- Measure p50, p99, p999 latencies
- Memory usage under load

## Success Criteria

1. ✅ Zero panics in production
2. ✅ All APIs documented
3. ✅ > 90% test coverage
4. ✅ < 5ms ingestion latency (p99)
5. ✅ Privacy compliance (no PII)
6. ✅ Zero unsafe code (except FFI)
7. ✅ Pre-commit hooks pass
8. ✅ Zero clippy warnings

## References

- RFC-0002: JavaScript SDK
- RFC-0003: Rust Ingestion Gateway
- RFC-0004: Fingerprinting & Enrichment
- RFC-0005: Storage Schema (ClickHouse)
- RFC-0006: Session Management (Redis)
- RFC-0007: Security & Privacy
- TigerStyle: https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md
- JA3/JA4: https://engineering.salesforce.com/tls-fingerprinting-with-ja3-and-ja3s-247362855ced/
- Cloudflare Bot Management: https://www.cloudflare.com/products/bot-management/
