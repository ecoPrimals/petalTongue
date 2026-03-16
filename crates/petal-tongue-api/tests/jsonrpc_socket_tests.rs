// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! JSON-RPC over Unix socket integration tests.
//!
//! Uses a mock Unix socket server to exercise BiomeOSJsonRpcClient paths.

use petal_tongue_api::BiomeOSJsonRpcClient;
use petal_tongue_core::PrimalHealthStatus;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

/// Spawn a mock JSON-RPC server that responds with the given result value.
/// Returns (socket_path, _temp_dir) - keep temp_dir alive for socket to exist.
fn spawn_mock_jsonrpc_server_with_result(
    result: serde_json::Value,
) -> (std::path::PathBuf, TempDir) {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("biomeos.sock");

    let listener = UnixListener::bind(&socket_path).expect("bind");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let _ = reader.read_line(&mut line).await;
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse request");
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        });
        let _ = writer
            .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
            .await;
        let _ = writer.flush().await;
    });

    (socket_path, temp)
}

/// Spawn a mock server that returns a JSON-RPC error response.
fn spawn_mock_jsonrpc_server_with_error(error: serde_json::Value) -> (std::path::PathBuf, TempDir) {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("biomeos.sock");

    let listener = UnixListener::bind(&socket_path).expect("bind");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let _ = reader.read_line(&mut line).await;
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse request");
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": error,
            "id": id
        });
        let _ = writer
            .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
            .await;
        let _ = writer.flush().await;
    });

    (socket_path, temp)
}

/// Spawn a mock server that returns a response without a result field.
fn spawn_mock_jsonrpc_server_no_result() -> (std::path::PathBuf, TempDir) {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("biomeos.sock");

    let listener = UnixListener::bind(&socket_path).expect("bind");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let _ = reader.read_line(&mut line).await;
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse request");
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id
        });
        let _ = writer
            .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
            .await;
        let _ = writer.flush().await;
    });

    (socket_path, temp)
}

#[tokio::test]
async fn test_jsonrpc_discover_primals_success() {
    let result = serde_json::json!([
        {
            "id": "p1",
            "name": "Test Primal",
            "primal_type": "Compute",
            "endpoint": "http://localhost:8000",
            "capabilities": ["compute"],
            "health": "Healthy",
            "last_seen": 12345
        }
    ]);
    let (socket_path, _temp) = spawn_mock_jsonrpc_server_with_result(result);

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let primals = client.discover_primals().await.expect("discover_primals");
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id.as_str(), "p1");
    assert_eq!(primals[0].health, PrimalHealthStatus::Healthy);
}

#[tokio::test]
async fn test_jsonrpc_get_topology_success() {
    let result = serde_json::json!([
        {"from": "a", "to": "b", "edge_type": "conn"}
    ]);
    let (socket_path, _temp) = spawn_mock_jsonrpc_server_with_result(result);

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let edges = client.get_topology().await.expect("get_topology");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from.as_str(), "a");
    assert_eq!(edges[0].to.as_str(), "b");
}

#[tokio::test]
async fn test_jsonrpc_health_check_success() {
    let (socket_path, _temp) = spawn_mock_jsonrpc_server_with_result(serde_json::json!({}));

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let healthy = client.health_check().await.expect("health_check");
    assert!(healthy);
}

#[tokio::test]
async fn test_jsonrpc_is_available_when_server_running() {
    let (socket_path, _temp) = spawn_mock_jsonrpc_server_with_result(serde_json::json!({}));

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let available = client.is_available().await;
    assert!(available);
}

#[tokio::test]
async fn test_jsonrpc_discover_primals_parse_error() {
    let (socket_path, _temp) =
        spawn_mock_jsonrpc_server_with_result(serde_json::json!({"invalid": "structure"}));

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let err = client.discover_primals().await.expect_err("should fail");
    assert!(matches!(
        err,
        petal_tongue_api::BiomeOsClientError::Parse(_)
    ));
}

#[tokio::test]
async fn test_jsonrpc_get_topology_parse_error() {
    let (socket_path, _temp) =
        spawn_mock_jsonrpc_server_with_result(serde_json::json!("not an array"));

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let err = client.get_topology().await.expect_err("should fail");
    assert!(matches!(
        err,
        petal_tongue_api::BiomeOsClientError::Parse(_)
    ));
}

#[tokio::test]
async fn test_jsonrpc_error_response() {
    let error = serde_json::json!({"code": -32603, "message": "Internal error"});
    let (socket_path, _temp) = spawn_mock_jsonrpc_server_with_error(error);

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let err = client.discover_primals().await.expect_err("should fail");
    assert!(matches!(
        err,
        petal_tongue_api::BiomeOsClientError::JsonRpcError(_)
    ));
}

#[tokio::test]
async fn test_jsonrpc_no_result() {
    let (socket_path, _temp) = spawn_mock_jsonrpc_server_no_result();

    let client = BiomeOSJsonRpcClient::with_socket_path(&socket_path);
    let err = client.discover_primals().await.expect_err("should fail");
    assert!(matches!(
        err,
        petal_tongue_api::BiomeOsClientError::NoResult
    ));
}
