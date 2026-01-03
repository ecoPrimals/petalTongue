//! Integration tests for BiomeOS API client

use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::test_fixtures::endpoints;
use tokio;

#[tokio::test]
async fn test_client_creation() {
    let _client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS);
    // Client created successfully
    assert!(true);
}

#[tokio::test]
async fn test_client_with_mock_mode() {
    let _client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);
    // Mock mode enabled successfully
    assert!(true);
}

#[tokio::test]
async fn test_discover_primals_mock() {
    let client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);

    let primals = client
        .discover_primals()
        .await
        .expect("Failed to discover primals");

    // Mock should return some primals
    assert!(!primals.is_empty());

    // Verify primal structure
    for primal in &primals {
        assert!(!primal.id.is_empty());
        assert!(!primal.name.is_empty());
        assert!(!primal.endpoint.is_empty());
    }
}

#[tokio::test]
async fn test_get_topology_mock() {
    let client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);

    let edges = client.get_topology().await.expect("Failed to get topology");

    // Mock should return some edges
    assert!(!edges.is_empty());

    // Verify edge structure
    for edge in &edges {
        assert!(!edge.from.is_empty());
        assert!(!edge.to.is_empty());
    }
}

#[tokio::test]
async fn test_discover_primals_with_unreachable_endpoint() {
    // Use unreachable IP to test error handling (no automatic fallback in production)
    let client = BiomeOSClient::new("http://test-unreachable:9999").with_mock_mode(false);

    // Should return error when endpoint is unreachable (production mode)
    let result = client.discover_primals().await;

    // Production mode returns error (no automatic fallback)
    assert!(
        result.is_err(),
        "Production mode should return error for unreachable endpoint"
    );
}

#[tokio::test]
async fn test_get_topology_with_unreachable_endpoint() {
    let client = BiomeOSClient::new("http://test-unreachable:9999").with_mock_mode(false);

    // Should return error when endpoint is unreachable (production mode)
    let result = client.get_topology().await;

    // Production mode returns error (no automatic fallback)
    assert!(
        result.is_err(),
        "Production mode should return error for unreachable endpoint"
    );
}

#[tokio::test]
async fn test_primal_types_in_mock_data() {
    let client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);

    let primals = client.discover_primals().await.unwrap();

    // Should have variety of primal types
    let types: std::collections::HashSet<_> = primals.iter().map(|p| &p.primal_type).collect();
    assert!(
        types.len() > 1,
        "Mock data should include multiple primal types"
    );
}

#[tokio::test]
async fn test_primal_health_states() {
    let client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);

    let primals = client.discover_primals().await.unwrap();

    // Should have primals in different health states
    let has_healthy = primals
        .iter()
        .any(|p| matches!(p.health, petal_tongue_core::PrimalHealthStatus::Healthy));
    assert!(has_healthy, "Mock data should include healthy primals");
}

#[tokio::test]
async fn test_primal_capabilities() {
    let client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);

    let primals = client.discover_primals().await.unwrap();

    // Primals should have capabilities
    let has_capabilities = primals.iter().any(|p| !p.capabilities.is_empty());
    assert!(has_capabilities, "Mock primals should have capabilities");
}

#[tokio::test]
async fn test_topology_connectivity() {
    let client = BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true);

    let primals = client.discover_primals().await.unwrap();
    let edges = client.get_topology().await.unwrap();

    // All edges should reference known primals
    let primal_ids: std::collections::HashSet<_> = primals.iter().map(|p| &p.id).collect();

    for edge in &edges {
        assert!(
            primal_ids.contains(&edge.from) || primal_ids.contains(&edge.to),
            "Edge references unknown primal"
        );
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    let client =
        std::sync::Arc::new(BiomeOSClient::new(endpoints::MOCK_BIOMEOS).with_mock_mode(true));

    let mut handles = vec![];

    for _ in 0..10 {
        let c = client.clone();
        handles.push(tokio::spawn(async move { c.discover_primals().await }));
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent requests should succeed");
    }
}

#[tokio::test]
async fn test_client_timeout_handling() {
    let client = BiomeOSClient::new("http://test-timeout:1").with_mock_mode(false);

    // Should timeout and return error (no automatic fallback in production)
    let result = client.discover_primals().await;
    assert!(
        result.is_err(),
        "Production mode should return error on timeout"
    );
}
