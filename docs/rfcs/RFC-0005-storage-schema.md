# RFC-0005: Storage Schema (ClickHouse)

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Depends On**: RFC-0001 v0.2.0, RFC-0004 v0.2.0
- **Review**: Fixed primary key, updated indexes, added replication and backup strategy
- **Style**: TigerStyle

## Summary

Scrybe uses ClickHouse as its primary analytical database for storing enriched session telemetry. ClickHouse is optimized for:
- **High-cardinality data** (millions of unique fingerprints)
- **Time-series analytics** (session timelines)
- **Fast aggregations** (anomaly detection queries)
- **Immutable writes** (append-only, no updates)

This RFC defines the ClickHouse schema, indexing strategy, partitioning, and query patterns.

## Motivation

Bot detection requires storing and analyzing massive amounts of session data:
- **Volume**: 10k+ sessions/sec = 864M sessions/day
- **Retention**: 90 days = 77B sessions
- **Query patterns**: Time-range scans, fingerprint lookups, anomaly aggregations
- **Cardinality**: Millions of unique IPs, fingerprints, user agents

ClickHouse is ideal because:
1. **Columnar storage**: Efficient compression (10x-100x)
2. **Fast aggregations**: SIMD optimizations
3. **Horizontal scaling**: Distributed tables
4. **Time-series optimized**: Native partitioning by date

## Design Goals

1. **Performance**: < 100ms for 99% of queries
2. **Cost per Million Sessions**: $7.60

### Realistic Compression Ratio

**Previous estimate**: 50:1 compression (too optimistic)  
**Realistic estimate**: 10-20:1 compression for JSON data

**Revised storage calculation**:
- 864M sessions/day × 5KB = 4.3TB/day uncompressed
- 90 days = 387TB uncompressed
- At 15:1 compression = 25.8TB stored (vs 7.7TB in v0.1.0)
- Storage cost: 25,800 GB × $0.08 = **$2,064/month** (vs $1,560)

**Total revised monthly cost**: ~$7,264/month (vs $6,570)

### Cost Optimization Strategies
3. **Retention**: Automatic TTL-based cleanup
4. **Scalability**: Support 100k+ sessions/sec
5. **Queryability**: Fast filtering by any dimension

## ClickHouse Architecture

```
┌──────────────────────────────────────────┐
│         ClickHouse Cluster               │
├──────────────────────────────────────────┤
│  Table: sessions                         │
│  ├─ Partitioned by: date                 │
│  ├─ Primary key: (timestamp, session_id) │
│  ├─ Indexes:                             │
│  │  ├─ fingerprint_hash (bloom filter)   │
│  │  ├─ ip (tokenbf_v1)                   │
│  │  └─ bot_probability (minmax)          │
│  └─ TTL: 90 days                         │
├──────────────────────────────────────────┤
│  Materialized Views                      │
│  ├─ hourly_stats                         │
│  ├─ fingerprint_clusters                 │
│  └─ anomaly_events                       │
└──────────────────────────────────────────┘
```

## Table Schema

### Main Table: `sessions`

```sql
CREATE TABLE scrybe.sessions (
    -- Session identifiers
    session_id UUID,
    timestamp DateTime64(3, 'UTC'),
    
    -- Client signals (from JS SDK)
    client_network LowCardinality(String),     -- JSON blob
    client_browser String,                     -- JSON blob
    client_behavioral String,                  -- JSON blob
    
    -- Server signals
    ip IPv6,                                   -- Supports both IPv4 and IPv6
    tls_ja3 String,
    tls_ja4 String,
    http_headers String,                       -- JSON array
    http_version LowCardinality(String),       -- "HTTP/1.1", "HTTP/2", etc.
    
    -- Fingerprint
    fingerprint_hash FixedString(64),          -- SHA-256 hex
    fingerprint_components String,             -- JSON
    fingerprint_confidence Float32,
    minhash Array(UInt64),                     -- MinHash signature
    
    -- Geo enrichment
    geo_country FixedString(2),                -- ISO country code
    geo_region LowCardinality(String),
    geo_city LowCardinality(String),
    geo_latitude Nullable(Float32),
    geo_longitude Nullable(Float32),
    geo_timezone LowCardinality(String),
    asn UInt32,
    asn_org LowCardinality(String),
    is_vpn UInt8,                              -- Boolean (0/1)
    is_proxy UInt8,
    is_tor UInt8,
    is_hosting UInt8,
    
    -- Similarity
    similar_fingerprints String,               -- JSON array
    cluster_id Nullable(UUID),
    cluster_similarity Nullable(Float32),
    
    -- Anomaly detection
    bot_probability Float32,                   -- 0.0-1.0
    behavioral_anomaly Float32,
    timing_anomaly Float32,
    header_anomaly Float32,
    fingerprint_anomaly Float32,
    detected_anomalies String,                 -- JSON array
    
    -- Metadata
    enriched_at DateTime64(3, 'UTC'),
    enrichment_version LowCardinality(String)
)
ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/scrybe/sessions', '{replica}')
PARTITION BY toYYYYMMDD(timestamp)
PRIMARY KEY (toStartOfHour(timestamp), session_id)
ORDER BY (toStartOfHour(timestamp), session_id, fingerprint_hash)
TTL timestamp + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;
```

