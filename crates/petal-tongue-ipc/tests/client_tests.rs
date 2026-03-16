// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for IpcClient.

use petal_tongue_core::InstanceId;
use petal_tongue_ipc::client::{IpcClient, IpcClientError};
use petal_tongue_ipc::{InstanceStatus, IpcResponse};
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_ping_server_error_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ping-error.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (reader, mut writer) = stream.into_split();
        let mut line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::BufReader::new(reader), &mut line)
            .await
            .expect("read");
        let err_resp =
            serde_json::to_string(&IpcResponse::error("server error")).expect("serialize");
        writer
            .write_all(format!("{err_resp}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");
    });
    let client = IpcClient::from_socket_path(sock);
    let result = client.ping().await;
    assert!(result.is_err());
    if let Err(IpcClientError::ServerError(msg)) = result {
        assert_eq!(msg, "server error");
    } else {
        panic!("Expected ServerError");
    }
}

#[tokio::test]
async fn test_ping_unexpected_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ping-unexpected.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (reader, mut writer) = stream.into_split();
        let mut line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::BufReader::new(reader), &mut line)
            .await
            .expect("read");
        let status_resp = serde_json::to_string(&IpcResponse::Status(InstanceStatus {
            instance_id: InstanceId::new(),
            pid: 0,
            window_id: None,
            name: None,
            uptime_seconds: 0,
            node_count: 0,
            edge_count: 0,
            window_visible: true,
            metadata: std::collections::HashMap::new(),
        }))
        .expect("serialize");
        writer
            .write_all(format!("{status_resp}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");
    });
    let client = IpcClient::from_socket_path(sock);
    let result = client.ping().await;
    assert!(result.is_err());
    assert!(matches!(result, Err(IpcClientError::UnexpectedResponse)));
}
