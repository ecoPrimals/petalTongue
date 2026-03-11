// SPDX-License-Identifier: AGPL-3.0-only
//! Comprehensive tests for demo provider
//!
//! Tests verify demo data generation and test isolation.
//! Only compiled with the `test-fixtures` feature.

#![cfg(feature = "test-fixtures")]

use petal_tongue_discovery::{DemoVisualizationProvider, VisualizationDataProvider};

#[tokio::test]
async fn test_demo_provider_creation() {
    let _provider = DemoVisualizationProvider::new();
}

#[tokio::test]
async fn test_demo_get_primals() {
    let provider = DemoVisualizationProvider::new();
    let primals = provider.get_primals().await;

    assert!(primals.is_ok(), "Demo provider should return primals");
    let primals = primals.unwrap();

    // Demo provider should return some test data
    assert!(
        !primals.is_empty(),
        "Demo provider should return non-empty data"
    );

    // Verify primals have required fields
    for primal in &primals {
        assert!(!primal.id.as_str().is_empty(), "Primal should have ID");
        assert!(!primal.name.is_empty(), "Primal should have name");
        assert!(!primal.endpoint.is_empty(), "Primal should have endpoint");
    }
}

#[tokio::test]
async fn test_demo_get_topology() {
    let provider = DemoVisualizationProvider::new();
    let topology = provider.get_topology().await;

    assert!(topology.is_ok(), "Demo provider should return topology");
    let edges = topology.unwrap();

    // Demo topology should have some edges
    // Verify edges have required fields
    for edge in &edges {
        assert!(!edge.from.as_str().is_empty(), "Edge should have from");
        assert!(!edge.to.as_str().is_empty(), "Edge should have to");
        assert!(!edge.edge_type.is_empty(), "Edge should have type");
    }
}

#[tokio::test]
async fn test_demo_health_check() {
    let provider = DemoVisualizationProvider::new();
    let health = provider.health_check().await;

    assert!(health.is_ok(), "Demo health check should succeed");
    let status = health.unwrap();
    assert!(
        status.contains("Demo") || status.contains("healthy"),
        "Status should indicate demo mode or health"
    );
}

#[tokio::test]
async fn test_demo_get_metadata() {
    let provider = DemoVisualizationProvider::new();
    let metadata = provider.get_metadata();

    assert_eq!(metadata.name, "Demo Provider");
    assert_eq!(metadata.protocol, "demo");
    assert_eq!(metadata.endpoint, "demo://local");
}

#[tokio::test]
async fn test_demo_consistent_data() {
    let provider = DemoVisualizationProvider::new();

    // Get primals twice
    let primals1 = provider.get_primals().await.unwrap();
    let primals2 = provider.get_primals().await.unwrap();

    // Demo data should be consistent across calls
    assert_eq!(
        primals1.len(),
        primals2.len(),
        "Demo data should be consistent"
    );
}

#[tokio::test]
async fn test_demo_no_side_effects() {
    let provider = DemoVisualizationProvider::new();

    // Multiple calls should not cause side effects
    for _ in 0..10 {
        let _ = provider.get_primals().await;
        let _ = provider.get_topology().await;
        let _ = provider.health_check().await;
    }

    // Should complete without panics or errors
}

#[tokio::test]
async fn test_demo_realistic_data() {
    let provider = DemoVisualizationProvider::new();
    let primals = provider.get_primals().await.unwrap();

    // Verify demo data looks realistic
    for primal in &primals {
        // IDs should be meaningful
        assert!(
            primal.id.as_str().starts_with("demo-"),
            "Demo IDs should be prefixed"
        );

        // Names should be present
        assert!(!primal.name.is_empty());

        // Endpoints should be URLs
        assert!(primal.endpoint.starts_with("http://") || primal.endpoint.starts_with("https://"));
    }
}

#[tokio::test]
async fn test_demo_provider_isolation() {
    // Create multiple demo providers
    let provider1 = DemoVisualizationProvider::new();
    let provider2 = DemoVisualizationProvider::new();

    // They should work independently
    let primals1 = provider1.get_primals().await.unwrap();
    let primals2 = provider2.get_primals().await.unwrap();

    // Data should be the same (consistent demo data)
    assert_eq!(primals1.len(), primals2.len());
}
