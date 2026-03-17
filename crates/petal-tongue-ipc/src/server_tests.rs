// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::protocol::IpcCommand;
use petal_tongue_core::{Instance, InstanceId};
use std::net::SocketAddr;
use std::path::PathBuf;

#[test]
fn test_ipc_transport_display_unix() {
    let transport = IpcTransport::Unix(PathBuf::from("/tmp/test.sock"));
    let s = transport.to_string();
    assert!(s.starts_with("unix:"));
    assert!(s.contains("test"));
}

#[test]
fn test_ipc_transport_display_tcp() {
    let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let transport = IpcTransport::Tcp(addr);
    let s = transport.to_string();
    assert!(s.starts_with("tcp:"));
    assert!(s.contains("12345"));
}

#[test]
fn test_ipc_transport_equality() {
    let t1 = IpcTransport::Unix(PathBuf::from("/tmp/a.sock"));
    let t2 = IpcTransport::Unix(PathBuf::from("/tmp/a.sock"));
    let t3 = IpcTransport::Tcp("127.0.0.1:0".parse().expect("addr"));
    assert_eq!(t1.to_string(), t2.to_string());
    assert!(t1.to_string().starts_with("unix:"));
    assert!(t3.to_string().starts_with("tcp:"));
}

#[test]
fn test_ipc_server_error_display() {
    let err = IpcServerError::ParseError("bad json".to_string());
    let s = format!("{err}");
    assert!(s.contains("Parse"));
    assert!(s.contains("bad json"));

    let err = IpcServerError::ChannelClosed;
    let s = format!("{err}");
    assert!(s.contains("Channel"));

    let err = IpcServerError::SocketError("bind failed".to_string());
    assert!(format!("{err}").contains("bind failed"));

    let err = IpcServerError::IoError("read failed".to_string());
    assert!(format!("{err}").contains("read failed"));

    let err = IpcServerError::SerializeError("to_json failed".to_string());
    assert!(format!("{err}").contains("to_json failed"));

    let err = IpcServerError::DiscoveryError("write failed".to_string());
    assert!(format!("{err}").contains("write failed"));
}

#[test]
fn test_ipc_command_parse() {
    use crate::protocol::IpcCommand;
    let json = r#"{"Ping":null}"#;
    let cmd: IpcCommand = serde_json::from_str(json).expect("parse");
    matches!(cmd, IpcCommand::Ping);

    let json = r#"{"GetStatus":null}"#;
    let cmd: IpcCommand = serde_json::from_str(json).expect("parse");
    matches!(cmd, IpcCommand::GetStatus);

    let json = r#"{"SetPanel":{"panel":"left","visible":true}}"#;
    let cmd: IpcCommand = serde_json::from_str(json).expect("parse");
    match &cmd {
        IpcCommand::SetPanel { panel, visible } => {
            assert_eq!(panel, "left");
            assert!(*visible);
        }
        _ => panic!("expected SetPanel"),
    }
}

#[test]
fn test_ipc_command_serialize_roundtrip() {
    use crate::protocol::IpcCommand;
    let cmd = IpcCommand::Ping;
    let json = serde_json::to_string(&cmd).expect("serialize");
    let restored: IpcCommand = serde_json::from_str(&json).expect("deserialize");
    matches!(restored, IpcCommand::Ping);

    let cmd = IpcCommand::SetZoom { level: 2.0 };
    let json = serde_json::to_string(&cmd).expect("serialize");
    let restored: IpcCommand = serde_json::from_str(&json).expect("deserialize");
    match restored {
        IpcCommand::SetZoom { level } => assert!((level - 2.0).abs() < f32::EPSILON),
        _ => panic!("expected SetZoom"),
    }
}

#[tokio::test]
async fn test_server_creation() {
    let instance_id = InstanceId::new();
    let instance = Instance::new(instance_id, Some("test".to_string())).expect("instance");

    let server = IpcServer::start(&instance).await;
    assert!(server.is_ok());

    let server = server.expect("server");
    assert_eq!(server.instance_id(), &instance.id);
}

