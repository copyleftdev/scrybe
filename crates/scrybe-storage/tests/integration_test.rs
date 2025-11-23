//! Integration tests for ClickHouse storage using testcontainers.
//!
//! These tests require Docker to be running.

use scrybe_core::types::{
    BehavioralSignals, BrowserSignals, Fingerprint, FingerprintComponents, NetworkSignals, Session,
    SessionId,
};
use scrybe_storage::{ClickHouseClient, SessionWriter};
use std::net::IpAddr;
use testcontainers::{clients::Cli, core::WaitFor, GenericImage};

/// Create a test ClickHouse container.
fn create_clickhouse_container() -> GenericImage {
    GenericImage::new("clickhouse/clickhouse-server", "latest")
        .with_wait_for(WaitFor::message_on_stderr("Ready for connections"))
        .with_exposed_port(8123)
}

/// Create a test session.
fn create_test_session() -> Session {
    Session {
        id: SessionId::new(),
        timestamp: chrono::Utc::now(),
        fingerprint: Fingerprint {
            hash: "test-fingerprint-hash-123".to_string(),
            components: FingerprintComponents::default(),
            confidence: 0.95,
        },
        network: NetworkSignals {
            ip: "127.0.0.1".parse::<IpAddr>().unwrap(),
            ja3: None,
            ja4: None,
            headers: vec![],
            http_version: scrybe_core::types::HttpVersion::Http11,
        },
        browser: BrowserSignals {
            user_agent: "Mozilla/5.0 Test".to_string(),
            screen: scrybe_core::types::ScreenInfo::default(),
            canvas_hash: None,
            webgl_hash: None,
            audio_hash: None,
            fonts: vec![],
            plugins: vec![],
            timezone: "UTC".to_string(),
            language: "en-US".to_string(),
        },
        behavioral: BehavioralSignals {
            mouse_events: vec![],
            scroll_events: vec![],
            click_events: vec![],
            timing: scrybe_core::types::TimingMetrics::default(),
        },
    }
}

#[tokio::test]
#[ignore] // Requires Docker - run with `cargo test -- --ignored`
async fn test_clickhouse_client_connection() {
    let docker = Cli::default();
    let container = docker.run(create_clickhouse_container());
    let port = container.get_host_port_ipv4(8123);

    let url = format!("http://localhost:{}", port);

    // Wait a bit for ClickHouse to fully initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let client = ClickHouseClient::new(&url, "default", "default", "")
        .await
        .expect("Failed to connect to ClickHouse");

    // Test health check
    client
        .health_check()
        .await
        .expect("Health check should pass");
}

#[tokio::test]
#[ignore] // Requires Docker - run with `cargo test -- --ignored`
async fn test_schema_initialization() {
    let docker = Cli::default();
    let container = docker.run(create_clickhouse_container());
    let port = container.get_host_port_ipv4(8123);

    let url = format!("http://localhost:{}", port);
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let client = ClickHouseClient::new(&url, "default", "default", "")
        .await
        .expect("Failed to connect");

    // Initialize schema
    client
        .init_schema()
        .await
        .expect("Schema initialization should succeed");

    // Verify table exists by querying it
    let result = client
        .client()
        .query("SELECT count() FROM sessions")
        .fetch_one::<u64>()
        .await;

    assert!(result.is_ok(), "Should be able to query sessions table");
}

#[tokio::test]
#[ignore] // Requires Docker - run with `cargo test -- --ignored`
async fn test_write_single_session() {
    let docker = Cli::default();
    let container = docker.run(create_clickhouse_container());
    let port = container.get_host_port_ipv4(8123);

    let url = format!("http://localhost:{}", port);
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let client = ClickHouseClient::new(&url, "default", "default", "")
        .await
        .expect("Failed to connect");

    client.init_schema().await.expect("Schema init failed");

    let writer = SessionWriter::new(client.clone());
    let session = create_test_session();

    // Write session
    writer.write(&session).await.expect("Write should succeed");

    // Verify it was written
    let count: u64 = client
        .client()
        .query("SELECT count() FROM sessions")
        .fetch_one()
        .await
        .expect("Query should succeed");

    assert_eq!(count, 1, "Should have exactly 1 session");
}

#[tokio::test]
#[ignore] // Requires Docker - run with `cargo test -- --ignored`
async fn test_write_batch_sessions() {
    let docker = Cli::default();
    let container = docker.run(create_clickhouse_container());
    let port = container.get_host_port_ipv4(8123);

    let url = format!("http://localhost:{}", port);
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let client = ClickHouseClient::new(&url, "default", "default", "")
        .await
        .expect("Failed to connect");

    client.init_schema().await.expect("Schema init failed");

    let writer = SessionWriter::new(client.clone());

    // Create multiple test sessions
    let sessions: Vec<Session> = (0..10).map(|_| create_test_session()).collect();

    // Batch write
    writer
        .write_batch(&sessions)
        .await
        .expect("Batch write should succeed");

    // Verify count
    let count: u64 = client
        .client()
        .query("SELECT count() FROM sessions")
        .fetch_one()
        .await
        .expect("Query should succeed");

    assert_eq!(count, 10, "Should have exactly 10 sessions");
}

#[tokio::test]
#[ignore] // Requires Docker - run with `cargo test -- --ignored`
async fn test_query_by_fingerprint() {
    let docker = Cli::default();
    let container = docker.run(create_clickhouse_container());
    let port = container.get_host_port_ipv4(8123);

    let url = format!("http://localhost:{}", port);
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let client = ClickHouseClient::new(&url, "default", "default", "")
        .await
        .expect("Failed to connect");

    client.init_schema().await.expect("Schema init failed");

    let writer = SessionWriter::new(client.clone());
    let session = create_test_session();
    let fingerprint_hash = session.fingerprint.hash.clone();

    writer.write(&session).await.expect("Write should succeed");

    // Query by fingerprint
    let count: u64 = client
        .client()
        .query("SELECT count() FROM sessions WHERE fingerprint_hash = ?")
        .bind(&fingerprint_hash)
        .fetch_one()
        .await
        .expect("Query should succeed");

    assert_eq!(count, 1, "Should find session by fingerprint");
}