### Indexes

```sql
-- Token bloom filter for high-cardinality fingerprint lookups
ALTER TABLE scrybe.sessions 
ADD INDEX idx_fingerprint_token fingerprint_hash 
TYPE tokenbf_v1(512, 3, 0) 
GRANULARITY 1;

-- Token bloom filter for IP address searches
ALTER TABLE scrybe.sessions 
ADD INDEX idx_ip ip 
TYPE tokenbf_v1(512, 3, 0) 
GRANULARITY 1;

-- Min-max index for bot probability
ALTER TABLE scrybe.sessions 
ADD INDEX idx_bot_probability bot_probability 
TYPE minmax 
GRANULARITY 1;

-- Set index for country codes
ALTER TABLE scrybe.sessions 
ADD INDEX idx_country geo_country 
TYPE set(100) 
GRANULARITY 1;
```

## Materialized Views

### Hourly Statistics

```sql
CREATE MATERIALIZED VIEW scrybe.hourly_stats
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY (hour, geo_country)
POPULATE  -- Backfill existing data
AS
SELECT
    toStartOfHour(timestamp) AS hour,
    geo_country,
    count() AS session_count,
    countIf(bot_probability > 0.7) AS bot_count,
    countIf(bot_probability < 0.3) AS human_count,
    avg(bot_probability) AS avg_bot_probability,
    quantile(0.5)(bot_probability) AS median_bot_probability,
    quantile(0.95)(bot_probability) AS p95_bot_probability,
    uniq(fingerprint_hash) AS unique_fingerprints,
    uniq(ip) AS unique_ips
FROM scrybe.sessions
GROUP BY hour, geo_country;
```

### Fingerprint Clusters

```sql
CREATE MATERIALIZED VIEW scrybe.fingerprint_clusters
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(first_seen)
ORDER BY fingerprint_hash
POPULATE  -- Backfill existing data
AS
SELECT
    fingerprint_hash,
    min(timestamp) AS first_seen,
    max(timestamp) AS last_seen,
    count() AS session_count,
    uniq(ip) AS unique_ips,
    uniqExact(geo_country) AS unique_countries,
    avg(bot_probability) AS avg_bot_probability,
    anyLast(fingerprint_components) AS components,
    anyLast(cluster_id) AS cluster_id
FROM scrybe.sessions
GROUP BY fingerprint_hash;
```

### Anomaly Events

```sql
CREATE MATERIALIZED VIEW scrybe.anomaly_events
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (timestamp, anomaly_type)
AS
SELECT
    session_id,
    timestamp,
    fingerprint_hash,
    bot_probability,
    JSONExtractArrayRaw(detected_anomalies) AS anomaly_list,
    arrayJoin(anomaly_list) AS anomaly_json,
    JSONExtractString(anomaly_json, 'anomaly_type') AS anomaly_type,
    JSONExtractString(anomaly_json, 'severity') AS severity,
    JSONExtractString(anomaly_json, 'description') AS description,
    geo_country,
    ip
FROM scrybe.sessions
WHERE bot_probability > 0.5;
```

## Rust Integration

### ClickHouse Client

