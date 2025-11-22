# RFC-0003: Rust Ingestion Gateway

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Depends On**: RFC-0001 v0.2.0, RFC-0002 v0.2.0
- **Style**: TigerStyle
- **Review**: Added authentication, security headers, and backpressure handling

## Summary

The Scrybe Ingestion Gateway is a high-performance Rust HTTP service built with Axum that receives browser session data from the JavaScript SDK, validates it, enriches it with server-side signals (IP, TLS, headers), and forwards it to the enrichment pipeline and storage layer.

## Motivation

The ingestion gateway is the critical entry point for all browser telemetry. It must:

1. **Handle high throughput**: 10,000+ requests/sec
2. **Validate rigorously**: Reject malformed data early
3. **Extract server signals**: TLS fingerprints, IP metadata, HTTP headers
4. **Be resilient**: Graceful degradation, rate limiting
5. **Maintain low latency**: < 5ms response time (p99)

## Design Goals (TigerStyle)

1. **Safety**: No panics, explicit error handling, type-safe validation
2. **Performance**: Async I/O, zero-copy where possible, connection pooling
3. **Observability**: Structured logging, metrics, tracing
4. **Security**: Rate limiting, auth, input validation, PII protection
5. **Simplicity**: Clear request flow, minimal abstractions

## Architecture

```
┌──────────────┐
│   Browser    │
│   (JS SDK)   │
└──────┬───────┘
       │ POST /api/v1/ingest
       ▼
┌─────────────────────────────────────────┐
│      Axum HTTP Server (Rust)            │
├─────────────────────────────────────────┤
│  Middleware Stack                       │
│  ├─ Request ID                          │
│  ├─ Tracing                             │
│  ├─ CORS                                │
│  ├─ Rate Limiting                       │
│  └─ Authentication (optional)           │
├─────────────────────────────────────────┤
│  /api/v1/ingest Handler                 │
│  ├─ Parse & validate JSON               │
│  ├─ Extract server-side signals         │
│  │  ├─ IP address                       │
│  │  ├─ TLS fingerprint (JA3/JA4)        │
│  │  ├─ HTTP headers                     │
│  │  └─ Connection metadata              │
│  ├─ Combine client + server signals     │
│  └─ Forward to enrichment               │
├─────────────────────────────────────────┤
│  Output Channels                        │
│  ├─ Redis (session cache)               │
│  └─ Enrichment Queue (async)            │
└─────────────────────────────────────────┘
```

## HTTP API

### POST /api/v1/ingest

Accepts browser session data from the JavaScript SDK.

**Request**:
```http
POST /api/v1/ingest HTTP/1.1
Host: scrybe.example.com
Content-Type: application/json
User-Agent: Mozilla/5.0 ...
X-Scrybe-SDK-Version: 1.0.0
X-Scrybe-Signature: sha256=abc123...  # HMAC signature

{
  "sessionId": "123e4567-e89b-12d3-a456-426614174000",
  "timestamp": "2025-01-22T10:30:00.000Z",
  "network": { ... },
  "browser": { ... },
  "behavioral": { ... }
}
```

**Response** (Success):
```http
HTTP/1.1 202 Accepted
X-Request-ID: req_abc123
Content-Type: application/json

{
  "status": "accepted",
  "sessionId": "123e4567-e89b-12d3-a456-426614174000"
}
```

**Response** (Validation Error):
```http
HTTP/1.1 400 Bad Request
X-Request-ID: req_abc123
Content-Type: application/json

{
  "error": "validation_error",
  "field": "timestamp",
  "message": "Invalid timestamp format"
}
```

**Response** (Rate Limited):
```http
HTTP/1.1 429 Too Many Requests
X-Request-ID: req_abc123
Retry-After: 60

{
  "error": "rate_limit_exceeded",
  "limit": 100,
  "window": "60s"
}
```

## Request Processing Flow