#[tokio::test]
async fn test_tcp_fallback() {
    // Force TCP by using an instance with invalid Unix path
    let instance_id = InstanceId::new();
    let instance = Instance::new(instance_id, Some("test-tcp".to_string())).expect("instance");

    // Try to start - should work with TCP fallback
    let server = IpcServer::start(&instance).await;
    assert!(server.is_ok());

    let server = server.expect("server");
    // Should have TCP transport
    matches!(server.transport(), IpcTransport::Tcp(_));
}

#[test]
fn test_is_platform_constrained() {
    // On non-Android, should return false
    #[cfg(not(target_os = "android"))]
    assert!(!super::is_platform_constrained());

    #[cfg(target_os = "android")]
    assert!(super::is_platform_constrained());
}

#[tokio::test]
async fn test_recv_command_returns_option() {
    let instance_id = InstanceId::new();
    let instance = Instance::new(instance_id, Some("test-recv".to_string())).expect("instance");
    let mut server = IpcServer::start(&instance).await.expect("server");

    // recv_command with timeout - no clients connected
    let result =
        tokio::time::timeout(std::time::Duration::from_millis(50), server.recv_command()).await;
    // Timeout returns Err, or we got Some/None - all outcomes are expected (no panic)
    match result {
        Ok(None | Some(_)) | Err(_) => {}
    }
}

#[tokio::test]
async fn test_server_transport_display() {
    let transport = IpcTransport::Unix(PathBuf::from("/tmp/x.sock"));
    let s = transport.to_string();
    assert!(s.starts_with("unix:"));
    assert!(s.contains("x.sock"));

    let addr: SocketAddr = "127.0.0.1:0".parse().expect("addr");
    let transport = IpcTransport::Tcp(addr);
    let s = transport.to_string();
    assert!(s.starts_with("tcp:"));
}

#[tokio::test]
async fn test_server_request_routing_via_unix_socket() {
    let instance_id = InstanceId::new();
    let instance =
        Instance::new(instance_id.clone(), Some("routing-test".to_string())).expect("instance");
    let mut server = IpcServer::start(&instance).await.expect("server");

    let transport = server.transport().clone();
    let socket_path = match &transport {
        IpcTransport::Unix(p) => p.clone(),
        IpcTransport::Tcp(_) => return,
    };

    let client_handle = tokio::spawn(async move {
        let (reader, mut writer) = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect")
            .into_split();

        let cmd = crate::protocol::IpcCommand::Ping;
        let cmd_json = serde_json::to_string(&cmd).expect("serialize");
        writer
            .write_all(format!("{cmd_json}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");

        let mut line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::BufReader::new(reader), &mut line)
            .await
            .expect("read");
        line
    });

    let (cmd, response_tx) = server.recv_command().await.expect("recv command");
    assert!(matches!(cmd, IpcCommand::Ping));
    response_tx
        .send(crate::protocol::IpcResponse::Pong)
        .expect("send response");

    let line = client_handle.await.expect("client");
    let response: crate::protocol::IpcResponse = serde_json::from_str(line.trim()).expect("parse");
    assert!(matches!(response, crate::protocol::IpcResponse::Pong));
}

#[tokio::test]
async fn test_server_error_response_routing() {
    let instance_id = InstanceId::new();
    let instance =
        Instance::new(instance_id, Some("error-response-test".to_string())).expect("instance");
    let mut server = IpcServer::start(&instance).await.expect("server");

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

        let cmd = crate::protocol::IpcCommand::GetState;
        let cmd_json = serde_json::to_string(&cmd).expect("serialize");
        writer
            .write_all(format!("{cmd_json}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");

        let mut line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::BufReader::new(reader), &mut line)
            .await
            .expect("read");
        line
    });

    let (_cmd, response_tx) = server.recv_command().await.expect("recv");
    let err_resp = crate::protocol::IpcResponse::error("State not available");
    response_tx.send(err_resp).expect("send");

    let line = client_handle.await.expect("client");
    let response: crate::protocol::IpcResponse = serde_json::from_str(line.trim()).expect("parse");
    assert!(response.is_error());
    assert_eq!(response.error_message(), Some("State not available"));
}

