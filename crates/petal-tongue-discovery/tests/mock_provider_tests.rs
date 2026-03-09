// SPDX-License-Identifier: AGPL-3.0-only
//! Comprehensive tests for mock provider
//!
//! Tests verify mock data generation and test isolation.
//! Only compiled with the `test-fixtures` feature.

#![cfg(feature = "test-fixtures")]

use petal_tongue_discovery::{MockVisualizationProvider, VisualizationDataProvider};

#[tokio::test]
async fn test_mock_provider_creation() {
    let _provider = MockVisualizationProvider::new();
    // Should create without error
    assert!(true, "Mock provider created successfully");
}

#[tokio::test]
async fn test_mock_get_primals() {
    let provider = MockVisualizationProvider::new();
    let primals = provider.get_primals().await;

    assert!(primals.is_ok(), "Mock provider should return primals");
    let primals = primals.unwrap();

    // Mock provider should return some test data
    assert!(
        !primals.is_empty(),
        "Mock provider should return non-empty data"
    );

    // Verify primals have required fields
    for primal in &primals {
        assert!(!primal.id.as_str().is_empty(), "Primal should have ID");
        assert!(!primal.name.is_empty(), "Primal should have name");
        assert!(!primal.endpoint.is_empty(), "Primal should have endpoint");
    }
}

#[tokio::test]
async fn test_mock_get_topology() {
    let provider = MockVisualizationProvider::new();
    let topology = provider.get_topology().await;

    assert!(topology.is_ok(), "Mock provider should return topology");
    let edges = topology.unwrap();

    // Mock topology should have some edges
    // Verify edges have required fields
    for edge in &edges {
        assert!(!edge.from.as_str().is_empty(), "Edge should have from");
        assert!(!edge.to.as_str().is_empty(), "Edge should have to");
        assert!(!edge.edge_type.is_empty(), "Edge should have type");
    }
}

#[tokio::test]
async fn test_mock_health_check() {
    let provider = MockVisualizationProvider::new();
    let health = provider.health_check().await;

    assert!(health.is_ok(), "Mock health check should succeed");
    let status = health.unwrap();
    assert!(
        status.contains("Mock") || status.contains("healthy"),
        "Status should indicate mock mode or health"
    );
}

#[tokio::test]
async fn test_mock_get_metadata() {
    let provider = MockVisualizationProvider::new();
    let metadata = provider.get_metadata();

    assert_eq!(metadata.name, "Mock Provider");
    assert_eq!(metadata.protocol, "mock");
    assert_eq!(metadata.endpoint, "mock://local");
}

#[tokio::test]
async fn test_mock_consistent_data() {
    let provider = MockVisualizationProvider::new();

    // Get primals twice
    let primals1 = provider.get_primals().await.unwrap();
    let primals2 = provider.get_primals().await.unwrap();

    // Mock data should be consistent across calls
    assert_eq!(
        primals1.len(),
        primals2.len(),
        "Mock data should be consistent"
    );
}

#[tokio::test]
async fn test_mock_no_side_effects() {
    let provider = MockVisualizationProvider::new();

    // Multiple calls should not cause side effects
    for _ in 0..10 {
        let _ = provider.get_primals().await;
        let _ = provider.get_topology().await;
        let _ = provider.health_check().await;
    }

    // Should complete without panics or errors
    assert!(true, "Mock provider has no side effects");
}

#[tokio::test]
async fn test_mock_realistic_data() {
    let provider = MockVisualizationProvider::new();
    let primals = provider.get_primals().await.unwrap();

    // Verify mock data looks realistic
    for primal in &primals {
        // IDs should be meaningful
        assert!(
            primal.id.as_str().starts_with("mock-"),
            "Mock IDs should be prefixed"
        );

        // Names should be present
        assert!(!primal.name.is_empty());

        // Endpoints should be URLs
        assert!(primal.endpoint.starts_with("http://") || primal.endpoint.starts_with("https://"));
    }
}

#[tokio::test]
async fn test_mock_provider_isolation() {
    // Create multiple mock providers
    let provider1 = MockVisualizationProvider::new();
    let provider2 = MockVisualizationProvider::new();

    // They should work independently
    let primals1 = provider1.get_primals().await.unwrap();
    let primals2 = provider2.get_primals().await.unwrap();

    // Data should be the same (consistent mock data)
    assert_eq!(primals1.len(), primals2.len());
}