```rust
async fn ingest_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, ScrybeError> {
    // 1. Verify API signature (prevent unauthorized submissions)
    verify_signature(&headers, &payload, &app_state.api_key)?;
    
    // 2. Validate payload schema
    payload.validate()?;
    
    // 3. Check nonce (prevent replay attacks)
    if app_state.cache.has_nonce(&payload.nonce).await? {
        return Err(ScrybeError::ReplayAttack);
    }
    app_state.cache.store_nonce(&payload.nonce, Duration::from_secs(300)).await?;
    
    // 4. Extract server-side signals
    let server_signals = extract_server_signals(&headers, &addr).await?;
    
    // 5. Combine client + server signals
    let session = Session {
        id: payload.session_id,
        timestamp: payload.timestamp,
        client_signals: payload,
        server_signals,
    };
    
    // 6. Check backpressure (prevent queue overflow)
    if app_state.enrichment_queue.len() > app_state.config.max_queue_size {
        return Err(ScrybeError::ServiceOverloaded);
    }
    
    // 7. Write to Redis cache (non-blocking)
    app_state.cache.store_session(&session).await?;
    
    // 8. Enqueue for enrichment (async)
    app_state.enrichment_queue.push(session).await?;
    
    // 9. Return success
    Ok(Json(IngestResponse {
        status: "accepted".to_string(),
        session_id: session.id,
    }))
}
```

## API Authentication

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn verify_signature(
    headers: &HeaderMap,
    payload: &IngestRequest,
    api_key: &Secret<String>,
) -> Result<(), ScrybeError> {
    // Get signature from headers
    let sig_header = headers
        .get("x-scrybe-signature")
        .ok_or(ScrybeError::MissingSignature)?
        .to_str()
        .map_err(|_| ScrybeError::InvalidSignature)?;
    
    if !sig_header.starts_with("sha256=") {
        return Err(ScrybeError::InvalidSignature);
    }
    
    let provided_sig = &sig_header[7..]; // Remove "sha256=" prefix
    
    // Compute expected signature
    let mut mac = HmacSha256::new_from_slice(api_key.expose().as_bytes())
        .map_err(|_| ScrybeError::InvalidSignature)?;
    
    let payload_bytes = serde_json::to_vec(payload)
        .map_err(|_| ScrybeError::InvalidSignature)?;
    
    mac.update(&payload_bytes);
    
    let expected_sig = hex::encode(mac.finalize().into_bytes());
    
    // Constant-time comparison (prevent timing attacks)
    use subtle::ConstantTimeEq;
    if provided_sig.as_bytes().ct_eq(expected_sig.as_bytes()).into() {
        Ok(())
    } else {
        Err(ScrybeError::InvalidSignature)
    }
}
```

## Core Types

### Ingest Request (from JS SDK)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,              // For replay prevention
    pub network: NetworkSignals,
    pub browser: BrowserSignals,
    pub behavioral: BehavioralSignals,
}

impl IngestRequest {
    /// Validate all fields.
    pub fn validate(&self) -> Result<(), ScrybeError> {
        // Check timestamp not in future
        if self.timestamp > Utc::now() {
            return Err(ScrybeError::InvalidSession {
                field: "timestamp".to_string(),
                reason: "Timestamp cannot be in the future".to_string(),
            });
        }
        
        // Check timestamp not too old (> 5 minutes for anti-replay)
        let age = Utc::now() - self.timestamp;
        if age.num_minutes() > 5 {
            return Err(ScrybeError::InvalidSession {
                field: "timestamp".to_string(),
                reason: "Timestamp too old (> 5 minutes)".to_string(),
            });
        }
        
        // Validate nonce format (UUID)
        if Uuid::parse_str(&self.nonce).is_err() {
            return Err(ScrybeError::InvalidSession {
                field: "nonce".to_string(),
                reason: "Invalid nonce format (must be UUID)".to_string(),
            });
        }
        
        // Validate network signals
        self.network.validate()?;
        
        // Validate browser signals
        self.browser.validate()?;
        
        // Validate behavioral signals
        self.behavioral.validate()?;
        
        Ok(())
    }
}
```

