# RFC-0006: Session Management (Redis)

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Depends On**: RFC-0001 v0.2.0, RFC-0003 v0.2.0
- **Review**: Added nonce storage, connection pool formula, memory calculations, input validation
- **Style**: TigerStyle

## Summary

Scrybe uses Redis as a fast session cache for:
1. **Session correlation** - Link multiple requests from same visitor
2. **Real-time lookups** - Fast fingerprint → session mapping
3. **Rate limiting** - Per-IP and per-session counters
4. **Temporary storage** - Buffer before ClickHouse writes

Redis provides < 1ms access time for session data, enabling real-time bot detection decisions.

## Motivation

While ClickHouse is excellent for analytical queries, we need a fast cache for:

1. **Session continuity**: Track returning visitors across multiple page views
2. **Real-time decisions**: Quick fingerprint lookups for instant bot detection
3. **Rate limiting**: Fast counter increments for abuse prevention
4. **Write buffering**: Batch writes to ClickHouse for performance

Redis is ideal because:
- **Fast**: < 1ms latency for most operations
- **Simple**: Key-value store with data structures
- **Persistent**: Optional AOF/RDB persistence
- **Scalable**: Redis Cluster for horizontal scaling

## Design Goals (TigerStyle)

1. **Performance**: < 1ms for 99% of operations
2. **Reliability**: Data persistence with AOF
3. **Simplicity**: Clear key naming, minimal data structures
4. **Memory efficiency**: TTL-based eviction, compact encoding
5. **Safety**: Connection pooling, graceful degradation

## Redis Architecture

```
┌──────────────────────────────────────────┐
│            Redis Instance                │
├──────────────────────────────────────────┤
│  Key Space Organization:                 │
│                                          │
│  session:{uuid}           (Hash)         │
│  ├─ fingerprint_hash                     │
│  ├─ ip                                   │
│  ├─ first_seen                           │
│  ├─ last_seen                            │
│  ├─ request_count                        │
│  └─ bot_probability                      │
│                                          │
│  fingerprint:{hash}       (Set)          │
│  └─ [session_id, ...]                    │
│                                          │
│  ip:{address}             (String)       │
│  └─ request_count (TTL: 60s)             │
│                                          │
│  ratelimit:{ip}:{window}  (String)       │
│  └─ count (TTL: window)                  │
└──────────────────────────────────────────┘
```

## Key Naming Conventions

```
session:{session_id}                  # Session metadata (Hash)
fingerprint:{fingerprint_hash}        # Sessions with this fingerprint (Set)
ip:{ip_address}                       # IP metadata (Hash)
ratelimit:{ip}:{window}               # Rate limit counter (String)
cluster:{cluster_id}                  # Fingerprint cluster (Set)
anomaly:{anomaly_type}:{timestamp}    # Real-time anomaly feed (Sorted Set)
```

## Data Structures

### Session Metadata (Hash)

```redis
HSET session:123e4567-e89b-12d3-a456-426614174000
  fingerprint_hash "abc123def456..."
  ip "192.0.2.1"
  first_seen "1705920000000"           # Unix timestamp (ms)
  last_seen "1705920123000"
  request_count "5"
  bot_probability "0.75"
  geo_country "US"
  
EXPIRE session:123e4567-e89b-12d3-a456-426614174000 86400  # 24 hours
```

### Fingerprint → Sessions (Set)

```redis
SADD fingerprint:abc123def456... 
  "123e4567-e89b-12d3-a456-426614174000"
  "234e5678-f89c-12d3-a456-426614174001"
  
EXPIRE fingerprint:abc123def456... 86400
```

### IP Metadata (Hash)

```redis
HSET ip:192.0.2.1
  first_seen "1705920000000"
  last_seen "1705920123000"
  total_sessions "10"
  unique_fingerprints "3"
  avg_bot_probability "0.45"
  
EXPIRE ip:192.0.2.1 3600  # 1 hour
```

### Rate Limit Counter (String)

```redis
INCR ratelimit:192.0.2.1:60
EXPIRE ratelimit:192.0.2.1:60 60  # 60 second window

# Check if over limit
GET ratelimit:192.0.2.1:60
# Returns: "42"
```

