# RFC-0004: Fingerprinting & Enrichment Pipeline

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Depends On**: RFC-0001 v0.2.0, RFC-0003 v0.2.0
- **Style**: TigerStyle
- **Review**: Added graceful degradation, circuit breaker, model versioning

## Summary

The Fingerprinting & Enrichment Pipeline is a Rust service that takes raw session data from the ingestion gateway and enriches it with:
1. **Composite fingerprints** (deterministic device identification)
2. **Geo/ASN resolution** (IP → location/network)
3. **Fingerprint similarity** (clustering via MinHash)
4. **Anomaly detection** (ML-based bot scoring)

This enriched data is then stored in ClickHouse for analysis.

## Motivation

Raw browser signals alone aren't enough for effective bot detection. We need:

1. **Composite fingerprinting**: Combine all signals into a stable device identifier
2. **Contextual enrichment**: Add geographic and network context
3. **Pattern recognition**: Identify similar fingerprints (bot farms)
4. **Anomaly scoring**: Flag suspicious behavioral patterns

The enrichment pipeline must be:
- **Fast**: < 50ms enrichment time (p99)
- **Deterministic**: Same input → same fingerprint
- **Resilient**: Graceful degradation when services fail
- **Observable**: Clear metrics and logging

## Design Goals (TigerStyle)

1. **Safety**: No panics, explicit error handling
2. **Performance**: Async processing, batching, caching
3. **Correctness**: Deterministic fingerprints, validated outputs
4. **Simplicity**: Clear pipeline stages, minimal dependencies
5. **Privacy**: Salted hashes, no PII

## Enrichment Pipeline Architecture

```
┌─────────────────┐
│  Ingestion      │
│  Gateway        │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────────┐
│         Enrichment Pipeline                 │
├─────────────────────────────────────────────┤
│  Stage 1: Fingerprint Generation            │
│  ├─ Combine all signals                     │
│  ├─ Compute composite hash (SHA-256)        │
│  ├─ Generate component hashes               │
│  └─ Calculate confidence score              │
├─────────────────────────────────────────────┤
│  Stage 2: Geo/ASN Resolution                │
│  ├─ Query MaxMind GeoIP2 (local DB)         │
│  ├─ Resolve ASN/ISP                         │
│  ├─ Check VPN/proxy databases               │
│  └─ Add timezone validation                 │
├─────────────────────────────────────────────┤
│  Stage 3: Fingerprint Similarity            │
│  ├─ Generate MinHash signature              │
│  ├─ Query existing fingerprints (LSH)       │
│  ├─ Find similar fingerprints (Jaccard)     │
│  └─ Cluster assignment                      │
├─────────────────────────────────────────────┤
│  Stage 4: Anomaly Detection                 │
│  ├─ Behavioral anomaly scoring              │
│  ├─ Timing anomaly detection                │
│  ├─ Header consistency checks               │
│  └─ ML model inference (optional)           │
├─────────────────────────────────────────────┤
│  Stage 5: Enriched Session Assembly         │
│  ├─ Combine all enrichment data             │
│  ├─ Validate schema                         │
│  └─ Prepare for storage                     │
└─────────────────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│   ClickHouse    │
│   Storage       │
└─────────────────┘
```

## Core Types

