// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code uses unwrap/expect for brevity"
)]
//! Comprehensive tests for offline visualization provider
//!
//! Tests verify offline sample data generation and test isolation.
//! Only compiled with the `test-fixtures` feature.

#![cfg(feature = "test-fixtures")]

use petal_tongue_discovery::{OfflineVisualizationProvider, VisualizationDataProvider};

#[tokio::test]
async fn test_offline_provider_creation() {
    let provider = OfflineVisualizationProvider::new();
    assert!(provider.is_offline());
}

#[tokio::test]
async fn test_offline_get_primals() {
    let provider = OfflineVisualizationProvider::new();
    let primals = provider.get_primals().await;

    assert!(primals.is_ok(), "Offline provider should return primals");
    let primals = primals.unwrap();

    assert!(
        !primals.is_empty(),
        "Offline provider should return non-empty sample data"
    );

    for primal in &primals {
        assert!(!primal.id.as_str().is_empty(), "Primal should have ID");
        assert!(!primal.name.is_empty(), "Primal should have name");
        assert!(!primal.endpoint.is_empty(), "Primal should have endpoint");
    }
}

#[tokio::test]
async fn test_offline_get_topology() {
    let provider = OfflineVisualizationProvider::new();
    let topology = provider.get_topology().await;

    assert!(topology.is_ok(), "Offline provider should return topology");
    let edges = topology.unwrap();

    for edge in &edges {
        assert!(!edge.from.as_str().is_empty(), "Edge should have from");
        assert!(!edge.to.as_str().is_empty(), "Edge should have to");
        assert!(!edge.edge_type.is_empty(), "Edge should have type");
    }
}

#[tokio::test]
async fn test_offline_health_check() {
    let provider = OfflineVisualizationProvider::new();
    let health = provider.health_check().await;

    assert!(health.is_ok(), "Offline health check should succeed");
    let status = health.unwrap();
    assert!(
        status.contains("degraded") || status.contains("offline"),
        "Status should indicate offline/degraded state"
    );
}

#[tokio::test]
async fn test_offline_get_metadata() {
    let provider = OfflineVisualizationProvider::new();
    let metadata = provider.get_metadata();

    assert!(metadata.name.contains("Offline"));
    assert_eq!(metadata.protocol, "offline");
    assert_eq!(metadata.endpoint, "offline://local");
}

#[tokio::test]
async fn test_offline_consistent_data() {
    let provider = OfflineVisualizationProvider::new();

    let primals1 = provider.get_primals().await.unwrap();
    let primals2 = provider.get_primals().await.unwrap();

    assert_eq!(
        primals1.len(),
        primals2.len(),
        "Offline sample data should be consistent"
    );
}

#[tokio::test]
async fn test_offline_no_side_effects() {
    let provider = OfflineVisualizationProvider::new();

    for _ in 0..10 {
        let _ = provider.get_primals().await;
        let _ = provider.get_topology().await;
        let _ = provider.health_check().await;
    }
}

#[tokio::test]
async fn test_offline_realistic_data() {
    let provider = OfflineVisualizationProvider::new();
    let primals = provider.get_primals().await.unwrap();

    for primal in &primals {
        assert!(
            primal.id.as_str().starts_with("offline-"),
            "Offline IDs should be prefixed"
        );

        assert!(!primal.name.is_empty());

        assert!(
            primal.endpoint.starts_with("capability://")
                || primal.endpoint.starts_with("offline://"),
            "Endpoint '{}' should be a valid offline URI scheme",
            primal.endpoint
        );
    }
}

#[tokio::test]
async fn test_offline_provider_isolation() {
    let provider1 = OfflineVisualizationProvider::new();
    let provider2 = OfflineVisualizationProvider::new();

    let primals1 = provider1.get_primals().await.unwrap();
    let primals2 = provider2.get_primals().await.unwrap();

    assert_eq!(primals1.len(), primals2.len());
}