### Anomaly Feed (Sorted Set)

```redis
ZADD anomaly:high_bot_probability 
  1705920123000 "session:123e4567..."
  1705920124000 "session:234e5678..."
  
# Get recent anomalies
ZREVRANGEBYSCORE anomaly:high_bot_probability +inf (now-300000) LIMIT 0 100
```

## Rust Integration

### Redis Client

```rust
use redis::{Client, Commands, AsyncCommands};
use deadpool_redis::{Pool, Config};

pub struct RedisCache {
    pool: Pool,
}

impl RedisCache {
    pub async fn connect(url: &str, pool_size: usize) -> Result<Self, ScrybeError> {
        let cfg = Config::from_url(url);
        let pool = cfg
            .builder(Some(redis::aio::MultiplexedConnection::builder()))
            .max_size(pool_size)
            .create_pool()
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(Self { pool })
    }
    
    /// Calculate optimal pool size based on expected load.
    /// Formula: pool_size = (expected_req_per_sec * avg_operation_duration) * safety_factor
    pub fn calculate_pool_size(req_per_sec: usize, avg_duration_ms: usize, safety_factor: f64) -> usize {
        let base_size = (req_per_sec * avg_duration_ms) as f64 / 1000.0;
        (base_size * safety_factor).ceil() as usize
    }
    
    /// Store session in cache.
    pub async fn store_session(&self, session: &Session) -> Result<(), ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("session:{}", session.id);
        
        // Store as hash
        let _: () = conn.hset_multiple(
            &key,
            &[
                ("fingerprint_hash", session.fingerprint.hash.as_str()),
                ("ip", session.server_signals.ip.to_string().as_str()),
                ("first_seen", session.timestamp.timestamp_millis().to_string().as_str()),
                ("last_seen", session.timestamp.timestamp_millis().to_string().as_str()),
                ("request_count", "1"),
            ]
        ).await
        .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        // Set TTL (24 hours)
        let _: () = conn.expire(&key, 86400).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        // Add to fingerprint → sessions mapping
        let fp_key = format!("fingerprint:{}", session.fingerprint.hash);
        let _: () = conn.sadd(&fp_key, session.id.to_string()).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        let _: () = conn.expire(&fp_key, 86400).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get session from cache.
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<CachedSession>, ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("session:{}", session_id);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        if !exists {
            return Ok(None);
        }
        
        let data: HashMap<String, String> = conn.hgetall(&key).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(Some(CachedSession::from_hash(data)?))
    }
    
    /// Find sessions by fingerprint.
    pub async fn find_by_fingerprint(
        &self,
        fingerprint_hash: &str,
    ) -> Result<Vec<Uuid>, ScrybeError> {
        // Validate fingerprint hash (prevent injection)
        if !fingerprint_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ScrybeError::Cache("Invalid fingerprint hash format".to_string()));
        }
        
        if fingerprint_hash.len() != 64 {  // SHA-256 = 64 hex chars
            return Err(ScrybeError::Cache("Fingerprint hash wrong length".to_string()));
        }
        
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("fingerprint:{}", fingerprint_hash);
        
        let session_ids: Vec<String> = conn.smembers(&key).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let ids = session_ids
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();
        
        Ok(ids)
    }
    
    /// Store nonce for replay prevention (5-minute TTL).
    pub async fn store_nonce(&self, nonce: &str, ttl: Duration) -> Result<(), ScrybeError> {
        // Validate nonce format (prevent injection)
        if !nonce.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ScrybeError::Cache("Invalid nonce format".to_string()));
        }
        
        if nonce.len() > 100 {
            return Err(ScrybeError::Cache("Nonce too long".to_string()));
        }
        
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("nonce:{}", nonce);
        
        let _: () = conn.set_ex(&key, "1", ttl.as_secs() as usize).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(())
    }
    
    /// Check if nonce exists (for replay detection).
    pub async fn has_nonce(&self, nonce: &str) -> Result<bool, ScrybeError> {
        // Validate nonce format (prevent injection)
        if !nonce.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ScrybeError::Cache("Invalid nonce format".to_string()));
        }
        
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("nonce:{}", nonce);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(exists)
    }
    
    /// Update session (increment request count).
    pub async fn update_session(&self, session_id: Uuid) -> Result<(), ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("session:{}", session_id);
        
        // Increment request count
        let _: () = conn.hincrby(&key, "request_count", 1).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        // Update last_seen
        let now = Utc::now().timestamp_millis();
        let _: () = conn.hset(&key, "last_seen", now).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(())
    }
}
```

