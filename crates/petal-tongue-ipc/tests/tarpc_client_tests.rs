// SPDX-License-Identifier: AGPL-3.0-only
//! Integration tests for tarpc client
//!
//! These tests verify the tarpc client functionality.
//! Note: Requires a running tarpc server for full integration testing.

use petal_tongue_ipc::{TarpcClient, TarpcClientError};
use std::time::Duration;

#[test]
fn test_tarpc_client_creation() {
    let client = TarpcClient::new("tarpc://localhost:9001");
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.endpoint(), "tarpc://localhost:9001");
}

#[test]
fn test_tarpc_client_with_timeout() {
    let client = TarpcClient::new("tarpc://localhost:9001")
        .unwrap()
        .with_timeout(Duration::from_secs(10));

    assert_eq!(client.timeout(), Duration::from_secs(10));
}

#[test]
fn test_invalid_endpoint() {
    // Missing tarpc:// prefix
    let result = TarpcClient::new("localhost:9001");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TarpcClientError::Configuration(_)
    ));

    // Missing port
    let result = TarpcClient::new("tarpc://localhost");
    assert!(result.is_err());

    // Invalid port
    let result = TarpcClient::new("tarpc://localhost:invalid");
    assert!(result.is_err());
}

#[test]
fn test_localhost_resolution() {
    let client = TarpcClient::new("tarpc://localhost:9001").unwrap();
    assert!(client.addr().ip().is_loopback());
    assert_eq!(client.addr().port(), 9001);
}

#[test]
fn test_ip_address_parsing() {
    let client = TarpcClient::new("tarpc://127.0.0.1:9001").unwrap();
    assert_eq!(client.addr().port(), 9001);
    assert!(client.addr().ip().is_loopback());

    let client = TarpcClient::new("tarpc://192.0.2.100:9002").unwrap();
    assert_eq!(client.addr().port(), 9002);
}

#[tokio::test]
async fn test_connection_timeout() {
    // Try to connect to a non-existent server
    // Use a non-routable IP (192.0.2.1 is TEST-NET-1, reserved for documentation)
    let client = TarpcClient::new("tarpc://192.0.2.1:9999")
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    // This should timeout quickly
    let result = client.get_capabilities().await;
    assert!(result.is_err());
}

#[test]
fn test_debug_impl() {
    let client = TarpcClient::new("tarpc://localhost:9001").unwrap();
    let debug_str = format!("{client:?}");

    assert!(debug_str.contains("TarpcClient"));
    assert!(debug_str.contains("localhost:9001"));
}

// Note: Live integration tests with actual server would go here
// These require a running tarpc server and are typically run separately
// or skipped in CI unless the server is available.

#[cfg(feature = "integration-tests")]
mod live_tests {
    use super::*;
    use std::env;

    fn get_test_endpoint() -> Option<String> {
        env::var("TARPC_TEST_ENDPOINT").ok()
    }

    #[tokio::test]
    #[ignore = "Run with: cargo test --features integration-tests -- --ignored"]
    async fn test_live_health_check() {
        let endpoint = get_test_endpoint().expect("TARPC_TEST_ENDPOINT not set");
        let client = TarpcClient::new(&endpoint).unwrap();

        let health = client.health().await;
        assert!(health.is_ok(), "Health check failed: {:?}", health.err());

        let health = health.unwrap();
        assert!(!health.status.is_empty());
        assert!(!health.version.is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires live tarpc server"]
    async fn test_live_get_capabilities() {
        let endpoint = get_test_endpoint().expect("TARPC_TEST_ENDPOINT not set");
        let client = TarpcClient::new(&endpoint).unwrap();

        let capabilities = client.get_capabilities().await;
        assert!(
            capabilities.is_ok(),
            "Get capabilities failed: {:?}",
            capabilities.err()
        );

        let capabilities = capabilities.unwrap();
        assert!(!capabilities.is_empty(), "No capabilities returned");
    }

    #[tokio::test]
    #[ignore = "Requires live tarpc server"]
    async fn test_live_version() {
        let endpoint = get_test_endpoint().expect("TARPC_TEST_ENDPOINT not set");
        let client = TarpcClient::new(&endpoint).unwrap();

        let version = client.version().await;
        assert!(version.is_ok(), "Version check failed: {:?}", version.err());

        let version = version.unwrap();
        assert!(!version.version.is_empty());
        assert!(!version.tarpc_version.is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires live tarpc server"]
    async fn test_live_protocols() {
        let endpoint = get_test_endpoint().expect("TARPC_TEST_ENDPOINT not set");
        let client = TarpcClient::new(&endpoint).unwrap();

        let protocols = client.protocols().await;
        assert!(
            protocols.is_ok(),
            "Protocols check failed: {:?}",
            protocols.err()
        );

        let protocols = protocols.unwrap();
        assert!(!protocols.is_empty(), "No protocols returned");

        // Should include tarpc at minimum
        assert!(protocols.iter().any(|p| p.name == "tarpc"));
    }
}
