// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Chaos and fault injection tests for petal-tongue-ipc.

use petal_tongue_core::graph_engine::GraphEngine;
use petal_tongue_core::test_fixtures::env_test_helpers;
use petal_tongue_ipc::json_rpc::error_codes;
use petal_tongue_ipc::unix_socket_connection::handle_connection;
use petal_tongue_ipc::unix_socket_rpc_handlers::RpcHandlers;
use petal_tongue_ipc::validate_insecure_guard;
use petal_tongue_ipc::visualization_handler::VisualizationState;
use petal_tongue_ipc::{BtspGuardError, JsonRpcResponse};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

fn spawn_line_server() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("chaos.sock");
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let viz = Arc::new(RwLock::new(VisualizationState::new()));
    let root = Arc::new(RpcHandlers::new(
        graph,
        "chaos-test".to_string(),
        Arc::clone(&viz),
    ));
    let accept_loop = Arc::clone(&root);
    let listener = UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let hh = Arc::clone(&accept_loop);
                tokio::spawn(async move {
                    let _ = handle_connection(hh.as_ref(), stream).await;
                });
            }
        }
    });
    (dir, sock)
}

async fn read_one_json(sock: &std::path::Path, line_including_nl: &[u8]) -> JsonRpcResponse {
    let mut stream = UnixStream::connect(sock).await.expect("connect");
    stream
        .write_all(line_including_nl)
        .await
        .expect("write request");
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .await
        .expect("read response");
    serde_json::from_str(line.trim()).expect("parse JsonRpcResponse")
}

#[tokio::test]
async fn malformed_json_rpc_messages() {
    let (_d, sock) = spawn_line_server();
    let parse_failures: [&[u8]; 3] = [
        b"{\"jsonrpc\":\"2.0\",\"id\":1}\n",
        b"{\"jsonrpc\":\"2.0\",\"method\":[],\"params\":{},\"id\":1}\n",
        b"{not json at all}\n",
    ];
    for bytes in parse_failures {
        let resp = read_one_json(&sock, bytes).await;
        let err = resp.error.expect("parse error response");
        assert_eq!(err.code, error_codes::PARSE_ERROR);
    }
    let pad = "x".repeat(96 * 1024);
    let huge = format!(
        "{{\"jsonrpc\":\"2.0\",\"method\":\"health.get\",\"params\":{{\"pad\":\"{pad}\"}},\"id\":1}}\n"
    );
    let big = read_one_json(&sock, huge.as_bytes()).await;
    assert!(
        big.error.is_none(),
        "valid but oversized line should still dispatch"
    );
}

#[tokio::test]
async fn concurrent_clients_share_handler() {
    let (_d, sock) = spawn_line_server();
    let n = 48u32;
    let mut tasks = Vec::with_capacity(n as usize);
    for i in 0..n {
        let p = sock.clone();
        tasks.push(tokio::spawn(async move {
            let req = format!(
                "{{\"jsonrpc\":\"2.0\",\"method\":\"health.get\",\"params\":{{}},\"id\":{i}}}\n"
            );
            read_one_json(&p, req.as_bytes()).await
        }));
    }
    for t in tasks {
        let resp = t.await.expect("join");
        assert!(resp.error.is_none(), "health.get should succeed");
        assert!(resp.result.is_some());
    }
}

#[tokio::test]
async fn connection_drop_mid_line_without_newline() {
    let (_d, sock) = spawn_line_server();
    let mut stream = UnixStream::connect(&sock).await.expect("connect");
    stream
        .write_all(br#"{"jsonrpc":"2.0","method":"health.get","par"#)
        .await
        .expect("partial write");
    drop(stream);
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
}

#[tokio::test]
async fn invalid_method_negative_id_null_params() {
    let (_d, sock) = spawn_line_server();
    let bad = read_one_json(
        &sock,
        br#"{"jsonrpc":"2.0","method":"totally.unknown.method","params":{},"id":-99}
"#,
    )
    .await;
    let e = bad.error.expect("method error");
    assert_eq!(e.code, error_codes::METHOD_NOT_FOUND);
    assert_eq!(bad.id, serde_json::json!(-99));

    let ok = read_one_json(
        &sock,
        br#"{"jsonrpc":"2.0","method":"health.get","params":null,"id":-1}
"#,
    )
    .await;
    assert!(ok.error.is_none());
    assert_eq!(ok.id, serde_json::json!(-1));
}

#[test]
fn btsp_guard_conflicting_env_vars() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("prod-family")),
            ("BIOMEOS_INSECURE", Some("1")),
        ],
        || match validate_insecure_guard() {
            Err(BtspGuardError::ConflictingPosture { family_id }) => {
                assert_eq!(family_id, "prod-family");
            }
            other => panic!("expected conflicting posture error, got {other:?}"),
        },
    );
}

#[tokio::test]
async fn rapid_connect_disconnect_cycles() {
    let (_d, sock) = spawn_line_server();
    for _ in 0..80 {
        if let Ok(s) = UnixStream::connect(&sock).await {
            drop(s);
        }
    }
}