### Cached Session Type

```rust
#[derive(Debug, Clone)]
pub struct CachedSession {
    pub fingerprint_hash: String,
    pub ip: IpAddr,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub request_count: u32,
    pub bot_probability: Option<f64>,
    pub geo_country: Option<String>,
}

impl CachedSession {
    fn from_hash(data: HashMap<String, String>) -> Result<Self, ScrybeError> {
        Ok(Self {
            fingerprint_hash: data.get("fingerprint_hash")
                .ok_or_else(|| ScrybeError::Cache("Missing fingerprint_hash".to_string()))?
                .clone(),
            
            ip: data.get("ip")
                .ok_or_else(|| ScrybeError::Cache("Missing ip".to_string()))?
                .parse()
                .map_err(|_| ScrybeError::Cache("Invalid IP".to_string()))?,
            
            first_seen: {
                let ts = data.get("first_seen")
                    .ok_or_else(|| ScrybeError::Cache("Missing first_seen".to_string()))?
                    .parse::<i64>()
                    .map_err(|_| ScrybeError::Cache("Invalid timestamp".to_string()))?;
                DateTime::from_timestamp_millis(ts)
                    .ok_or_else(|| ScrybeError::Cache("Invalid timestamp".to_string()))?
            },
            
            last_seen: {
                let ts = data.get("last_seen")
                    .ok_or_else(|| ScrybeError::Cache("Missing last_seen".to_string()))?
                    .parse::<i64>()
                    .map_err(|_| ScrybeError::Cache("Invalid timestamp".to_string()))?;
                DateTime::from_timestamp_millis(ts)
                    .ok_or_else(|| ScrybeError::Cache("Invalid timestamp".to_string()))?
            },
            
            request_count: data.get("request_count")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            
            bot_probability: data.get("bot_probability")
                .and_then(|s| s.parse().ok()),
            
            geo_country: data.get("geo_country").cloned(),
        })
    }
}
```

## Rate Limiting

### Per-IP Rate Limiter

```rust
pub struct RateLimiter {
    cache: Arc<RedisCache>,
    limit: u32,        // Max requests per window
    window: u32,       // Window in seconds
}

impl RateLimiter {
    pub async fn check(&self, ip: IpAddr) -> Result<RateLimitResult, ScrybeError> {
        let mut conn = self.cache.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("ratelimit:{}:{}", ip, self.window);
        
        // Get current count
        let count: Option<u32> = conn.get(&key).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let current = count.unwrap_or(0);
        
        if current >= self.limit {
            // Rate limit exceeded
            Ok(RateLimitResult::Exceeded {
                limit: self.limit,
                reset_at: self.get_reset_time().await?,
            })
        } else {
            // Increment counter
            let new_count: u32 = conn.incr(&key, 1).await
                .map_err(|e| ScrybeError::Cache(e.to_string()))?;
            
            // Set expiry if first request
            if new_count == 1 {
                let _: () = conn.expire(&key, self.window as usize).await
                    .map_err(|e| ScrybeError::Cache(e.to_string()))?;
            }
            
            Ok(RateLimitResult::Allowed {
                remaining: self.limit - new_count,
            })
        }
    }
    
    async fn get_reset_time(&self) -> Result<DateTime<Utc>, ScrybeError> {
        // Calculate when the window resets
        let now = Utc::now();
        let reset = now + Duration::seconds(self.window as i64);
        Ok(reset)
    }
}

pub enum RateLimitResult {
    Allowed { remaining: u32 },
    Exceeded { limit: u32, reset_at: DateTime<Utc> },
}
```

## Session Correlation

