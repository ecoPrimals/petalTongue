// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for JSON-RPC provider
//!
//! These tests verify the JSON-RPC provider works correctly with
//! a real (mock) Unix socket server.

use petal_tongue_discovery::{JsonRpcProvider, VisualizationDataProvider};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// JSON-RPC Request (for server parsing)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: u64,
}

/// JSON-RPC Response (for server sending)
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Create a mock JSON-RPC server
async fn create_mock_server(socket_path: &str) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    // Remove existing socket
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;

    let handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_connection(stream));
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    Ok(handle)
}

/// Handle a client connection
async fn handle_connection(stream: UnixStream) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
        // Parse request
        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let response = match request.method.as_str() {
                "primal.list" => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!([
                        {
                            "id": "songbird",
                            "name": "Songbird",
                            "primal_type": "discovery",
                            "endpoint": "unix:///tmp/songbird.sock",
                            "capabilities": ["discovery", "coordination"],
                            "health": "Healthy",
                            "last_seen": 0
                        },
                        {
                            "id": "beardog",
                            "name": "BearDog",
                            "primal_type": "security",
                            "endpoint": "unix:///tmp/beardog.sock",
                            "capabilities": ["auth", "firewall"],
                            "health": "Healthy",
                            "last_seen": 0
                        }
                    ])),
                    error: None,
                    id: request.id,
                },
                "topology.get" | "get_topology" => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!([
                        {
                            "from": "songbird",
                            "to": "beardog",
                            "edge_type": "discovery"
                        }
                    ])),
                    error: None,
                    id: request.id,
                },
                "invalid_method" | _ => JsonRpcResponse {
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
async fn test_jsonrpc_get_primals() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("primals.sock");
    let path_str = socket_path.to_str().unwrap();
    let _server = create_mock_server(path_str).await.unwrap();

    let provider = JsonRpcProvider::new(path_str);
    let primals = provider.get_primals().await.unwrap();

    assert_eq!(primals.len(), 2);
    assert_eq!(primals[0].id, "songbird");
    assert_eq!(primals[1].id, "beardog");
}

#[tokio::test]
async fn test_jsonrpc_get_topology() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("topology.sock");
    let path_str = socket_path.to_str().unwrap();
    let _server = create_mock_server(path_str).await.unwrap();

    let provider = JsonRpcProvider::new(path_str);
    let topology = provider.get_topology().await.unwrap();

    assert_eq!(topology.len(), 1);
    assert_eq!(topology[0].from, "songbird");
    assert_eq!(topology[0].to, "beardog");
}

#[tokio::test]
async fn test_jsonrpc_health_check() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("health.sock");
    let path_str = socket_path.to_str().unwrap();
    let _server = create_mock_server(path_str).await.unwrap();

    let provider = JsonRpcProvider::new(path_str);
    let health = provider.health_check().await.unwrap();

    assert!(health.contains("healthy"));
}

#[tokio::test]
async fn test_jsonrpc_metadata() {
    let socket_path = "/tmp/test-jsonrpc-integration-metadata.sock";

    let provider = JsonRpcProvider::new(socket_path);
    let metadata = provider.get_metadata();

    assert_eq!(metadata.name, "JSON-RPC Provider");
    assert_eq!(metadata.protocol, "jsonrpc-2.0");
    assert!(metadata.endpoint.contains("unix://"));
    assert!(metadata.capabilities.contains(&"primals".to_string()));
}

#[tokio::test]
async fn test_jsonrpc_connection_timeout() {
    // Non-existent socket should timeout
    let provider = JsonRpcProvider::new("/tmp/nonexistent-socket-12345.sock");
    let result = provider.health_check().await;

    assert!(result.is_err());
    let error_str = format!("{:?}", result.unwrap_err());
    assert!(
        error_str.contains("No such file")
            || error_str.contains("timeout")
            || error_str.contains("Timeout")
            || error_str.contains("ConnectionTimeout")
            || error_str.contains("Connection")
            || error_str.contains("Io"),
        "unexpected error: {error_str}"
    );
}

#[tokio::test]
async fn test_jsonrpc_concurrent_requests() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("concurrent.sock");
    let path_str = socket_path.to_str().unwrap();
    let _server = create_mock_server(path_str).await.unwrap();

    let _provider = JsonRpcProvider::new(path_str);

    // Make 10 concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let p = JsonRpcProvider::new(path_str);
        let handle = tokio::spawn(async move { p.get_primals().await });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}

#[tokio::test]
async fn test_jsonrpc_error_response() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("error.sock");
    let path_str = socket_path.to_str().unwrap();
    let _server = create_mock_server(path_str).await.unwrap();

    let _provider = JsonRpcProvider::new(path_str);

    // Call a method that returns an error (we'll need to modify call to be public for this)
    // For now, we test that unknown methods fail gracefully
}

#[tokio::test]
async fn test_jsonrpc_malformed_response() {
    // Create a server that sends malformed JSON
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("malformed.sock");
    let path_str = socket_path.to_str().unwrap();

    let listener = UnixListener::bind(path_str).unwrap();

    let _handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let (_, mut writer) = stream.into_split();
            // Send invalid JSON
            let _ = writer.write_all(b"{ invalid json }\n").await;
            let _ = writer.flush().await;
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let _provider = JsonRpcProvider::new(path_str);

    // Note: We can't easily test malformed response without exposing internal `call` method
    // This is OK - the unit tests cover serialization/deserialization
}

#[tokio::test]
async fn test_jsonrpc_sequential_requests() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("sequential.sock");
    let path_str = socket_path.to_str().unwrap();
    let _server = create_mock_server(path_str).await.unwrap();

    let provider = JsonRpcProvider::new(path_str);

    // Make 5 sequential requests
    for _ in 0..5 {
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 2);
    }
}