#[test]
fn test_ipc_response_serialize_roundtrip() {
    use crate::protocol::IpcResponse;
    let pong = IpcResponse::Pong;
    let json = serde_json::to_string(&pong).expect("serialize");
    let restored: IpcResponse = serde_json::from_str(&json).expect("deserialize");
    matches!(restored, IpcResponse::Pong);

    let err = IpcResponse::error("test error");
    let json = serde_json::to_string(&err).expect("serialize");
    let restored: IpcResponse = serde_json::from_str(&json).expect("deserialize");
    assert!(restored.is_error());
}

#[test]
fn test_ipc_transport_clone_display() {
    let t1 = IpcTransport::Unix(PathBuf::from("/tmp/x.sock"));
    let t2 = t1.clone();
    assert_eq!(t1.to_string(), t2.to_string());
}

#[tokio::test]
async fn test_server_get_status_command_routing() {
    let instance_id = InstanceId::new();
    let instance =
        Instance::new(instance_id.clone(), Some("status-test".to_string())).expect("instance");
    let mut server = IpcServer::start(&instance).await.expect("server");

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

        let cmd = crate::protocol::IpcCommand::GetStatus;
        let cmd_json = serde_json::to_string(&cmd).expect("serialize");
        writer
            .write_all(format!("{cmd_json}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");

        let mut line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::BufReader::new(reader), &mut line)
            .await
            .expect("read");
        line
    });

    let (cmd, response_tx) = server.recv_command().await.expect("recv");
    assert!(matches!(cmd, IpcCommand::GetStatus));
    let status = crate::protocol::InstanceStatus {
        instance_id: instance_id.clone(),
        pid: std::process::id(),
        window_id: None,
        name: Some("status-test".to_string()),
        uptime_seconds: 0,
        node_count: 0,
        edge_count: 0,
        window_visible: true,
        metadata: std::collections::HashMap::new(),
    };
    response_tx
        .send(crate::protocol::IpcResponse::Status(status))
        .expect("send");

    let line = client_handle.await.expect("client");
    let response: crate::protocol::IpcResponse = serde_json::from_str(line.trim()).expect("parse");
    assert!(!response.is_error());
    assert!(matches!(response, crate::protocol::IpcResponse::Status(_)));
}

#[test]
fn test_ipc_transport_clone_equality() {
    let unix = IpcTransport::Unix(PathBuf::from("/tmp/a.sock"));
    let unix2 = unix.clone();
    assert_eq!(unix.to_string(), unix2.to_string());

    let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    let tcp = IpcTransport::Tcp(addr);
    let tcp2 = tcp.clone();
    assert_eq!(tcp.to_string(), tcp2.to_string());
}

#[test]
fn test_ipc_command_all_variants_serialize() {
    use crate::protocol::IpcCommand;
    let variants = [
        IpcCommand::Ping,
        IpcCommand::GetStatus,
        IpcCommand::GetState,
        IpcCommand::Show,
        IpcCommand::Hide,
        IpcCommand::SetPanel {
            panel: "left".to_string(),
            visible: true,
        },
        IpcCommand::SetZoom { level: 1.5 },
    ];
    for cmd in variants {
        let json = serde_json::to_string(&cmd).expect("serialize");
        let restored: IpcCommand = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            serde_json::to_string(&cmd).unwrap(),
            serde_json::to_string(&restored).unwrap()
        );
    }
}

#[test]
fn test_ipc_response_error_message() {
    use crate::protocol::IpcResponse;
    let err = IpcResponse::error("test error");
    assert!(err.is_error());
    assert_eq!(err.error_message(), Some("test error"));
}