```rust
impl RedisCache {
    /// Correlate new request with existing session.
    pub async fn correlate_session(
        &self,
        fingerprint_hash: &str,
        ip: IpAddr,
    ) -> Result<Option<Uuid>, ScrybeError> {
        // Find sessions with matching fingerprint
        let sessions = self.find_by_fingerprint(fingerprint_hash).await?;
        
        if sessions.is_empty() {
            return Ok(None);
        }
        
        // Check if any match the IP
        for session_id in sessions {
            if let Some(cached) = self.get_session(session_id).await? {
                if cached.ip == ip {
                    // Same fingerprint + same IP = same session
                    self.update_session(session_id).await?;
                    return Ok(Some(session_id));
                }
            }
        }
        
        // Fingerprint match but different IP = new session
        Ok(None)
    }
}
```

## Anomaly Feed

```rust
impl RedisCache {
    /// Publish anomaly to real-time feed.
    pub async fn publish_anomaly(
        &self,
        session_id: Uuid,
        anomaly_type: &str,
        severity: &str,
    ) -> Result<(), ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("anomaly:{}", severity);
        let score = Utc::now().timestamp_millis() as f64;
        let value = format!("{}:{}", session_id, anomaly_type);
        
        // Add to sorted set
        let _: () = conn.zadd(&key, value, score).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        // Keep only last 1000 anomalies
        let _: () = conn.zremrangebyrank(&key, 0, -1001).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get recent anomalies.
    pub async fn get_recent_anomalies(
        &self,
        severity: &str,
        limit: usize,
    ) -> Result<Vec<(Uuid, String, DateTime<Utc>)>, ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let key = format!("anomaly:{}", severity);
        
        // Get recent entries with scores
        let results: Vec<(String, f64)> = conn
            .zrevrange_withscores(&key, 0, limit as isize - 1)
            .await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let anomalies = results
            .into_iter()
            .filter_map(|(value, score)| {
                let parts: Vec<&str> = value.split(':').collect();
                if parts.len() != 2 {
                    return None;
                }
                
                let session_id = Uuid::parse_str(parts[0]).ok()?;
                let anomaly_type = parts[1].to_string();
                let timestamp = DateTime::from_timestamp_millis(score as i64)?;
                
                Some((session_id, anomaly_type, timestamp))
            })
            .collect();
        
        Ok(anomalies)
    }
}
```

## Pipeline Operations

```rust
impl RedisCache {
    /// Store session and update indexes atomically using pipeline.
    pub async fn store_with_pipeline(&self, session: &Session) -> Result<(), ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let mut pipe = redis::pipe();
        
        let session_key = format!("session:{}", session.id);
        let fp_key = format!("fingerprint:{}", session.fingerprint.hash);
        let ip_key = format!("ip:{}", session.server_signals.ip);
        
        // Store session hash
        pipe.hset_multiple(
            &session_key,
            &[
                ("fingerprint_hash", session.fingerprint.hash.as_str()),
                ("ip", session.server_signals.ip.to_string().as_str()),
                // ... other fields
            ]
        ).ignore();
        
        pipe.expire(&session_key, 86400).ignore();
        
        // Add to fingerprint set
        pipe.sadd(&fp_key, session.id.to_string()).ignore();
        pipe.expire(&fp_key, 86400).ignore();
        
        // Update IP metadata
        pipe.hincrby(&ip_key, "total_sessions", 1).ignore();
        pipe.expire(&ip_key, 3600).ignore();
        
        // Execute pipeline
        pipe.query_async(&mut conn).await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        Ok(())
    }
}
```

## Monitoring

```rust
impl RedisCache {
    /// Get cache statistics.
    pub async fn get_stats(&self) -> Result<CacheStats, ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        // Parse info string (simplified)
        Ok(CacheStats {
            total_keys: self.count_keys("session:*").await?,
            memory_usage_mb: 0,  // Parse from INFO
            hit_rate: 0.0,       // Parse from INFO
        })
    }
    
    async fn count_keys(&self, pattern: &str) -> Result<usize, ScrybeError> {
        let mut conn = self.pool.get().await
            .map_err(|e| ScrybeError::Cache(e.to_string()))?;
        
        // Use SCAN instead of KEYS for production
        let mut cursor = 0;
        let mut count = 0;
        
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| ScrybeError::Cache(e.to_string()))?;
            
            count += keys.len();
            cursor = new_cursor;
            
            if cursor == 0 {
                break;
            }
        }
        
        Ok(count)
    }
}
```

