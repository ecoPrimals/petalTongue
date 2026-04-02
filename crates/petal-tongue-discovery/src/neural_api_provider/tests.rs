// SPDX-License-Identifier: AGPL-3.0-or-later

use super::NeuralApiProvider;
use super::mock_server::create_mock_neural_api_server;
use crate::traits::VisualizationDataProvider;
use petal_tongue_core::PrimalHealthStatus;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn test_search_paths() {
    let paths = NeuralApiProvider::get_search_paths();
    assert!(!paths.is_empty());
    // Should always have /tmp as fallback
    assert!(paths.iter().any(|p| p.to_str() == Some("/tmp")));
}

#[test]
fn test_search_paths_with_xdg_runtime() {
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "XDG_RUNTIME_DIR",
        "/custom/runtime",
        || {
            let paths = NeuralApiProvider::get_search_paths();
            assert_eq!(
                paths.first().and_then(|p| p.to_str()),
                Some("/custom/runtime")
            );
        },
    );
}

#[test]
fn test_get_metadata() {
    let provider = NeuralApiProvider::with_socket_path(PathBuf::from("/tmp/test.sock"));
    let metadata = provider.get_metadata();

    assert_eq!(metadata.name, "Neural API (Central Coordinator)");
    assert!(metadata.endpoint.contains("test.sock"));
    assert_eq!(metadata.protocol, "unix+jsonrpc");
    assert!(
        metadata
            .capabilities
            .contains(&"primal-discovery".to_string())
    );
    assert!(
        metadata
            .capabilities
            .contains(&"proprioception".to_string())
    );
}

#[test]
fn test_jsonrpc_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "method": "primal.list",
        "params": {},
        "id": 1
    });
    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["method"], "primal.list");
    assert!(request["params"].is_object());
}

#[test]
fn test_jsonrpc_request_with_params() {
    let params = json!({"graph_id": "g-1"});
    let request = json!({
        "jsonrpc": "2.0",
        "method": "neural_api.load_graph",
        "params": params,
        "id": 2
    });
    assert_eq!(request["params"]["graph_id"], "g-1");
}

#[test]
fn test_search_paths_contains_uid() {
    let paths = NeuralApiProvider::get_search_paths();
    let uid = petal_tongue_core::system_info::get_current_uid();
    let run_user = format!("/run/user/{uid}");
    assert!(
        paths.iter().any(|p| p.to_str() == Some(&run_user)),
        "paths should include /run/user/<uid>"
    );
}

#[test]
fn test_parse_primal_full() {
    let primal = serde_json::json!({
        "id": "p1",
        "primal_type": "airSpring",
        "socket_path": "/run/user/1000/p1.sock",
        "capabilities": ["science.et0", "visualization"],
        "health": "healthy"
    });
    let info = NeuralApiProvider::parse_primal(&primal).unwrap();
    assert_eq!(info.id.as_str(), "p1");
    assert_eq!(info.name, "airSpring");
    assert_eq!(info.endpoint, "/run/user/1000/p1.sock");
    assert_eq!(info.capabilities.len(), 2);
    assert_eq!(info.health, PrimalHealthStatus::Healthy);
}

#[test]
fn test_parse_primal_minimal() {
    let primal = serde_json::json!({});
    let info = NeuralApiProvider::parse_primal(&primal).unwrap();
    assert_eq!(info.id.as_str(), "unknown");
    assert_eq!(info.name, "unknown");
    assert_eq!(info.endpoint, "");
    assert!(info.capabilities.is_empty());
    assert_eq!(info.health, PrimalHealthStatus::Unknown);
}

#[test]
fn test_parse_primal_health_unknown() {
    let primal = serde_json::json!({
        "id": "p2",
        "primal_type": "test",
        "health": "degraded"
    });
    let info = NeuralApiProvider::parse_primal(&primal).unwrap();
    assert_eq!(info.health, PrimalHealthStatus::Unknown);
}

#[test]
fn test_parse_primal_capabilities_empty_array() {
    let primal = serde_json::json!({
        "id": "p3",
        "primal_type": "test",
        "capabilities": []
    });
    let info = NeuralApiProvider::parse_primal(&primal).unwrap();
    assert!(info.capabilities.is_empty());
}

#[test]
fn test_jsonrpc_error_extraction() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {"code": -32600, "message": "Invalid request"},
        "id": 1
    });
    let msg = response
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("Unknown error");
    assert_eq!(msg, "Invalid request");
}

#[test]
fn test_parse_primal_socket_path_empty() {
    let primal = serde_json::json!({
        "id": "p4",
        "primal_type": "test",
        "socket_path": ""
    });
    let info = NeuralApiProvider::parse_primal(&primal).unwrap();
    assert_eq!(info.endpoint, "");
}

#[test]
fn test_parse_primal_capabilities_non_string_filtered() {
    let primal = serde_json::json!({
        "id": "p5",
        "primal_type": "test",
        "socket_path": "/tmp/p5.sock",
        "capabilities": ["valid", 123, null, "also-valid"]
    });
    let info = NeuralApiProvider::parse_primal(&primal).unwrap();
    assert_eq!(info.capabilities.len(), 2);
    assert!(info.capabilities.contains(&"valid".to_string()));
    assert!(info.capabilities.contains(&"also-valid".to_string()));
}