### Enriched Session

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedSession {
    // Original session data
    pub session: Session,
    
    // Fingerprint enrichment
    pub fingerprint: CompositeFingerprint,
    
    // Geo/network enrichment
    pub geo: GeoEnrichment,
    
    // Similarity enrichment
    pub similarity: SimilarityEnrichment,
    
    // Anomaly enrichment
    pub anomaly: AnomalyScore,
    
    // Metadata
    pub enriched_at: DateTime<Utc>,
    pub enrichment_version: String,
    pub model_version: String,      // ML model version for reproducibility
}
```

### Composite Fingerprint

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeFingerprint {
    /// SHA-256 hash of all components
    pub hash: String,
    
    /// Individual component hashes
    pub components: FingerprintComponents,
    
    /// Stability confidence (0.0-1.0)
    pub confidence: f64,
    
    /// MinHash signature for similarity
    pub minhash: MinHashSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintComponents {
    pub network_hash: String,       // TLS + headers
    pub browser_hash: String,       // Canvas + WebGL + fonts
    pub behavioral_hash: String,    // Mouse + scroll patterns
    pub device_hash: String,        // Screen + navigator
}

impl CompositeFingerprint {
    /// Generate deterministic fingerprint from session.
    pub fn from_session(session: &Session) -> Result<Self, ScrybeError> {
        // Compute component hashes
        let network_hash = Self::hash_network(&session.server_signals.tls)?;
        let browser_hash = Self::hash_browser(&session.client_signals.browser)?;
        let behavioral_hash = Self::hash_behavioral(&session.client_signals.behavioral)?;
        let device_hash = Self::hash_device(&session.client_signals.browser.navigator)?;
        
        let components = FingerprintComponents {
            network_hash: network_hash.clone(),
            browser_hash: browser_hash.clone(),
            behavioral_hash: behavioral_hash.clone(),
            device_hash: device_hash.clone(),
        };
        
        // Combine into composite hash
        let composite = format!(
            "{}:{}:{}:{}",
            network_hash, browser_hash, behavioral_hash, device_hash
        );
        let hash = format!("{:x}", sha2::Sha256::digest(composite.as_bytes()));
        
        // Calculate confidence
        let confidence = Self::calculate_confidence(&components);
        
        // Generate MinHash for similarity
        let minhash = MinHashSignature::from_components(&components)?;
        
        Ok(Self {
            hash,
            components,
            confidence,
            minhash,
        })
    }
    
    fn hash_network(tls: &Option<TlsFingerprint>) -> Result<String, ScrybeError> {
        if let Some(tls) = tls {
            Ok(tls.ja3_hash.clone())
        } else {
            Ok("no_tls".to_string())
        }
    }
    
    fn hash_browser(browser: &BrowserSignals) -> Result<String, ScrybeError> {
        // Combine canvas + webgl + audio + fonts
        let combined = format!(
            "{}:{}:{}:{}",
            browser.canvas.as_ref().map(|c| c.hash.as_str()).unwrap_or(""),
            browser.webgl.as_ref().map(|w| w.hash.as_str()).unwrap_or(""),
            browser.audio.as_ref().map(|a| a.hash.as_str()).unwrap_or(""),
            browser.fonts.as_ref().map(|f| f.hash.as_str()).unwrap_or("")
        );
        
        Ok(format!("{:x}", sha2::Sha256::digest(combined.as_bytes())))
    }
    
    fn hash_behavioral(behavioral: &BehavioralSignals) -> Result<String, ScrybeError> {
        // Hash behavioral patterns (mouse entropy, scroll smoothness, etc.)
        let pattern = format!(
            "entropy:{:.2}:velocity_avg:{:.2}",
            behavioral.mouse.as_ref().map(|m| m.entropy).unwrap_or(0.0),
            behavioral.mouse.as_ref()
                .and_then(|m| m.velocity.first())
                .copied()
                .unwrap_or(0.0)
        );
        
        Ok(format!("{:x}", sha2::Sha256::digest(pattern.as_bytes())))
    }
    
    fn hash_device(navigator: &NavigatorInfo) -> Result<String, ScrybeError> {
        let device = format!(
            "{}:{}:{}:{}",
            navigator.platform,
            navigator.hardware_concurrency,
            navigator.device_memory.unwrap_or(0),
            navigator.max_touch_points
        );
        
        Ok(format!("{:x}", sha2::Sha256::digest(device.as_bytes())))
    }
    
    fn calculate_confidence(components: &FingerprintComponents) -> f64 {
        // Higher confidence when more components are available
        let mut score = 0.0;
        
        if !components.network_hash.is_empty() { score += 0.25; }
        if !components.browser_hash.is_empty() { score += 0.35; }
        if !components.behavioral_hash.is_empty() { score += 0.20; }
        if !components.device_hash.is_empty() { score += 0.20; }
        
        score
    }
}
```

### Geo Enrichment

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoEnrichment {
    pub country: String,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: String,
    pub asn: u32,
    pub asn_org: String,
    pub is_vpn: bool,
    pub is_proxy: bool,
    pub is_tor: bool,
    pub is_hosting: bool,
}