## Redis Configuration

```conf
# /etc/redis/redis.conf

# Memory
maxmemory 8gb
maxmemory-policy allkeys-lru

# Persistence (AOF for durability)
appendonly yes
appendfsync everysec

# Performance
tcp-backlog 511
timeout 0
tcp-keepalive 300

# Snapshotting (optional)
save 900 1
save 300 10
save 60 10000
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_store_and_retrieve_session() {
    let cache = RedisCache::connect("redis://localhost:6379").await.unwrap();
    let session = create_test_session();
    
    cache.store_session(&session).await.unwrap();
    
    let retrieved = cache.get_session(session.id).await.unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_rate_limiter() {
    let cache = Arc::new(RedisCache::connect("redis://localhost:6379").await.unwrap());
    let limiter = RateLimiter {
        cache,
        limit: 10,
        window: 60,
    };
    
    let ip = "192.0.2.1".parse().unwrap();
    
    // First 10 requests should succeed
    for _ in 0..10 {
        let result = limiter.check(ip).await.unwrap();
        assert!(matches!(result, RateLimitResult::Allowed { .. }));
    }
    
    // 11th request should fail
    let result = limiter.check(ip).await.unwrap();
    assert!(matches!(result, RateLimitResult::Exceeded { .. }));
}
```

## Memory Requirements

### Actual Memory Calculation

**Assumptions**:
- 10,000 sessions/sec sustained
- Session metadata in Redis: ~1KB per session
- 24-hour retention in Redis
- Nonce storage: 5-minute window

**Session Storage**:
```
10,000 sessions/sec × 86,400 sec/day × 1KB = 864GB
```

**Nonce Storage**:
```
10,000 nonces/sec × 300 sec (5 min) × 32 bytes = 96MB
```

**Total**: ~864GB + overhead = **~900GB**

**Required Redis Cluster**:
- Memory needed: 900GB
- With 50% safety margin: 1,350GB
- Recommended: 4× cache.r6g.8xlarge (256GB each) = **1TB total**
- Monthly cost: ~$4,800

**Cost Optimization**:
1. Reduce session retention: 24h → 1h = **96% reduction** (36GB vs 864GB)
2. Store only fingerprint hash + essential metadata
3. Use Redis cluster with eviction policy: `allkeys-lru`

**Revised recommendation**:
- 1-hour retention = 36GB
- With safety margin: 54GB
- Use: 2× cache.r6g.2xlarge (64GB each) = 128GB total
- **Monthly cost**: ~$1,200 (vs $4,800)

## Connection Pool Sizing

### Formula

```rust
// pool_size = (req_per_sec * avg_operation_ms / 1000) * safety_factor

let pool_size = RedisCache::calculate_pool_size(
    10_000,  // requests per second
    1,       // average operation duration (1ms for Redis GET/SET)
    2.0,     // safety factor (2x for bursts)
);
// Result: (10,000 * 1 / 1000) * 2.0 = 20 connections
```

### Configuration

```rust
// Initialize with calculated pool size
let cache = RedisCache::connect(
    &config.redis_url,
    20,  // Calculated pool size
).await?;
```

## Performance Targets

| Operation | Target | Acceptable | Unacceptable |
|-----------|--------|------------|--------------|
| GET (cached) | < 0.5ms | < 1ms | > 1ms |
| SET | < 1ms | < 2ms | > 2ms |
| HGETALL | < 1ms | < 3ms | > 3ms |
| Pipeline (5 ops) | < 2ms | < 5ms | > 5ms |
| Memory per session | < 1KB | < 2KB | > 2KB |

## Success Criteria

1. ✅ < 1ms latency (p99)
2. ✅ Connection pooling working
3. ✅ Rate limiting accurate
4. ✅ TTL-based eviction
5. ✅ Pipeline operations efficient
6. ✅ Graceful degradation on failures

## References

- RFC-0001: Core Architecture
- RFC-0003: Ingestion Gateway
- Redis Documentation: https://redis.io/documentation
- redis-rs: https://docs.rs/redis/latest/redis/
- deadpool-redis: https://docs.rs/deadpool-redis/latest/deadpool_redis/
