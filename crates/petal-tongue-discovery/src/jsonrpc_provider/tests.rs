// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC provider tests

use std::sync::atomic::Ordering;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use crate::traits::VisualizationDataProvider;

use super::JsonRpcProvider;
use super::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

async fn create_mock_server(socket_path: &str) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;

    let handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_mock_connection(stream));
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    Ok(handle)
}

async fn handle_mock_connection(stream: UnixStream) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let response = match request.method.as_str() {
                "get_primals_extended" => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!([
                        {
                            "id": "test-primal",
                            "name": "Test Primal",
                            "primal_type": "test",
                            "endpoint": "unix:///tmp/test.sock",
                            "capabilities": ["test"],
                            "health": "Healthy",
                            "last_seen": 0
                        }
                    ])),
                    error: None,
                    id: request.id,
                },
                "get_topology" => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!([])),
                    error: None,
                    id: request.id,
                },
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                    id: request.id,
                },
            };

            let response_json = serde_json::to_string(&response).unwrap() + "\n";
            let _ = writer.write_all(response_json.as_bytes()).await;
            let _ = writer.flush().await;
        }

        line.clear();
    }
}

#[tokio::test]
async fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "get_primals_extended".to_string(),
        params: None,
        id: 1,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"get_primals_extended\""));
    assert!(json.contains("\"id\":1"));
}

#[tokio::test]
async fn test_jsonrpc_response_deserialization() {
    let json = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, 1);
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_jsonrpc_error_response() {
    let json = r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601);
}

#[tokio::test]
async fn test_jsonrpc_provider_get_primals() {
    let socket_path = "/tmp/test-jsonrpc-primals.sock";
    let _server = create_mock_server(socket_path).await.unwrap();

    let provider = JsonRpcProvider::new(socket_path);
    let primals = provider.get_primals().await.unwrap();

    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id, "test-primal");

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_jsonrpc_provider_health_check() {
    let socket_path = "/tmp/test-jsonrpc-health.sock";
    let _server = create_mock_server(socket_path).await.unwrap();

    let provider = JsonRpcProvider::new(socket_path);
    let health = provider.health_check().await.unwrap();

    assert!(health.contains("healthy"));

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_jsonrpc_provider_get_topology() {
    let socket_path = "/tmp/test-jsonrpc-topology.sock";
    let _server = create_mock_server(socket_path).await.unwrap();

    let provider = JsonRpcProvider::new(socket_path);
    let topology = provider.get_topology().await.unwrap();

    assert_eq!(topology.len(), 0);

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_jsonrpc_provider_metadata() {
    let provider = JsonRpcProvider::new("/tmp/test.sock");
    let metadata = provider.get_metadata();

    assert_eq!(metadata.name, "JSON-RPC Provider");
    assert_eq!(metadata.protocol, "jsonrpc-2.0");
    assert!(metadata.capabilities.contains(&"primals".to_string()));
}

#[tokio::test]
async fn test_jsonrpc_request_id_increment() {
    let provider = JsonRpcProvider::new("/tmp/test.sock");
    let id1 = provider.request_id.load(Ordering::SeqCst);
    provider.request_id.fetch_add(1, Ordering::SeqCst);
    let id2 = provider.request_id.load(Ordering::SeqCst);

    assert_eq!(id2, id1 + 1);
}

#[tokio::test]
async fn test_standard_socket_paths() {
    let paths = JsonRpcProvider::get_standard_socket_paths().unwrap();

    assert!(paths.len() >= 4);
    assert!(
        paths
            .iter()
            .any(|p| p.to_string_lossy().contains("biomeos"))
    );
}

#[tokio::test]
#[cfg(feature = "test-fixtures")]
async fn test_jsonrpc_discover_with_biomeos_url() {
    let socket_path = "/tmp/test-jsonrpc-discover.sock";
    let _ = std::fs::remove_file(socket_path);
    let _server = create_mock_server(socket_path).await.unwrap();

    let result = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
        "BIOMEOS_URL",
        &format!("unix://{socket_path}"),
        || async { JsonRpcProvider::discover().await },
    )
    .await;

    assert!(result.is_ok());
    let provider = result.unwrap();
    assert!(
        provider
            .get_metadata()
            .endpoint
            .contains("test-jsonrpc-discover")
    );

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_jsonrpc_get_topology_method_not_found() {
    let socket_path = "/tmp/test-jsonrpc-topology-nf.sock";
    let _ = std::fs::remove_file(socket_path);
    let _server = create_mock_server(socket_path).await.unwrap();

    let provider = JsonRpcProvider::new(socket_path);
    let topology = provider.get_topology().await.unwrap();

    assert_eq!(topology.len(), 0);

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_jsonrpc_provider_construction() {
    let provider = JsonRpcProvider::new("/nonexistent/socket.sock");
    let metadata = provider.get_metadata();
    assert_eq!(metadata.name, "JSON-RPC Provider");
    assert!(metadata.endpoint.contains("nonexistent"));
}