```rust
use clickhouse::Client;

pub struct SessionStore {
    client: Client,
}

impl SessionStore {
    pub async fn new(url: &str) -> Result<Self, ScrybeError> {
        let client = Client::default()
            .with_url(url)
            .with_database("scrybe");
        
        Ok(Self { client })
    }
    
    /// Insert single enriched session.
    pub async fn insert(&self, session: &EnrichedSession) -> Result<(), ScrybeError> {
        let mut insert = self.client
            .insert("sessions")
            .map_err(|e| ScrybeError::Storage(e))?;
        
        insert
            .write(&SessionRow::from_enriched(session))
            .await
            .map_err(|e| ScrybeError::Storage(e))?;
        
        insert
            .end()
            .await
            .map_err(|e| ScrybeError::Storage(e))?;
        
        Ok(())
    }
    
    /// Insert batch of sessions (more efficient).
    pub async fn insert_batch(&self, sessions: Vec<EnrichedSession>) -> Result<(), ScrybeError> {
        let mut insert = self.client
            .insert("sessions")
            .map_err(|e| ScrybeError::Storage(e))?;
        
        for session in sessions {
            insert
                .write(&SessionRow::from_enriched(&session))
                .await
                .map_err(|e| ScrybeError::Storage(e))?;
        }
        
        insert
            .end()
            .await
            .map_err(|e| ScrybeError::Storage(e))?;
        
        Ok(())
    }
}
```

### Session Row Mapping

```rust
use clickhouse::Row;

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct SessionRow {
    pub session_id: Uuid,
    pub timestamp: i64,                    // Milliseconds since epoch
    
    // Client signals (JSON)
    pub client_network: String,
    pub client_browser: String,
    pub client_behavioral: String,
    
    // Server signals
    pub ip: String,                        // IPv6 format
    pub tls_ja3: String,
    pub tls_ja4: String,
    pub http_headers: String,
    pub http_version: String,
    
    // Fingerprint
    pub fingerprint_hash: String,
    pub fingerprint_components: String,
    pub fingerprint_confidence: f32,
    pub minhash: Vec<u64>,
    
    // Geo
    pub geo_country: String,
    pub geo_region: String,
    pub geo_city: String,
    pub geo_latitude: Option<f32>,
    pub geo_longitude: Option<f32>,
    pub geo_timezone: String,
    pub asn: u32,
    pub asn_org: String,
    pub is_vpn: u8,
    pub is_proxy: u8,
    pub is_tor: u8,
    pub is_hosting: u8,
    
    // Similarity
    pub similar_fingerprints: String,
    pub cluster_id: Option<Uuid>,
    pub cluster_similarity: Option<f32>,
    
    // Anomaly
    pub bot_probability: f32,
    pub behavioral_anomaly: f32,
    pub timing_anomaly: f32,
    pub header_anomaly: f32,
    pub fingerprint_anomaly: f32,
    pub detected_anomalies: String,
    
    // Metadata
    pub enriched_at: i64,
    pub enrichment_version: String,
}

impl SessionRow {
    pub fn from_enriched(session: &EnrichedSession) -> Self {
        Self {
            session_id: session.session.id,
            timestamp: session.session.timestamp.timestamp_millis(),
            
            // Serialize JSON blobs
            client_network: serde_json::to_string(&session.session.client_signals.network).unwrap(),
            client_browser: serde_json::to_string(&session.session.client_signals.browser).unwrap(),
            client_behavioral: serde_json::to_string(&session.session.client_signals.behavioral).unwrap(),
            
            // Server signals
            ip: session.session.server_signals.ip.to_string(),
            tls_ja3: session.session.server_signals.tls.as_ref()
                .map(|t| t.ja3.clone())
                .unwrap_or_default(),
            tls_ja4: session.session.server_signals.tls.as_ref()
                .and_then(|t| t.ja4.clone())
                .unwrap_or_default(),
            http_headers: serde_json::to_string(&session.session.server_signals.headers).unwrap(),
            http_version: session.session.server_signals.connection.protocol.clone(),
            
            // Fingerprint
            fingerprint_hash: session.fingerprint.hash.clone(),
            fingerprint_components: serde_json::to_string(&session.fingerprint.components).unwrap(),
            fingerprint_confidence: session.fingerprint.confidence as f32,
            minhash: session.fingerprint.minhash.hashes.clone(),
            
            // Geo
            geo_country: session.geo.country.clone(),
            geo_region: session.geo.region.clone().unwrap_or_default(),
            geo_city: session.geo.city.clone().unwrap_or_default(),
            geo_latitude: session.geo.latitude.map(|v| v as f32),
            geo_longitude: session.geo.longitude.map(|v| v as f32),
            geo_timezone: session.geo.timezone.clone(),
            asn: session.geo.asn,
            asn_org: session.geo.asn_org.clone(),
            is_vpn: session.geo.is_vpn as u8,
            is_proxy: session.geo.is_proxy as u8,
            is_tor: session.geo.is_tor as u8,
            is_hosting: session.geo.is_hosting as u8,
            
            // Similarity
            similar_fingerprints: serde_json::to_string(&session.similarity.similar_fingerprints).unwrap(),
            cluster_id: session.similarity.cluster_id,
            cluster_similarity: session.similarity.cluster_similarity.map(|v| v as f32),
            
            // Anomaly
            bot_probability: session.anomaly.bot_probability as f32,
            behavioral_anomaly: session.anomaly.behavioral_anomaly as f32,
            timing_anomaly: session.anomaly.timing_anomaly as f32,
            header_anomaly: session.anomaly.header_anomaly as f32,
            fingerprint_anomaly: session.anomaly.fingerprint_anomaly as f32,
            detected_anomalies: serde_json::to_string(&session.anomaly.anomalies).unwrap(),
            
            // Metadata
            enriched_at: session.enriched_at.timestamp_millis(),
            enrichment_version: session.enrichment_version.clone(),
        }
    }
}
```