### Server-Side Signals

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSignals {
    pub ip: IpAddr,
    pub tls: Option<TlsFingerprint>,
    pub headers: Vec<HttpHeader>,
    pub connection: ConnectionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprint {
    pub ja3: String,                // TLS client fingerprint
    pub ja3_hash: String,           // MD5 of JA3
    pub ja4: Option<String>,        // Newer TLS fingerprint
    pub cipher_suite: String,
    pub tls_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,              // Potentially redacted
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub protocol: String,           // "HTTP/1.1", "HTTP/2", "HTTP/3"
    pub remote_addr: SocketAddr,
    pub received_at: DateTime<Utc>,
}
```

### Session (Combined)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub client_signals: IngestRequest,
    pub server_signals: ServerSignals,
}
```

## Server-Side Signal Extraction

### IP Address

```rust
fn extract_ip(headers: &HeaderMap, addr: &SocketAddr) -> IpAddr {
    // Check X-Forwarded-For (if behind proxy)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // Take first IP (client)
            if let Some(first_ip) = xff_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return ip;
                }
            }
        }
    }
    
    // Fallback to direct connection IP
    addr.ip()
}
```

### TLS Fingerprint (JA3/JA4)

```rust
async fn extract_tls_fingerprint(
    connection: &Connection,
) -> Result<Option<TlsFingerprint>, ScrybeError> {
    // Get TLS info from connection
    let tls_info = connection.tls_info()?;
    
    if let Some(tls) = tls_info {
        // Build JA3 string
        let ja3 = format!(
            "{},{},{},{},{}",
            tls.version,
            tls.cipher_suites.join("-"),
            tls.extensions.join("-"),
            tls.elliptic_curves.join("-"),
            tls.elliptic_curve_formats.join("-")
        );
        
        // Hash it (MD5 for JA3 standard)
        let ja3_hash = format!("{:x}", md5::compute(&ja3));
        
        Ok(Some(TlsFingerprint {
            ja3,
            ja3_hash,
            ja4: None,  // TODO: Implement JA4
            cipher_suite: tls.cipher_suites.first().cloned().unwrap_or_default(),
            tls_version: tls.version.clone(),
        }))
    } else {
        Ok(None)
    }
}
```

### HTTP Headers

```rust
fn extract_headers(headers: &HeaderMap) -> Vec<HttpHeader> {
    // Capture important headers (preserve order)
    const IMPORTANT_HEADERS: &[&str] = &[
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        "referer",
        "sec-ch-ua",
        "sec-ch-ua-mobile",
        "sec-ch-ua-platform",
        "sec-fetch-dest",
        "sec-fetch-mode",
        "sec-fetch-site",
    ];
    
    let mut result = Vec::new();
    
    for &name in IMPORTANT_HEADERS {
        if let Some(value) = headers.get(name) {
            if let Ok(value_str) = value.to_str() {
                result.push(HttpHeader {
                    name: name.to_string(),
                    value: value_str.to_string(),
                });
            }
        }
    }
    
    result
}
```

## Middleware

### Rate Limiting

```rust
use tower_governor::{GovernorLayer, GovernorConfigBuilder};

pub fn rate_limiter() -> GovernorLayer<'static> {
    let config = Box::new(
        GovernorConfigBuilder::default()
            .per_second(100)              // 100 requests/sec per IP
            .burst_size(20)               // Allow bursts up to 20
            .finish()
            .unwrap()
    );
    
    GovernorLayer {
        config: Box::leak(config),
    }
}
```

### Request ID

```rust
use axum::middleware::Next;
use uuid::Uuid;

pub async fn request_id_middleware(
    mut req: Request<Body>,
    next: Next<Body>,
) -> Response {
    // Generate request ID
    let request_id = Uuid::new_v4().to_string();
    
    // Add to request extensions
    req.extensions_mut().insert(RequestId(request_id.clone()));
    
    // Process request
    let mut response = next.run(req).await;
    
    // Add to response headers
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    response
}
```

### Tracing

```rust
use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn tracing_layer() -> TraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(|request: &Request<Body>, _span: &Span| {
            tracing::info!("started processing request");
        })
        .on_response(|response: &Response, latency: Duration, _span: &Span| {
            tracing::info!(
                status = response.status().as_u16(),
                latency_ms = latency.as_millis(),
                "finished processing request"
            );
        })
}
```

### Security Headers

```rust
use axum::middleware::Next;
use axum::http::{Request, Response, HeaderValue};

async fn security_headers_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let mut response = next.run(req).await;
    
    let headers = response.headers_mut();
    
    // Prevent MIME sniffing
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    
    // Prevent clickjacking
    headers.insert(
        "x-frame-options",
        HeaderValue::from_static("DENY"),
    );
    
    // XSS protection (legacy but harmless)
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );
    
    // HSTS (force HTTPS)
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    
    // Content Security Policy
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    
    response
}
```

### CORS

```rust
use tower_http::cors::{CorsLayer, Any};

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)                      // Or specific origins (production: whitelist)
        .allow_methods([Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("x-scrybe-sdk-version"),
        ])
        .max_age(Duration::from_secs(3600))
}
```

## Application State

```rust
#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<RedisCache>,
    pub enrichment_queue: Arc<BoundedEnrichmentQueue>,
    pub metrics: Arc<Metrics>,
    pub api_key: Secret<String>,
    pub config: Config,
}

/// Bounded queue with backpressure
pub struct BoundedEnrichmentQueue {
    tx: mpsc::Sender<Session>,
    rx: Arc<Mutex<mpsc::Receiver<Session>>>,
}

impl BoundedEnrichmentQueue {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }
    
    pub async fn push(&self, session: Session) -> Result<(), ScrybeError> {
        self.tx.send(session).await
            .map_err(|_| ScrybeError::QueueFull)
    }
    
    pub fn len(&self) -> usize {
        // Approximation (channel doesn't expose exact len)
        self.tx.capacity() - self.tx.max_capacity()
    }
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, ScrybeError> {
        // Load secrets
        let secret_config = SecretConfig::from_env()?;
        
        // Initialize Redis cache
        let cache = Arc::new(RedisCache::connect(secret_config.redis_url.expose()).await?);
        
        // Initialize bounded enrichment queue
        let enrichment_queue = Arc::new(
            BoundedEnrichmentQueue::new(config.max_queue_size)
        );
        
        // Initialize metrics
        let metrics = Arc::new(Metrics::new());
        
        Ok(Self {
            cache,
            enrichment_queue,
            metrics,
            api_key: secret_config.api_key_salt,
            config,
        })
    }
}
```

## Server Initialization

```rust
use axum::{Router, routing::post};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), ScrybeError> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    // Load configuration
    let config = Config::load()?;
    
    // Initialize app state
    let state = AppState::new(config.clone()).await?;
    
    // Build router
    let app = Router::new()
        .route("/api/v1/ingest", post(ingest_handler))
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(tracing_layer())
        .layer(cors_layer())
        .layer(rate_limiter())
        .layer(axum::middleware::from_fn(request_id_middleware))
        .with_state(state);
    
    // Bind address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    
    tracing::info!("Starting Scrybe Ingestion Gateway on {}", addr);
    
    // Start server with graceful shutdown
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ScrybeError::ServerError(e.to_string()))?;
    
    Ok(())
}
```

## Error Handling

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

impl IntoResponse for ScrybeError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ScrybeError::InvalidSession { field, reason } => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "validation_error",
                    "field": field,
                    "message": reason,
                }),
            ),
            
            ScrybeError::RateLimit { limit } => (
                StatusCode::TOO_MANY_REQUESTS,
                json!({
                    "error": "rate_limit_exceeded",
                    "limit": limit,
                    "window": "60s",
                }),
            ),
            
            ScrybeError::Storage(_) | ScrybeError::Cache(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": "internal_error",
                    "message": "Please try again later",
                }),
            ),
            
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": "internal_error",
                }),
            ),
        };
        
        (status, Json(error_message)).into_response()
    }
}
```

## Performance Optimizations

### Connection Pooling

```rust
// Redis connection pool
let redis_client = redis::Client::open(config.redis_url)?;
let redis_pool = deadpool_redis::Pool::builder(redis_client)
    .max_size(100)
    .build()?;
