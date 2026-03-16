// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for IpcServer.

use petal_tongue_core::{Instance, InstanceId};
use petal_tongue_ipc::server::{IpcServer, IpcTransport};
use petal_tongue_ipc::{IpcCommand, IpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn test_server_malformed_json_connection() {
    let instance_id = InstanceId::new();
    let instance = Instance::new(instance_id.clone(), Some("malformed-json-test".to_string()))
        .expect("instance");
    let server = IpcServer::start(&instance).await.expect("server");

    let transport = server.transport();
    let socket_path = match transport {
        IpcTransport::Unix(p) => p.clone(),
        IpcTransport::Tcp(_) => return,
    };

    let client_handle = tokio::spawn(async move {
        let (reader, mut writer) = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect")
            .into_split();

        writer.write_all(b"{invalid json}\n").await.expect("write");
        writer.flush().await.expect("flush");

        let mut buf = String::new();
        let _ = AsyncBufReadExt::read_line(&mut BufReader::new(reader), &mut buf).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let _ = client_handle.await;
}

#[tokio::test]
async fn test_server_request_routing_via_tcp() {
    let instance_id = InstanceId::new();
    let instance =
        Instance::new(instance_id.clone(), Some("test-tcp".to_string())).expect("instance");
    let mut server = IpcServer::start(&instance).await.expect("server");

    let addr = match server.transport() {
        IpcTransport::Tcp(a) => *a,
        IpcTransport::Unix(_) => return,
    };

    let client_handle = tokio::spawn(async move {
        let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
        let (reader, mut writer) = stream.into_split();

        let cmd = IpcCommand::Ping;
        let cmd_json = serde_json::to_string(&cmd).expect("serialize");
        writer
            .write_all(format!("{cmd_json}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");

        let mut line = String::new();
        AsyncBufReadExt::read_line(&mut BufReader::new(reader), &mut line)
            .await
            .expect("read");
        line
    });

    let (cmd, response_tx) = server.recv_command().await.expect("recv command");
    assert!(matches!(cmd, IpcCommand::Ping));
    response_tx.send(IpcResponse::Pong).expect("send response");

    let line = client_handle.await.expect("client");
    let response: IpcResponse = serde_json::from_str(line.trim()).expect("parse");
    assert!(matches!(response, IpcResponse::Pong));
}

#[tokio::test]
async fn test_server_drop_removes_socket_file() {
    let instance_id = InstanceId::new();
    let instance =
        Instance::new(instance_id, Some("drop-cleanup-test".to_string())).expect("instance");
    let server = IpcServer::start(&instance).await.expect("server");

    let socket_path = match server.transport() {
        IpcTransport::Unix(p) => p.clone(),
        IpcTransport::Tcp(_) => return,
    };
    assert!(
        socket_path.exists(),
        "socket should exist while server runs"
    );

    drop(server);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(!socket_path.exists(), "socket should be removed on drop");
}
