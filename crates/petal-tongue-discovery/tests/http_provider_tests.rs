// SPDX-License-Identifier: AGPL-3.0-only
//! Comprehensive tests for HTTP provider
//!
//! Tests verify HTTP discovery, error handling, and capability-based routing.
//! Requires `--features legacy-http` to run.

#![cfg(feature = "legacy-http")]
#![expect(deprecated)]

use petal_tongue_discovery::{HttpVisualizationProvider, VisualizationDataProvider};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_http_provider_creation() {
    let _provider = HttpVisualizationProvider::new("http://test:3000").unwrap();
}

#[tokio::test]
async fn test_get_primals_success() {
    let mock_server = MockServer::start().await;

    let response_json = serde_json::json!({
        "primals": [
            {
                "id": "test-primal-1",
                "name": "Test Primal 1",
                "primal_type": "compute",
                "endpoint": "http://test-1:8080",
                "capabilities": ["compute", "storage"],
                "health": "healthy",
                "last_seen": 1_234_567_890_u64
            },
            {
                "id": "test-primal-2",
                "name": "Test Primal 2",
                "primal_type": "storage",
                "endpoint": "http://test-2:8080",
                "capabilities": ["storage"],
                "health": "healthy",
                "last_seen": 1_234_567_890_u64
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_json))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let primals = provider.get_primals().await;

    assert!(primals.is_ok(), "Should fetch primals successfully");
    let primals = primals.unwrap();
    assert_eq!(primals.len(), 2, "Should have 2 primals");
    assert_eq!(primals[0].id, "test-primal-1");
    assert_eq!(primals[1].id, "test-primal-2");
}

#[tokio::test]
async fn test_get_primals_empty_response() {
    let mock_server = MockServer::start().await;

    let response_json = serde_json::json!({
        "primals": []
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_json))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let primals = provider.get_primals().await;

    assert!(primals.is_ok(), "Should handle empty response");
    assert_eq!(primals.unwrap().len(), 0, "Should have no primals");
}

#[tokio::test]
async fn test_get_primals_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let primals = provider.get_primals().await;

    assert!(primals.is_err(), "Should fail on HTTP 500");
}

#[tokio::test]
async fn test_get_primals_invalid_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let primals = provider.get_primals().await;

    assert!(primals.is_err(), "Should fail on invalid JSON");
}

#[tokio::test]
async fn test_get_topology_success() {
    let mock_server = MockServer::start().await;

    let response_json = serde_json::json!({
        "edges": [
            {
                "from": "primal-1",
                "to": "primal-2",
                "edge_type": "connection",
                "label": "test"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/topology"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_json))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let topology = provider.get_topology().await;

    assert!(topology.is_ok(), "Should fetch topology successfully");
    let edges = topology.unwrap();
    assert_eq!(edges.len(), 1, "Should have 1 edge");
    assert_eq!(edges[0].from, "primal-1");
    assert_eq!(edges[0].to, "primal-2");
}

#[tokio::test]
async fn test_get_topology_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/topology"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let topology = provider.get_topology().await;

    // Should return empty topology on 404 (graceful degradation)
    assert!(topology.is_ok(), "Should handle 404 gracefully");
    assert_eq!(topology.unwrap().len(), 0, "Should have no edges");
}

#[tokio::test]
async fn test_health_check_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_string("healthy"))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let health = provider.health_check().await;

    assert!(health.is_ok(), "Health check should succeed");
    let status = health.unwrap();
    assert!(
        status.contains("healthy") || status.contains("UP"),
        "Should indicate healthy status"
    );
}

#[tokio::test]
async fn test_health_check_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(503).set_body_string("unhealthy"))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let health = provider.health_check().await;

    // Health check returns Err on unhealthy status
    assert!(
        health.is_err(),
        "Health check should return error on unhealthy status"
    );
}

#[tokio::test]
async fn test_get_metadata() {
    let provider = HttpVisualizationProvider::new("http://test:3000").unwrap();
    let metadata = provider.get_metadata();

    assert_eq!(metadata.name, "HTTP Provider");
    assert_eq!(metadata.protocol, "http");
    assert!(
        metadata
            .capabilities
            .contains(&"visualization.primal-provider".to_string())
    );
}

#[tokio::test]
async fn test_multiple_requests_same_provider() {
    let mock_server = MockServer::start().await;

    let response_json = serde_json::json!({
        "primals": [
            {
                "id": "test-1",
                "name": "Test",
                "primal_type": "test",
                "endpoint": "http://test:8080",
                "capabilities": [],
                "health": "healthy",
                "last_seen": 0_u64
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_json))
        .expect(3) // Should be called 3 times
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();

    // Make multiple requests
    for _ in 0..3 {
        let primals = provider.get_primals().await;
        assert!(primals.is_ok());
    }
}

#[tokio::test]
async fn test_timeout_handling() {
    let mock_server = MockServer::start().await;

    // Mock server doesn't respond (simulates timeout)
    // Note: This test is limited because we can't easily simulate timeouts with wiremock
    // In production, the client has a 30s timeout configured

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();

    // Try to access non-existent endpoint
    let result = provider.get_primals().await;
    // Should error (connection refused or timeout)
    assert!(result.is_err(), "Should handle connection errors");
}

#[tokio::test]
async fn test_primal_health_status_mapping() {
    let mock_server = MockServer::start().await;

    let response_json = serde_json::json!({
        "primals": [
            {
                "id": "healthy-1",
                "name": "Healthy Primal",
                "primal_type": "test",
                "endpoint": "http://test:8080",
                "capabilities": [],
                "health": "healthy",
                "last_seen": 0_u64
            },
            {
                "id": "degraded-1",
                "name": "Degraded Primal",
                "primal_type": "test",
                "endpoint": "http://test:8081",
                "capabilities": [],
                "health": "degraded",
                "last_seen": 0_u64
            },
            {
                "id": "unhealthy-1",
                "name": "Unhealthy Primal",
                "primal_type": "test",
                "endpoint": "http://test:8082",
                "capabilities": [],
                "health": "unhealthy",
                "last_seen": 0_u64
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_json))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let primals = provider.get_primals().await.unwrap();

    assert_eq!(primals.len(), 3);
    // Health status mapping is handled by the From impl
}

#[tokio::test]
async fn test_capability_parsing() {
    let mock_server = MockServer::start().await;

    let response_json = serde_json::json!({
        "primals": [
            {
                "id": "capable-1",
                "name": "Capable Primal",
                "primal_type": "multi",
                "endpoint": "http://test:8080",
                "capabilities": ["compute", "storage", "networking"],
                "health": "healthy",
                "last_seen": 0_u64
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_json))
        .mount(&mock_server)
        .await;

    let provider = HttpVisualizationProvider::new(mock_server.uri()).unwrap();
    let primals = provider.get_primals().await.unwrap();

    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].capabilities.len(), 3);
    assert!(primals[0].capabilities.contains(&"compute".to_string()));
}