#[test]
fn test_topology_connection_parsing_structure() {
    let conn = serde_json::json!({
        "from": "primal-a",
        "to": "primal-b",
        "connection_type": "trust"
    });
    let from = conn["from"].as_str().unwrap_or("").to_string();
    let to = conn["to"].as_str().unwrap_or("").to_string();
    let edge_type = conn["connection_type"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    assert_eq!(from, "primal-a");
    assert_eq!(to, "primal-b");
    assert_eq!(edge_type, "trust");
}

#[test]
fn test_topology_connection_missing_fields() {
    let conn = serde_json::json!({});
    let from = conn["from"].as_str().unwrap_or("").to_string();
    let to = conn["to"].as_str().unwrap_or("").to_string();
    assert_eq!(from, "");
    assert_eq!(to, "");
}

#[test]
fn test_socket_name_format() {
    let family = "nat0";
    let socket_name = format!("biomeos-neural-api-{family}.sock");
    assert_eq!(socket_name, "biomeos-neural-api-nat0.sock");
}

#[test]
fn test_jsonrpc_result_extraction() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"health": {"status": "ok"}},
        "id": 1
    });
    let result = response.get("result").cloned().unwrap();
    let status = result["health"]["status"].as_str().unwrap();
    assert_eq!(status, "ok");
}

#[test]
fn test_primals_array_format() {
    let result = serde_json::json!({"primals": [{"id": "p1", "primal_type": "t"}]});
    let primals = result["primals"].as_array().unwrap();
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0]["id"], "p1");
}

#[test]
fn test_primals_direct_array_format() {
    let result = serde_json::json!([{"id": "p1", "primal_type": "t"}]);
    let arr = result.as_array();
    assert!(arr.is_some());
    assert_eq!(arr.unwrap().len(), 1);
}

#[tokio::test]
async fn test_neural_api_call_method_get_primals() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("neural.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = NeuralApiProvider::with_socket_path(sock_path.clone());
    let primals = provider.get_primals().await.unwrap();
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id.as_str(), "p1");
}

#[tokio::test]
async fn test_neural_api_call_method_get_topology() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("neural-topology.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = NeuralApiProvider::with_socket_path(sock_path);
    let topology = provider.get_topology().await.unwrap();
    assert_eq!(topology.len(), 1);
    assert_eq!(topology[0].from.as_str(), "p1");
    assert_eq!(topology[0].to.as_str(), "p2");
}

#[tokio::test]
async fn test_neural_api_health_check() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("neural-health.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = NeuralApiProvider::with_socket_path(sock_path);
    let health = provider.health_check().await.unwrap();
    assert!(health.contains("healthy"));
}

#[tokio::test]
async fn test_neural_api_get_proprioception() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("neural-proprio.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = NeuralApiProvider::with_socket_path(sock_path);
    let proprio = provider.get_proprioception().await.unwrap();
    assert_eq!(proprio.family_id, "test");
    assert_eq!(
        proprio.health.status,
        petal_tongue_core::ProprioceptionHealthStatus::Healthy
    );
}

#[tokio::test]
async fn test_neural_api_get_metrics() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("neural-metrics.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = NeuralApiProvider::with_socket_path(sock_path);
    let metrics = provider.get_metrics().await.unwrap();
    assert_eq!(metrics["cpu_percent"], 10);
    assert_eq!(metrics["memory_mb"], 128);
}

#[tokio::test]
async fn test_neural_api_connection_failure() {
    let provider = NeuralApiProvider::with_socket_path(PathBuf::from(
        "/tmp/nonexistent-neural-api-xyz-99999.sock",
    ));
    let result = provider.call_method("primal.list", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_neural_graph_client_save_load_list_execute() {
    use crate::NeuralGraphClient;

    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("neural-graph.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = NeuralApiProvider::with_socket_path(sock_path.clone());
    let client = NeuralGraphClient::new(&provider);

    let graph_id = client
        .save_graph(serde_json::json!({"nodes": [], "edges": []}))
        .await
        .unwrap();
    assert_eq!(graph_id, "g-saved-123");

    let graph = client.load_graph("g1").await.unwrap();
    assert!(graph.get("nodes").is_some());

    let graphs = client.list_graphs().await.unwrap();
    assert_eq!(graphs.len(), 1);
    assert_eq!(graphs[0].id, "g1");

    let exec_id = client
        .execute_graph("g1", Some(serde_json::json!({"param": 1})))
        .await
        .unwrap();
    assert_eq!(exec_id, "exec-456");

    let status = client.get_execution_status("exec-456").await.unwrap();
    assert_eq!(status.status, crate::ExecutionStatus::Completed);

    client.cancel_execution("exec-456").await.unwrap();
    client.delete_graph("g1").await.unwrap();
    client
        .update_graph_metadata("g1", Some("New".to_string()), Some("Desc".to_string()))
        .await
        .unwrap();
}

#[tokio::test]
#[cfg(feature = "test-fixtures")]
async fn test_neural_api_discover_with_mock_socket() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("biomeos-neural-api-testfam.sock");
    let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

    let provider = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
        "XDG_RUNTIME_DIR",
        dir.path().to_str().unwrap(),
        || async { NeuralApiProvider::discover(Some("testfam")).await },
    )
    .await
    .unwrap();

    assert!(
        provider
            .get_metadata()
            .endpoint
            .contains("biomeos-neural-api-testfam")
    );
}

#[tokio::test]
async fn test_neural_graph_client_connection_failure() {
    use crate::NeuralGraphClient;

    let provider = NeuralApiProvider::with_socket_path(PathBuf::from(
        "/tmp/nonexistent-neural-xyz-88888.sock",
    ));
    let client = NeuralGraphClient::new(&provider);

    let result = client.save_graph(serde_json::json!({})).await;
    assert!(result.is_err());

    let result = client.load_graph("g1").await;
    assert!(result.is_err());

    let result = client.list_graphs().await;
    assert!(result.is_err());
}