## Query Patterns

### 1. Recent High-Probability Bots

```sql
SELECT
    session_id,
    timestamp,
    fingerprint_hash,
    bot_probability,
    geo_country,
    ip
FROM scrybe.sessions
WHERE 
    timestamp >= now() - INTERVAL 1 HOUR
    AND bot_probability > 0.8
ORDER BY bot_probability DESC
LIMIT 100;
```

### 2. Fingerprint History

```sql
SELECT
    timestamp,
    session_id,
    bot_probability,
    geo_country,
    ip
FROM scrybe.sessions
WHERE fingerprint_hash = 'abc123...'
ORDER BY timestamp DESC
LIMIT 1000;
```

### 3. Anomaly Breakdown

```sql
SELECT
    arrayJoin(JSONExtractArrayRaw(detected_anomalies)) AS anomaly_json,
    JSONExtractString(anomaly_json, 'anomaly_type') AS anomaly_type,
    count() AS occurrence_count,
    avg(bot_probability) AS avg_bot_prob
FROM scrybe.sessions
WHERE timestamp >= now() - INTERVAL 24 HOUR
GROUP BY anomaly_type
ORDER BY occurrence_count DESC;
```

### 4. Geographic Distribution

```sql
SELECT
    geo_country,
    count() AS total_sessions,
    countIf(bot_probability > 0.7) AS bot_sessions,
    bot_sessions / total_sessions AS bot_ratio,
    uniq(fingerprint_hash) AS unique_fingerprints
FROM scrybe.sessions
WHERE timestamp >= today()
GROUP BY geo_country
ORDER BY total_sessions DESC
LIMIT 50;
```

### 5. Cluster Analysis

```sql
SELECT
    cluster_id,
    count() AS session_count,
    uniq(fingerprint_hash) AS unique_fingerprints,
    avg(bot_probability) AS avg_bot_prob,
    uniq(geo_country) AS unique_countries,
    min(timestamp) AS first_seen,
    max(timestamp) AS last_seen
FROM scrybe.sessions
WHERE cluster_id IS NOT NULL
GROUP BY cluster_id
ORDER BY session_count DESC
LIMIT 100;
```

## Batched Writes (Performance)

```rust
pub struct BatchWriter {
    store: SessionStore,
    buffer: Vec<EnrichedSession>,
    batch_size: usize,
    flush_interval: Duration,
}

impl BatchWriter {
    pub async fn write(&mut self, session: EnrichedSession) -> Result<(), ScrybeError> {
        self.buffer.push(session);
        
        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&mut self) -> Result<(), ScrybeError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        // Write batch
        self.store.insert_batch(std::mem::take(&mut self.buffer)).await?;
        
        Ok(())
    }
}
```