impl GeoEnrichment {
    pub async fn from_ip(ip: IpAddr, geoip: &GeoIpDatabase) -> Result<Self, ScrybeError> {
        // Query MaxMind GeoIP2 database
        let geo = geoip.lookup(ip)?;
        
        // Check VPN/proxy databases
        let is_vpn = geoip.is_vpn(ip)?;
        let is_proxy = geoip.is_proxy(ip)?;
        let is_tor = geoip.is_tor(ip)?;
        let is_hosting = geoip.is_hosting(ip)?;
        
        Ok(Self {
            country: geo.country,
            region: geo.region,
            city: geo.city,
            latitude: geo.latitude,
            longitude: geo.longitude,
            timezone: geo.timezone,
            asn: geo.asn,
            asn_org: geo.asn_org,
            is_vpn,
            is_proxy,
            is_tor,
            is_hosting,
        })
    }
}
```

### Similarity Enrichment

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityEnrichment {
    /// Similar fingerprints (within threshold)
    pub similar_fingerprints: Vec<SimilarFingerprint>,
    
    /// Cluster assignment (if any)
    pub cluster_id: Option<String>,
    
    /// Similarity score to cluster centroid
    pub cluster_similarity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFingerprint {
    pub fingerprint_hash: String,
    pub jaccard_similarity: f64,    // 0.0-1.0
    pub session_count: u64,          // How many sessions with this fingerprint
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl SimilarityEnrichment {
    pub async fn compute(
        fingerprint: &CompositeFingerprint,
        similarity_index: &SimilarityIndex,
    ) -> Result<Self, ScrybeError> {
        // Find similar fingerprints using LSH
        let similar = similarity_index
            .find_similar(&fingerprint.minhash, 0.7)  // 70% similarity threshold
            .await?;
        
        // Find cluster assignment
        let cluster = similarity_index
            .find_cluster(&fingerprint.hash)
            .await?;
        
        Ok(Self {
            similar_fingerprints: similar,
            cluster_id: cluster.as_ref().map(|c| c.id.clone()),
            cluster_similarity: cluster.map(|c| c.similarity),
        })
    }
}
```

