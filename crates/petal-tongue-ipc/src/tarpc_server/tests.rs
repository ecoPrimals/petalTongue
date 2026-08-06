// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tests for the tarpc UDS server.

use super::*;
use crate::tarpc_types::PetalTongueRpcClient;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn tarpc_server_binds_and_serves_capabilities() {
    let tmp = TempDir::new().unwrap();
    let sock = tmp.path().join("test.tarpc.sock");

    let server = TarpcServer::new(sock.clone());
    let server_handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock).await.unwrap();
    let codec = tokio_util::codec::LengthDelimitedCodec::builder()
        .max_frame_length(16 * 1024 * 1024)
        .new_framed(stream);
    let transport = tarpc::serde_transport::new(codec, tokio_serde::formats::Bincode::default());
    let client = PetalTongueRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let caps = client
        .capabilities_list(tarpc::context::current())
        .await
        .unwrap();
    assert!(!caps.is_empty());
    assert!(caps.iter().any(|c| c == "ui.render"));

    server_handle.abort();
}

#[tokio::test]
async fn tarpc_server_health_check() {
    let tmp = TempDir::new().unwrap();
    let sock = tmp.path().join("health.tarpc.sock");

    let server = TarpcServer::new(sock.clone());
    let server_handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock).await.unwrap();
    let codec = tokio_util::codec::LengthDelimitedCodec::builder()
        .max_frame_length(16 * 1024 * 1024)
        .new_framed(stream);
    let transport = tarpc::serde_transport::new(codec, tokio_serde::formats::Bincode::default());
    let client = PetalTongueRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let health = client
        .health_check(tarpc::context::current())
        .await
        .unwrap();
    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());

    server_handle.abort();
}

#[tokio::test]
async fn tarpc_server_version_reports_037() {
    let tmp = TempDir::new().unwrap();
    let sock = tmp.path().join("version.tarpc.sock");

    let server = TarpcServer::new(sock.clone());
    let server_handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock).await.unwrap();
    let codec = tokio_util::codec::LengthDelimitedCodec::builder()
        .max_frame_length(16 * 1024 * 1024)
        .new_framed(stream);
    let transport = tarpc::serde_transport::new(codec, tokio_serde::formats::Bincode::default());
    let client = PetalTongueRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let version = client
        .version_get(tarpc::context::current())
        .await
        .unwrap();
    assert_eq!(version.tarpc_version, "0.37");
    assert_eq!(version.jsonrpc_version, "2.0");

    server_handle.abort();
}

#[tokio::test]
async fn tarpc_server_protocols_list_dual_socket() {
    let tmp = TempDir::new().unwrap();
    let sock = tmp.path().join("proto.tarpc.sock");

    let server = TarpcServer::new(sock.clone());
    let server_handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock).await.unwrap();
    let codec = tokio_util::codec::LengthDelimitedCodec::builder()
        .max_frame_length(16 * 1024 * 1024)
        .new_framed(stream);
    let transport = tarpc::serde_transport::new(codec, tokio_serde::formats::Bincode::default());
    let client = PetalTongueRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let protos = client
        .protocols_list(tarpc::context::current())
        .await
        .unwrap();
    assert_eq!(protos.len(), 2);
    assert_eq!(protos[0].name, "tarpc");
    assert_eq!(protos[0].priority, 1);
    assert_eq!(protos[1].name, "jsonrpc");
    assert_eq!(protos[1].priority, 2);

    server_handle.abort();
}

#[test]
fn tarpc_socket_path_resolution() {
    let path = PathBuf::from("/tmp/biomeos/petaltongue.tarpc.sock");
    let server = TarpcServer::new(path.clone());
    assert_eq!(server.socket_path(), path);
}