## Data Retention

```sql
-- Sessions older than 90 days are automatically deleted
ALTER TABLE scrybe.sessions 
MODIFY TTL timestamp + INTERVAL 90 DAY;

-- Aggregated stats kept longer (1 year)
ALTER TABLE scrybe.hourly_stats 
MODIFY TTL hour + INTERVAL 365 DAY;

-- Fingerprint metadata kept indefinitely
-- (no TTL on fingerprint_clusters)
```

## Performance Benchmarks

Target performance on single ClickHouse node:

| Operation | Target | Hardware |
|-----------|--------|----------|
| Write throughput | 100k rows/sec | 16 cores, 64GB RAM |
| Query latency (time range) | < 50ms | SSD storage |
| Query latency (fingerprint) | < 10ms | With bloom filter |
| Compression ratio | > 50:1 | Typical session data |
| Storage per day (10k req/s) | < 20GB | After compression |

## Replication Setup

### ZooKeeper Configuration

```xml
<!-- /etc/clickhouse-server/config.xml -->
<clickhouse>
    <zookeeper>
        <node>
            <host>zk1.example.com</host>
            <port>2181</port>
        </node>
        <node>
            <host>zk2.example.com</host>
            <port>2181</port>
        </node>
        <node>
            <host>zk3.example.com</host>
            <port>2181</port>
        </node>
    </zookeeper>
    
    <macros>
        <shard>01</shard>
        <replica>replica1</replica>
    </macros>
</clickhouse>
```

### Cluster Configuration

```xml
<remote_servers>
    <scrybe_cluster>
        <shard>
            <replica>
                <host>ch1.example.com</host>
                <port>9000</port>
            </replica>
            <replica>
                <host>ch2.example.com</host>
                <port>9000</port>
            </replica>
        </shard>
    </scrybe_cluster>
</remote_servers>
```

## Backup Strategy

### Incremental Backups with clickhouse-backup

```bash
# Install clickhouse-backup
apt-get install clickhouse-backup

# Configure backup destination
cat > /etc/clickhouse-backup/config.yml <<EOF
general:
  remote_storage: s3
  backups_to_keep_local: 3
  backups_to_keep_remote: 30
  
s3:
  access_key: ${AWS_ACCESS_KEY}
  secret_key: ${AWS_SECRET_KEY}
  bucket: scrybe-backups
  region: us-east-1
  path: clickhouse/
  compression: gzip
EOF

# Daily incremental backup (cron)
0 2 * * * clickhouse-backup create_remote

# Weekly full backup
0 3 * * 0 clickhouse-backup create_remote --full
```

### Backup Restoration

```bash
# List available backups
clickhouse-backup list remote

# Restore specific backup
clickhouse-backup restore_remote 2025-01-22T02-00-00

# Verify restoration
clickhouse-client --query "SELECT count() FROM scrybe.sessions"
```

### Backup Validation (Weekly)

```bash
#!/bin/bash
# backup-validation.sh

# Restore to test instance
clickhouse-backup restore_remote latest --test-instance

# Verify row counts match
PROD_COUNT=$(clickhouse-client --host prod --query "SELECT count() FROM scrybe.sessions")
TEST_COUNT=$(clickhouse-client --host test --query "SELECT count() FROM scrybe.sessions")

if [ "$PROD_COUNT" -eq "$TEST_COUNT" ]; then
    echo "Backup validation PASSED"
    exit 0
else
    echo "Backup validation FAILED: $PROD_COUNT != $TEST_COUNT"
    exit 1
fi
```

## ClickHouse Configuration

