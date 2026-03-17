// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Cross-primal IPC end-to-end tests.
//!
//! Exercises the Unix socket JSON-RPC protocol between petalTongue's server
//! and client using the real UnixSocketServer and JsonRpcClient.

use petal_tongue_core::graph_engine::GraphEngine;
use petal_tongue_core::test_fixtures::env_test_helpers;
use petal_tongue_ipc::{JsonRpcClient, JsonRpcClientError, UnixSocketServer};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

/// Spawn the real UnixSocketServer on a tempdir socket and return (tempdir, socket_path, server_handle).
/// The server runs until the returned JoinHandle is dropped/aborted.
fn spawn_real_server() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    tokio::task::JoinHandle<Result<(), petal_tongue_ipc::server::IpcServerError>>,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("e2e.sock");
    let sock_str = sock.to_str().expect("utf8 path").to_string();

    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let server = env_test_helpers::with_env_var("PETALTONGUE_SOCKET", &sock_str, || {
        UnixSocketServer::new(graph).expect("server")
    });

    let server = Arc::new(server);
    let server_clone = Arc::clone(&server);
    let handle = tokio::spawn(async move { server_clone.start().await });

    (dir, sock, handle)
}

#[tokio::test]
async fn test_health_check_e2e() {
    let (_dir, sock, _server_handle) = spawn_real_server();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");
    let result = client.call("health.check", serde_json::json!({})).await;
    assert!(result.is_ok(), "health.check should succeed: {result:?}");

    let health = result.expect("ok");
    assert_eq!(health["status"], "healthy");
    assert!(health["version"].is_string());
    assert!(health["modalities_active"].is_array());
}

#[tokio::test]
async fn test_topology_get_e2e() {
    let (_dir, sock, _server_handle) = spawn_real_server();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");
    let result = client.call("topology.get", serde_json::json!({})).await;
    assert!(result.is_ok(), "topology.get should succeed: {result:?}");

    let topo = result.expect("ok");
    assert!(topo["nodes"].is_array());
    assert!(topo["edges"].is_array());
}

#[tokio::test]
async fn test_capability_list_e2e() {
    let (_dir, sock, _server_handle) = spawn_real_server();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");
    let result = client.call("capability.list", serde_json::json!({})).await;
    assert!(result.is_ok(), "capability.list should succeed: {result:?}");

    let caps = result.expect("ok");
    assert!(caps["capabilities"].is_array());
    assert!(caps["family_id"].is_string());
}

#[tokio::test]
async fn test_unknown_method_returns_error() {
    let (_dir, sock, _server_handle) = spawn_real_server();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");
    let result = client
        .call("nonexistent.unknown_method", serde_json::json!({}))
        .await;
    assert!(result.is_err(), "unknown method should fail: {result:?}");

    let err = result.unwrap_err();
    if let JsonRpcClientError::RpcError { code, message, .. } = err {
        assert_eq!(code, -32601, "METHOD_NOT_FOUND");
        assert!(
            message.contains("Method not found") || message.contains("nonexistent.unknown_method"),
            "message should mention method not found: {message}"
        );
    } else {
        panic!("Expected RpcError, got: {err:?}");
    }
}

#[tokio::test]
async fn test_concurrent_clients() {
    let (_dir, sock, _server_handle) = spawn_real_server();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client1 = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");
    let client2 = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");

    let (r1, r2) = tokio::join!(
        client1.call("health.check", serde_json::json!({})),
        client2.call("topology.get", serde_json::json!({})),
    );

    assert!(r1.is_ok(), "client1 health.check: {r1:?}");
    assert!(r2.is_ok(), "client2 topology.get: {r2:?}");

    assert_eq!(r1.expect("ok")["status"], "healthy");
    assert!(r2.expect("ok")["nodes"].is_array());
}

#[tokio::test]
async fn test_server_cleanup_on_shutdown() {
    let (_dir, sock, server_handle) = spawn_real_server();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(sock.exists(), "socket should exist while server runs");

    let client = JsonRpcClient::with_timeout(&sock, Duration::from_secs(2)).expect("client");
    let result = client.call("health.check", serde_json::json!({})).await;
    assert!(result.is_ok(), "health.check before shutdown: {result:?}");

    server_handle.abort();
    let _ = server_handle.await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client_after =
        JsonRpcClient::with_timeout(&sock, Duration::from_millis(100)).expect("client");
    let result_after = client_after
        .call("health.check", serde_json::json!({}))
        .await;
    assert!(
        result_after.is_err(),
        "client should fail to connect after server shutdown: {result_after:?}"
    );
}
