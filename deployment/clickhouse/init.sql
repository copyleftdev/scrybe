-- ClickHouse initialization script for Scrybe

-- Create database
CREATE DATABASE IF NOT EXISTS scrybe;

USE scrybe;

-- Sessions table with proper schema
CREATE TABLE IF NOT EXISTS sessions (
    -- Identity
    session_id UUID,
    timestamp DateTime64(3, 'UTC'),
    
    -- Network signals
    ip String,
    ja3 String,
    ja4 String,
    
    -- Browser signals
    user_agent String,
    canvas_hash String,
    webgl_hash String,
    audio_hash String,
    fonts Array(String),
    plugins Array(String),
    timezone String,
    language String,
    
    -- Screen info
    screen_width UInt32,
    screen_height UInt32,
    color_depth UInt8,
    pixel_ratio Float32,
    
    -- Fingerprint
    fingerprint_hash String,
    fingerprint_confidence Float64,
    
    -- Behavioral
    mouse_events_count UInt32,
    scroll_events_count UInt32,
    click_events_count UInt32,
    timing_dns UInt32,
    timing_tcp UInt32,
    timing_dom_load UInt32,
    timing_page_load UInt32,
    
    -- Metadata
    created_at DateTime64(3, 'UTC') DEFAULT now64(3)
    
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, session_id)
TTL timestamp + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- Indexes for common queries
ALTER TABLE sessions ADD INDEX idx_fingerprint_hash fingerprint_hash TYPE bloom_filter GRANULARITY 1;
ALTER TABLE sessions ADD INDEX idx_ip ip TYPE tokenbf_v1(32768, 3, 0) GRANULARITY 1;

-- Materialized view for session statistics
CREATE MATERIALIZED VIEW IF NOT EXISTS session_stats_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (timestamp, fingerprint_hash)
AS SELECT
    toStartOfHour(timestamp) as timestamp,
    fingerprint_hash,
    count() as session_count,
    uniq(session_id) as unique_sessions,
    avg(fingerprint_confidence) as avg_confidence
FROM sessions
GROUP BY timestamp, fingerprint_hash;

-- Create user (if not exists)
-- Note: This may fail in some ClickHouse versions, it's okay
CREATE USER IF NOT EXISTS scrybe IDENTIFIED BY 'scrybe_dev_password';
GRANT ALL ON scrybe.* TO scrybe;

-- Success message
SELECT 'ClickHouse schema initialized successfully' as message;