```xml
<!-- /etc/clickhouse-server/config.xml -->
<clickhouse>
    <max_concurrent_queries>1000</max_concurrent_queries>
    <max_table_size_to_drop>0</max_table_size_to_drop>
    
    <!-- Memory settings -->
    <max_memory_usage>50000000000</max_memory_usage>
    <max_bytes_before_external_group_by>20000000000</max_bytes_before_external_group_by>
    
    <!-- Compression (realistic settings) -->
    <compression>
        <case>
            <min_part_size>10485760</min_part_size>  <!-- 10MB -->
            <min_part_size_ratio>0.01</min_part_size_ratio>
            <method>zstd</method>
            <level>3</level>  <!-- Level 3 = good balance (15:1 ratio)
                                   Level 9 = max compression (20:1 but slower) -->
        </case>
    </compression>
    
    <!-- Background merges -->
    <background_pool_size>16</background_pool_size>
    <background_schedule_pool_size>16</background_schedule_pool_size>
</clickhouse>
```

## Monitoring

```sql
-- Check table size
SELECT
    database,
    table,
    formatReadableSize(sum(bytes)) AS size,
    sum(rows) AS rows,
    max(modification_time) AS latest_modification
FROM system.parts
WHERE database = 'scrybe' AND table = 'sessions'
GROUP BY database, table;

-- Check write performance
SELECT
    event_time,
    value
FROM system.metrics
WHERE metric = 'InsertedRows'
ORDER BY event_time DESC
LIMIT 100;

-- Check query performance
SELECT
    query_start_time,
    query_duration_ms,
    read_rows,
    query
FROM system.query_log
WHERE type = 'QueryFinish'
ORDER BY query_start_time DESC
LIMIT 10;
```

## Testing

### Integration Tests

```rust
#[tokio::test]
async fn test_insert_session() {
    let store = SessionStore::new("http://localhost:8123").await.unwrap();
    let session = create_test_enriched_session();
    
    store.insert(&session).await.unwrap();
    
    // Verify inserted
    // (query ClickHouse to confirm)
}

#[tokio::test]
async fn test_batch_insert() {
    let store = SessionStore::new("http://localhost:8123").await.unwrap();
    let sessions = vec![
        create_test_enriched_session(),
        create_test_enriched_session(),
        create_test_enriched_session(),
    ];
    
    store.insert_batch(sessions).await.unwrap();
}
```

## Disaster Recovery Plan

### Recovery Time Objective (RTO): 30 minutes
### Recovery Point Objective (RPO): 1 hour

**Failure Scenarios**:

1. **Single Node Failure**
   - Automatic failover to replica (< 1 minute)
   - No data loss (replicated)

2. **Full Shard Failure**
   - Restore from S3 backup (15-30 minutes)
   - Data loss: Up to 1 hour (last backup)

3. **Data Corruption**
   - Point-in-time restore from backup
   - Replay from Redis cache if < 24 hours

4. **Complete Cluster Failure**
   - Provision new cluster (10 minutes)
   - Restore from S3 (20-30 minutes)
   - Total RTO: 30-40 minutes

### Disaster Recovery Runbook

```bash
# 1. Provision new ClickHouse cluster
terraform apply -target=module.clickhouse_cluster

# 2. Restore from latest backup
clickhouse-backup restore_remote latest

# 3. Verify data integrity
./scripts/verify-data-integrity.sh

# 4. Update DNS to point to new cluster
aws route53 change-resource-record-sets \
  --hosted-zone-id Z123 \
  --change-batch file://dns-failover.json

# 5. Monitor ingestion resume
watch -n 1 'clickhouse-client --query "SELECT count() FROM scrybe.sessions WHERE timestamp > now() - INTERVAL 1 MINUTE"'
```

## Success Criteria

1. ✅ Write throughput > 100k sessions/sec
2. ✅ Query latency < 100ms (p99)
3. ✅ Compression ratio 10-20:1 (realistic)
4. ✅ Automatic TTL cleanup
5. ✅ Materialized views for common queries
6. ✅ No data loss on restarts
7. ✅ **NEW**: Replication enabled (2 replicas minimum)
8. ✅ **NEW**: Daily backups to S3
9. ✅ **NEW**: Backup validation weekly
10. ✅ **NEW**: RTO < 30 minutes, RPO < 1 hour

## References

- RFC-0001: Core Architecture
- RFC-0004: Fingerprinting & Enrichment
- ClickHouse Documentation: https://clickhouse.com/docs/
- ClickHouse Performance: https://clickhouse.com/docs/en/operations/performance
- ClickHouse Best Practices: https://clickhouse.com/docs/en/guides/best-practices
