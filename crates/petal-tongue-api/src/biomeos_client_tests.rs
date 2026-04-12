// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_fixture_mode() {
    let client = BiomeOSClient::new("http://test-mock:9000").with_fixture_mode(true);

    let primals = client.discover_primals().await.unwrap();
    assert_eq!(primals.len(), 5);
    assert_eq!(primals[0].id, "primal-alpha");

    let topology = client.get_topology().await.unwrap();
    assert_eq!(topology.len(), 5);

    // Health check should always succeed in mock mode
    let health = client.health_check().await.unwrap();
    assert!(health);
}

#[tokio::test]
async fn test_health_check_failure() {
    let client = BiomeOSClient::new("http://nonexistent:99999");

    let health = client.health_check().await.unwrap();
    assert!(!health);
}

#[tokio::test]
async fn test_convert_discovered_primal() {
    let discovered = DiscoveredPrimal {
        id: "test-1".to_string(),
        name: "Test Primal".to_string(),
        primal_type: "Test".to_string(),
        endpoint: "http://test:8000".to_string(),
        capabilities: vec!["test".to_string()],
        health: "healthy".to_string(),
        last_seen: 1_234_567_890,
    };

    let primal_info: PrimalInfo = discovered.into();
    assert_eq!(primal_info.id, "test-1");
    assert_eq!(primal_info.health, PrimalHealthStatus::Healthy);
}

#[test]
fn test_discovery_response_serialization() {
    let response = DiscoveryResponse {
        primals: vec![DiscoveredPrimal {
            id: "p1".to_string(),
            name: "Primal 1".to_string(),
            primal_type: "Compute".to_string(),
            endpoint: "http://localhost:8000".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            last_seen: 0,
        }],
    };
    let json = serde_json::to_string(&response).unwrap();
    let parsed: DiscoveryResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.primals.len(), 1);
    assert_eq!(parsed.primals[0].id, "p1");
}

#[test]
fn test_topology_response_serialization() {
    let response = TopologyResponse {
        nodes: vec![TopologyNode {
            id: "n1".to_string(),
            name: "Node 1".to_string(),
            node_type: "Compute".to_string(),
            status: "healthy".to_string(),
            trust_level: Some(3),
            family_id: Some("fam-1".to_string()),
            capabilities: vec!["compute".to_string()],
        }],
        edges: vec![TopologyEdge {
            from: "n1".into(),
            to: "n2".into(),
            edge_type: "conn".to_string(),
            label: None,
            capability: None,
            metrics: None,
        }],
        mode: "live".to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    let parsed: TopologyResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.nodes.len(), 1);
    assert_eq!(parsed.edges.len(), 1);
}

#[test]
fn test_discovered_primal_health_mapping() {
    for (health_str, expected) in [
        ("healthy", PrimalHealthStatus::Healthy),
        ("warning", PrimalHealthStatus::Warning),
        ("critical", PrimalHealthStatus::Critical),
        ("unknown", PrimalHealthStatus::Unknown),
    ] {
        let discovered = DiscoveredPrimal {
            id: "x".to_string(),
            name: "X".to_string(),
            primal_type: "T".to_string(),
            endpoint: "http://x".to_string(),
            capabilities: vec![],
            health: health_str.to_string(),
            last_seen: 0,
        };
        let info: PrimalInfo = discovered.into();
        assert_eq!(info.health, expected);
    }
}

#[tokio::test]
async fn test_discover_primals_success_via_wiremock() {
    let mock_server = MockServer::start().await;

    let discovery_json = serde_json::json!({
        "primals": [
            {
                "id": "p1",
                "name": "Primal 1",
                "primal_type": "Compute",
                "endpoint": "http://localhost:8000",
                "capabilities": ["compute"],
                "health": "healthy",
                "last_seen": 12345
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(discovery_json))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let primals = client.discover_primals().await.expect("discover_primals");
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id, "p1");
    assert_eq!(primals[0].name, "Primal 1");
}

#[tokio::test]
async fn test_discover_primals_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let err = client.discover_primals().await.expect_err("should fail");
    assert!(matches!(
        err,
        BiomeOsClientError::ServerError { status: 500, .. }
    ));
}

#[tokio::test]
async fn test_discover_primals_parse_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/primals"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let err = client.discover_primals().await.expect_err("should fail");
    assert!(matches!(err, BiomeOsClientError::Parse(_)));
}

#[tokio::test]
async fn test_get_topology_success_via_wiremock() {
    let mock_server = MockServer::start().await;

    let topology_json = serde_json::json!({
        "nodes": [{"id": "n1", "name": "Node 1"}],
        "edges": [
            {"from": "n1", "to": "n2", "edge_type": "conn"}
        ],
        "mode": "live"
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/topology"))
        .respond_with(ResponseTemplate::new(200).set_body_json(topology_json))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let edges = client.get_topology().await.expect("get_topology");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from.as_str(), "n1");
    assert_eq!(edges[0].to.as_str(), "n2");
}

#[tokio::test]
async fn test_get_topology_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/topology"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let err = client.get_topology().await.expect_err("should fail");
    assert!(matches!(
        err,
        BiomeOsClientError::ServerError { status: 404, .. }
    ));
}

#[tokio::test]
async fn test_get_topology_parse_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/topology"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid"))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let err = client.get_topology().await.expect_err("should fail");
    assert!(matches!(err, BiomeOsClientError::Parse(_)));
}

#[tokio::test]
async fn test_health_check_success_via_wiremock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let healthy = client.health_check().await.expect("health_check");
    assert!(healthy);
}

#[tokio::test]
async fn test_health_check_non_success_status() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
    let healthy = client.health_check().await.expect("health_check");
    assert!(!healthy);
}