### Anomaly Score

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScore {
    /// Overall bot probability (0.0-1.0)
    pub bot_probability: f64,
    
    /// Individual anomaly scores
    pub behavioral_anomaly: f64,
    pub timing_anomaly: f64,
    pub header_anomaly: f64,
    pub fingerprint_anomaly: f64,
    
    /// Detected anomalies
    pub anomalies: Vec<DetectedAnomaly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAnomaly {
    pub anomaly_type: String,
    pub severity: String,          // "low", "medium", "high"
    pub description: String,
    pub evidence: serde_json::Value,
}

impl AnomalyScore {
    pub fn detect(
        session: &Session,
        fingerprint: &CompositeFingerprint,
        geo: &GeoEnrichment,
    ) -> Result<Self, ScrybeError> {
        let mut anomalies = Vec::new();
        
        // 1. Behavioral anomalies
        let behavioral_anomaly = Self::check_behavioral(session, &mut anomalies)?;
        
        // 2. Timing anomalies
        let timing_anomaly = Self::check_timing(session, &mut anomalies)?;
        
        // 3. Header consistency
        let header_anomaly = Self::check_headers(session, &mut anomalies)?;
        
        // 4. Fingerprint anomalies
        let fingerprint_anomaly = Self::check_fingerprint(fingerprint, &mut anomalies)?;
        
        // Compute overall bot probability
        let bot_probability = Self::compute_bot_probability(
            behavioral_anomaly,
            timing_anomaly,
            header_anomaly,
            fingerprint_anomaly,
        );
        
        Ok(Self {
            bot_probability,
            behavioral_anomaly,
            timing_anomaly,
            header_anomaly,
            fingerprint_anomaly,
            anomalies,
        })
    }
    
    fn check_behavioral(
        session: &Session,
        anomalies: &mut Vec<DetectedAnomaly>,
        thresholds: &AnomalyThresholds,
    ) -> Result<f64, ScrybeError> {
        let behavioral = &session.client_signals.behavioral;
        let mut score = 0.0;
        
        // Check mouse entropy (percentile-based threshold, not static)
        if let Some(mouse) = &behavioral.mouse {
            if mouse.entropy < thresholds.mouse_entropy_p5 {
                score += 0.4;
                anomalies.push(DetectedAnomaly {
                    anomaly_type: "low_mouse_entropy".to_string(),
                    severity: "high".to_string(),
                    description: format!("Mouse entropy {} below P5 threshold {}", 
                        mouse.entropy, thresholds.mouse_entropy_p5),
                    evidence: json!({ 
                        "entropy": mouse.entropy,
                        "threshold": thresholds.mouse_entropy_p5,
                        "percentile": 5
                    }),
                });
            }
        } else {
            // No mouse events = suspicious
            score += 0.3;
            anomalies.push(DetectedAnomaly {
                anomaly_type: "no_mouse_events".to_string(),
                severity: "medium".to_string(),
                description: "No mouse events recorded".to_string(),
                evidence: json!({}),
            });
        }
        
        // Check scroll smoothness (percentile-based)
        if let Some(scroll) = &behavioral.scroll {
            if scroll.smoothness > thresholds.scroll_smoothness_p95 {
                score += 0.3;
                anomalies.push(DetectedAnomaly {
                    anomaly_type: "perfect_scroll".to_string(),
                    severity: "medium".to_string(),
                    description: "Scroll pattern is unnaturally smooth".to_string(),
                    evidence: json!({ "smoothness": scroll.smoothness }),
                });
            }
        }
        
        Ok(score.min(1.0))
    }
    
    fn check_timing(
        session: &Session,
        anomalies: &mut Vec<DetectedAnomaly>,
    ) -> Result<f64, ScrybeError> {
        let timing = &session.client_signals.behavioral.timing;
        let mut score = 0.0;
        
        // Check time on page (too short = bot)
        if timing.time_on_page < 500 {  // < 500ms
            score += 0.4;
            anomalies.push(DetectedAnomaly {
                anomaly_type: "instant_interaction".to_string(),
                severity: "high".to_string(),
                description: "Session started and interacted too quickly".to_string(),
                evidence: json!({ "time_on_page_ms": timing.time_on_page }),
            });
        }
        
        // Check focus changes (rapid = bot)
        if timing.focus_changes > 10 && timing.time_on_page < 5000 {
            score += 0.3;
            anomalies.push(DetectedAnomaly {
                anomaly_type: "rapid_focus_changes".to_string(),
                severity: "medium".to_string(),
                description: "Too many focus changes in short time".to_string(),
                evidence: json!({
                    "focus_changes": timing.focus_changes,
                    "time_on_page_ms": timing.time_on_page,
                }),
            });
        }
        
        Ok(score.min(1.0))
    }
    
    fn check_headers(
        session: &Session,
        anomalies: &mut Vec<DetectedAnomaly>,
    ) -> Result<f64, ScrybeError> {
        let headers = &session.server_signals.headers;
        let mut score = 0.0;
        
        // Check for automation indicators
        for header in headers {
            if header.name.to_lowercase() == "user-agent" {
                let ua = &header.value;
                
                // Check for headless Chrome
                if ua.contains("HeadlessChrome") {
                    score += 0.9;
                    anomalies.push(DetectedAnomaly {
                        anomaly_type: "headless_chrome".to_string(),
                        severity: "high".to_string(),
                        description: "Headless Chrome detected in User-Agent".to_string(),
                        evidence: json!({ "user_agent": ua }),
                    });
                }
                
                // Check for Selenium
                if ua.contains("Selenium") {
                    score += 0.9;
                    anomalies.push(DetectedAnomaly {
                        anomaly_type: "selenium_detected".to_string(),
                        severity: "high".to_string(),
                        description: "Selenium detected in User-Agent".to_string(),
                        evidence: json!({ "user_agent": ua }),
                    });
                }
            }
        }
        
        // Check for webdriver flag
        if let Some(browser) = &session.client_signals.browser.quirks {
            if browser.webdriver {
                score += 0.8;
                anomalies.push(DetectedAnomaly {
                    anomaly_type: "webdriver_flag".to_string(),
                    severity: "high".to_string(),
                    description: "navigator.webdriver is true".to_string(),
                    evidence: json!({}),
                });
            }
        }
        
        Ok(score.min(1.0))
    }
    
    fn check_fingerprint(
        fingerprint: &CompositeFingerprint,
        anomalies: &mut Vec<DetectedAnomaly>,
    ) -> Result<f64, ScrybeError> {
        let mut score = 0.0;
        
        // Low confidence fingerprint = suspicious
        if fingerprint.confidence < 0.5 {
            score += 0.3;
            anomalies.push(DetectedAnomaly {
                anomaly_type: "low_fingerprint_confidence".to_string(),
                severity: "medium".to_string(),
                description: "Fingerprint has low confidence score".to_string(),
                evidence: json!({ "confidence": fingerprint.confidence }),
            });
        }
        
        Ok(score)
    }
    
    fn compute_bot_probability(
        behavioral: f64,
        timing: f64,
        header: f64,
        fingerprint: f64,
    ) -> f64 {
        // Weighted average (headers most important)
        let weighted_sum = 
            behavioral * 0.25 +
            timing * 0.25 +
            header * 0.40 +
            fingerprint * 0.10;
        
        weighted_sum.min(1.0)
    }
}
```

## MinHash Implementation (Similarity Detection)

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinHashSignature {
    pub hashes: Vec<u64>,       // 128 hash values
}

impl MinHashSignature {
    const NUM_HASHES: usize = 128;
    
    pub fn from_components(components: &FingerprintComponents) -> Result<Self, ScrybeError> {
        // Create shingles (n-grams) from fingerprint components
        let shingles = Self::create_shingles(components);
        
        // Compute MinHash signature
        let mut hashes = vec![u64::MAX; Self::NUM_HASHES];
        
        for shingle in shingles {
            for i in 0..Self::NUM_HASHES {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                shingle.hash(&mut hasher);
                let hash_value = hasher.finish();
                
                hashes[i] = hashes[i].min(hash_value);
            }
        }
        
        Ok(Self { hashes })
    }
    
    fn create_shingles(components: &FingerprintComponents) -> Vec<String> {
        // Create 3-character shingles from all components
        let combined = format!(
            "{}:{}:{}:{}",
            components.network_hash,
            components.browser_hash,
            components.behavioral_hash,
            components.device_hash
        );
        
        combined
            .chars()
            .collect::<Vec<_>>()
            .windows(3)
            .map(|w| w.iter().collect::<String>())
            .collect()
    }
    
    pub fn jaccard_similarity(&self, other: &Self) -> f64 {
        let matching = self.hashes
            .iter()
            .zip(&other.hashes)
            .filter(|(a, b)| a == b)
            .count();
        
        matching as f64 / Self::NUM_HASHES as f64
    }
}
```

## Pipeline Executor

```rust
pub struct EnrichmentPipeline {
    geoip: Arc<GeoIpDatabase>,
    similarity_index: Arc<SimilarityIndex>,
    geoip_circuit_breaker: CircuitBreaker,
    thresholds: AnomalyThresholds,
    model_version: String,
}

/// Percentile-based thresholds (updated from historical data)
pub struct AnomalyThresholds {
    pub mouse_entropy_p5: f64,          // 5th percentile (likely bots)
    pub scroll_smoothness_p95: f64,     // 95th percentile (too smooth)
    pub time_on_page_p5: u64,           // 5th percentile (too fast)
    pub updated_at: DateTime<Utc>,
}

impl AnomalyThresholds {
    /// Load thresholds from historical data analysis
    pub async fn load_from_db(storage: &SessionStore) -> Result<Self, ScrybeError> {
        // Query ClickHouse for percentiles over last 7 days
        let query = r#"
            SELECT
                quantile(0.05)(mouse_entropy) AS entropy_p5,
                quantile(0.95)(scroll_smoothness) AS smoothness_p95,
                quantile(0.05)(time_on_page) AS time_p5
            FROM scrybe.sessions
            WHERE timestamp >= now() - INTERVAL 7 DAY
                AND bot_probability < 0.3  -- Only use likely-human sessions
        "#;
        
        // Execute query and parse results
        // ...
        
        Ok(Self {
            mouse_entropy_p5: 0.3,  // Placeholder, will be computed
            scroll_smoothness_p95: 0.95,
            time_on_page_p5: 500,
            updated_at: Utc::now(),
        })
    }
    
    /// Get default thresholds for cold start
    pub fn default() -> Self {
        Self {
            mouse_entropy_p5: 0.3,
            scroll_smoothness_p95: 0.95,
            time_on_page_p5: 500,
            updated_at: Utc::now(),
        }
    }
}

impl EnrichmentPipeline {
    pub async fn enrich(&self, session: Session) -> Result<EnrichedSession, ScrybeError> {
        // Stage 1: Generate fingerprint (always succeeds)
        let fingerprint = CompositeFingerprint::from_session(&session)?;
        
        // Stage 2: Geo enrichment with circuit breaker + graceful degradation
        let geo = match self.geoip_circuit_breaker.call(async {
            GeoEnrichment::from_ip(session.server_signals.ip, &self.geoip).await
        }).await {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("GeoIP enrichment failed, using fallback: {}", e);
                GeoEnrichment::unknown() // Graceful degradation
            }
        };
        
        // Stage 3: Similarity enrichment (independent, can fail gracefully)
        let similarity = match SimilarityEnrichment::compute(&fingerprint, &self.similarity_index).await {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Similarity enrichment failed, using empty: {}", e);
                SimilarityEnrichment::empty()
            }
        };
        
        // Stage 4: Anomaly detection (critical, must succeed or fail)
        let anomaly = AnomalyScore::detect(&session, &fingerprint, &geo)?;
        
        // Stage 5: Assemble enriched session
        Ok(EnrichedSession {
            session,
            fingerprint,
            geo,
            similarity,
            anomaly,
            enriched_at: Utc::now(),
            enrichment_version: env!("CARGO_PKG_VERSION").to_string(),
            model_version: self.model_version.clone(),
        })
    }
}

/// Circuit breaker for GeoIP service
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
}

enum CircuitState {
    Closed { failures: usize },
    Open { opened_at: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed { failures: 0 })),
            failure_threshold,
            timeout,
        }
    }
    
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        let mut state = self.state.lock().await;
        
        match *state {
            CircuitState::Open { opened_at } => {
                // Check if timeout has passed
                if opened_at.elapsed() > self.timeout {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err(/* Circuit open error */);
                }
            }
            _ => {}
        }
        
        drop(state);
        
        // Execute the function
        match f.await {
            Ok(result) => {
                // Success: reset circuit
                let mut state = self.state.lock().await;
                *state = CircuitState::Closed { failures: 0 };
                Ok(result)
            }
            Err(e) => {
                // Failure: increment counter
                let mut state = self.state.lock().await;
                match *state {
                    CircuitState::Closed { failures } => {
                        if failures + 1 >= self.failure_threshold {
                            *state = CircuitState::Open { opened_at: Instant::now() };
                            tracing::error!("Circuit breaker opened after {} failures", failures + 1);
                        } else {
                            *state = CircuitState::Closed { failures: failures + 1 };
                        }
                    }
                    CircuitState::HalfOpen => {
                        *state = CircuitState::Open { opened_at: Instant::now() };
                    }
                    _ => {}
                }
                Err(e)
            }
        }
    }
}
```

## Performance Optimizations

### Caching
```rust
use moka::future::Cache;

pub struct GeoIpDatabase {
    reader: maxminddb::Reader<Vec<u8>>,
    cache: Cache<IpAddr, GeoEnrichment>,
}

impl GeoIpDatabase {
    pub async fn lookup(&self, ip: IpAddr) -> Result<GeoInfo, ScrybeError> {
        // Check cache first
        if let Some(cached) = self.cache.get(&ip).await {
            return Ok(cached);
        }
        
        // Query database
        let result = self.reader.lookup(ip)?;
        
        // Cache result
        self.cache.insert(ip, result.clone()).await;
        
        Ok(result)
    }
}
```

### Batching
```rust
pub async fn enrich_batch(
    &self,
    sessions: Vec<Session>,
) -> Result<Vec<EnrichedSession>, ScrybeError> {
    // Process in parallel
    let futures = sessions
        .into_iter()
        .map(|s| self.enrich(s));
    
    let results = futures::future::try_join_all(futures).await?;
    
    Ok(results)
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_deterministic() {
        let session = create_test_session();
        
        let fp1 = CompositeFingerprint::from_session(&session).unwrap();
        let fp2 = CompositeFingerprint::from_session(&session).unwrap();
        
        assert_eq!(fp1.hash, fp2.hash);
    }
    
    #[test]
    fn test_minhash_similarity() {
        let sig1 = create_test_signature();
        let sig2 = create_similar_signature();
        
        let similarity = sig1.jaccard_similarity(&sig2);
        
        assert!(similarity > 0.7);
    }
}
```

## Success Criteria

1. ✅ Deterministic fingerprints
2. ✅ < 50ms enrichment time (p99)
3. ✅ > 90% test coverage
4. ✅ Graceful degradation on service failures
5. ✅ Clear metrics for each stage
6. ✅ No PII in fingerprints

## References

- RFC-0001: Core Architecture
- RFC-0003: Ingestion Gateway
- RFC-0005: Storage Schema
- MinHash: https://en.wikipedia.org/wiki/MinHash
- MaxMind GeoIP2: https://dev.maxmind.com/geoip/docs/databases
- LSH: https://en.wikipedia.org/wiki/Locality-sensitive_hashing