```

### Async I/O

```rust
// All I/O operations are async
async fn ingest_handler(...) -> Result<...> {
    // Parallel async operations
    let (cache_result, queue_result) = tokio::join!(
        app_state.cache.store_session(&session),
        app_state.enrichment_queue.push(session),
    );
    
    cache_result?;
    queue_result?;
    
    Ok(...)
}
```

### Zero-Copy Where Possible

```rust
// Use references to avoid cloning
fn extract_headers(headers: &HeaderMap) -> Vec<HttpHeader> {
    // Borrow, don't clone
}
```

## Metrics & Observability

```rust
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub validation_errors: Counter,
    pub rate_limit_hits: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests_total: Counter::new("scrybe_requests_total", "Total requests").unwrap(),
            request_duration: Histogram::new("scrybe_request_duration_ms", "Request duration").unwrap(),
            validation_errors: Counter::new("scrybe_validation_errors", "Validation errors").unwrap(),
            rate_limit_hits: Counter::new("scrybe_rate_limit_hits", "Rate limit hits").unwrap(),
        }
    }
    
    pub fn record_request(&self, duration: Duration) {
        self.requests_total.inc();
        self.request_duration.observe(duration.as_millis() as f64);
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_timestamp_future() {
        let mut req = IngestRequest::default();
        req.timestamp = Utc::now() + Duration::hours(1);
        
        assert!(req.validate().is_err());
    }
    
    #[test]
    fn test_validate_timestamp_too_old() {
        let mut req = IngestRequest::default();
        req.timestamp = Utc::now() - Duration::hours(25);
        
        assert!(req.validate().is_err());
    }
    
    #[test]
    fn test_extract_ip_from_xff() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        
        let ip = extract_ip(&headers, &"10.0.0.1:1234".parse().unwrap());
        assert_eq!(ip, "1.2.3.4".parse().unwrap());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_ingest_endpoint_success() {
    let app = create_test_app().await;
    
    let session = create_test_session();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ingest")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&session).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}
```

## Performance Requirements

| Metric | Target | Acceptable | Unacceptable |
|--------|--------|------------|--------------|
| Latency (p50) | < 2ms | < 5ms | > 5ms |
| Latency (p99) | < 5ms | < 10ms | > 10ms |
| Throughput | 10k req/s | 5k req/s | < 1k req/s |
| Memory per request | < 10KB | < 50KB | > 50KB |
| CPU per request | < 100μs | < 500μs | > 500μs |

## Security Checklist

- ✅ Input validation on all fields
- ✅ Rate limiting per IP
- ✅ CORS properly configured
- ✅ No PII in logs
- ✅ TLS required (HTTPS only)
- ✅ Request size limits (< 1MB)
- ✅ Timeout on slow requests (5s)

## Success Criteria

1. ✅ Zero panics in production
2. ✅ < 5ms latency (p99)
3. ✅ 10k req/s throughput
4. ✅ All errors return appropriate status codes
5. ✅ Structured logging and metrics
6. ✅ Graceful shutdown
7. ✅ > 90% test coverage

## References

- RFC-0001: Core Architecture
- RFC-0002: JavaScript SDK
- RFC-0004: Fingerprinting & Enrichment
- RFC-0006: Session Management (Redis)
- Axum Documentation: https://docs.rs/axum/latest/axum/
- Tower HTTP: https://docs.rs/tower-http/latest/tower_http/
- JA3 Fingerprinting: https://engineering.salesforce.com/tls-fingerprinting-with-ja3-and-ja3s-247362855ced/
