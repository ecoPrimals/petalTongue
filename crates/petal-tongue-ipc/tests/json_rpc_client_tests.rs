// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for JsonRpcClient semantic methods and fallback paths.

use petal_tongue_ipc::JsonRpcClient;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn mock_server_responds(
    method_responds: &[(&str, &str)],
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("mock.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    let method_responds: Vec<(String, String)> = method_responds
        .iter()
        .map(|(m, r)| ((*m).to_string(), (*r).to_string()))
        .collect();
    tokio::spawn(async move {
        for expected in method_responds {
            let (stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = stream.into_split();
            let mut line = String::new();
            AsyncBufReadExt::read_line(&mut BufReader::new(reader), &mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(&line).expect("parse");
            let method = req["method"].as_str().unwrap_or("");
            let id = &req["id"];
            let resp = if method == expected.0 {
                expected.1.replace("\"id\":1", &format!("\"id\":{id}"))
            } else {
                r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#
                    .replace("\"id\":1", &format!("\"id\":{id}"))
            };
            writer
                .write_all(format!("{resp}\n").as_bytes())
                .await
                .expect("write");
            writer.flush().await.expect("flush");
        }
    });
    (dir, sock)
}

#[tokio::test]
async fn test_discover_primals_primary_path() {
    let (_dir, sock) = mock_server_responds(&[(
        "discovery.primals",
        r#"{"jsonrpc":"2.0","result":[{"id":"p1","name":"test","primal_type":"test","endpoint":"/","capabilities":[],"health":"Healthy","last_seen":0}],"id":1}"#,
    )]);
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let primals = client.discover_primals().await.expect("discover");
    assert_eq!(primals.len(), 1);
}

#[tokio::test]
async fn test_discover_primals_fallback_to_primal_list() {
    let (_dir, sock) = mock_server_responds(&[
        (
            "discovery.primals",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "primal.list",
            r#"{"jsonrpc":"2.0","result":[{"id":"p1","name":"test","primal_type":"test","endpoint":"/","capabilities":[],"health":"Healthy","last_seen":0}],"id":1}"#,
        ),
    ]);
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let primals = client.discover_primals().await.expect("discover");
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id.as_str(), "p1");
}

#[tokio::test]
async fn test_get_topology_primary_path() {
    let (_dir, sock) = mock_server_responds(&[(
        "topology.get",
        r#"{"jsonrpc":"2.0","result":{"nodes":[{"id":"n1"}],"edges":[]},"id":1}"#,
    )]);
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let topo = client.get_topology().await.expect("topology");
    assert_eq!(topo.nodes.len(), 1);
}

#[tokio::test]
async fn test_get_topology_fallback_chain() {
    let (_dir, sock) = mock_server_responds(&[
        (
            "topology.get",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "get_topology",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "neural_api.get_topology",
            r#"{"jsonrpc":"2.0","result":{"nodes":[],"edges":[]},"id":1}"#,
        ),
    ]);
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let topo = client.get_topology().await.expect("topology");
    assert!(topo.nodes.is_empty());
    assert!(topo.edges.is_empty());
}

#[tokio::test]
async fn test_health_check_fallback_to_health_get() {
    let (_dir, sock) = mock_server_responds(&[
        (
            "health.check",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "health_check",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "health.get",
            r#"{"jsonrpc":"2.0","result":{"status":"ok"},"id":1}"#,
        ),
    ]);
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let health = client.health_check().await.expect("health");
    assert_eq!(health["status"], "ok");
}

#[tokio::test]
async fn test_get_capabilities_fallback_to_announce_capabilities() {
    let (_dir, sock) = mock_server_responds(&[
        (
            "capability.list",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "capability.announce",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "get_capabilities",
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#,
        ),
        (
            "announce_capabilities",
            r#"{"jsonrpc":"2.0","result":["ui","graph"],"id":1}"#,
        ),
    ]);
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let caps = client.get_capabilities().await.expect("capabilities");
    assert!(caps.is_array());
    assert_eq!(caps.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_notify_success_via_mock_server() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("notify-success.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut reader, _writer) = stream.into_split();
        let mut buf = [0u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
    });
    let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).unwrap();
    let result = client.notify("test.notify", serde_json::json!({})).await;
    assert!(result.is_ok());
}
