// SPDX-License-Identifier: AGPL-3.0-or-later

use super::content_backend::{ContentBackendClient, ContentEndpoint, content_fallback};
use super::handlers::*;

use std::sync::Arc;

use axum::{Router, routing::get};

use crate::data_service::DataService;

#[tokio::test]
async fn test_content_backend_client_from_env_default() {
    let client = ContentBackendClient::from_env().await;
    let ep = format!("{}", client.endpoint);
    assert!(
        ep.contains(".sock") || ep.starts_with("tcp:"),
        "endpoint should be a socket or TCP address: {ep}"
    );
}

#[tokio::test]
async fn test_content_backend_client_env_override() {
    use petal_tongue_core::test_fixtures::env_test_helpers;
    let client = env_test_helpers::with_env_var_async(
        "CONTENT_BACKEND_SOCKET",
        "/custom/content.sock",
        || async { ContentBackendClient::from_env().await },
    )
    .await;
    match &client.endpoint {
        ContentEndpoint::Unix(p) => {
            assert_eq!(p, &std::path::PathBuf::from("/custom/content.sock"));
        }
        ContentEndpoint::Tcp(addr) => panic!("expected Unix endpoint, got tcp:{addr}"),
    }
}

#[tokio::test]
async fn test_content_backend_tcp_endpoint_override() {
    use petal_tongue_core::test_fixtures::env_test_helpers;
    let client = env_test_helpers::with_env_vars_async(
        &[
            ("CONTENT_BACKEND_ENDPOINT", Some("eastgate.mesh:9100")),
            ("CONTENT_BACKEND_SOCKET", None),
        ],
        || async { ContentBackendClient::from_env().await },
    )
    .await;
    match &client.endpoint {
        ContentEndpoint::Tcp(addr) => {
            assert_eq!(addr, "eastgate.mesh:9100");
        }
        ContentEndpoint::Unix(p) => panic!("expected TCP endpoint, got unix:{}", p.display()),
    }
}

#[test]
fn test_content_backend_request_id_increments() {
    let client = ContentBackendClient {
        endpoint: ContentEndpoint::Unix(std::path::PathBuf::from("/tmp/test.sock")),
        request_id: std::sync::atomic::AtomicU64::new(1),
    };
    let id1 = client.next_id();
    let id2 = client.next_id();
    assert_eq!(id2, id1 + 1);
}

#[tokio::test]
async fn test_content_backend_socket_beats_tcp_priority() {
    use petal_tongue_core::test_fixtures::env_test_helpers;
    let client = env_test_helpers::with_env_vars_async(
        &[
            ("CONTENT_BACKEND_SOCKET", Some("/explicit/content.sock")),
            ("CONTENT_BACKEND_ENDPOINT", Some("eastgate.mesh:9100")),
        ],
        || async { ContentBackendClient::from_env().await },
    )
    .await;
    match &client.endpoint {
        ContentEndpoint::Unix(p) => {
            assert_eq!(p, &std::path::PathBuf::from("/explicit/content.sock"));
        }
        ContentEndpoint::Tcp(addr) => {
            panic!("socket should beat TCP, but got tcp:{addr}");
        }
    }
}

#[tokio::test]
async fn test_content_backend_convention_socket_found() {
    use petal_tongue_core::test_fixtures::env_test_helpers;
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("content-provider-testfam.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&sock_path).unwrap();

    let client = env_test_helpers::with_env_vars_async(
        &[
            ("CONTENT_BACKEND_SOCKET", None),
            ("CONTENT_BACKEND_ENDPOINT", None),
            ("CONTENT_BACKEND_PROVIDER", Some("content-provider")),
            ("FAMILY_ID", Some("testfam")),
            ("BIOMEOS_SOCKET_DIR", Some(tmp.path().to_str().unwrap())),
        ],
        || async { ContentBackendClient::from_env().await },
    )
    .await;
    match &client.endpoint {
        ContentEndpoint::Unix(p) => {
            assert_eq!(p, &sock_path);
        }
        ContentEndpoint::Tcp(addr) => {
            panic!("expected Unix convention socket, got tcp:{addr}");
        }
    }
}

#[test]
fn test_content_endpoint_display_unix() {
    let ep = ContentEndpoint::Unix(std::path::PathBuf::from(
        "/run/user/1000/biomeos/content.sock",
    ));
    assert_eq!(format!("{ep}"), "unix:/run/user/1000/biomeos/content.sock");
}

#[test]
fn test_content_endpoint_display_tcp() {
    let ep = ContentEndpoint::Tcp("eastgate.mesh:9100".to_owned());
    assert_eq!(format!("{ep}"), "tcp:eastgate.mesh:9100");
}

#[tokio::test]
async fn test_content_backend_tcp_connect_failure_returns_error() {
    let client = ContentBackendClient {
        endpoint: ContentEndpoint::Tcp("127.0.0.1:1".to_owned()),
        request_id: std::sync::atomic::AtomicU64::new(1),
    };
    let result = client.resolve("/test").await;
    assert!(result.is_err(), "TCP connect to port 1 should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("tcp:"),
        "error should mention tcp endpoint: {err}"
    );
}

#[tokio::test]
async fn test_content_index_fallback_to_dashboard() {
    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(std::path::PathBuf::from(
            "/tmp/nonexistent-content-index-test.sock",
        )),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    let resp = super::content_backend::content_index(client).await;
    assert_eq!(
        resp.status(),
        200,
        "index should return 200 (dashboard fallback)"
    );
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(
        html.contains("petalTongue") || html.contains("<!DOCTYPE") || html.contains("<html"),
        "should serve compiled-in dashboard HTML"
    );
}

#[tokio::test]
async fn test_content_backend_resolve_via_unix_socket() {
    use base64::Engine as _;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("test-content.sock");

    let listener = UnixListener::bind(&sock_path).unwrap();

    let content = b"<h1>Hello from content backend</h1>";
    let sock_path_clone = sock_path.clone();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(req["method"], "content.resolve");
        assert_eq!(req["params"]["path"], "/index.html");

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "content": base64::engine::general_purpose::STANDARD.encode(content),
                "mime_type": "text/html"
            },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = ContentBackendClient {
        endpoint: ContentEndpoint::Unix(sock_path_clone),
        request_id: std::sync::atomic::AtomicU64::new(1),
    };
    let result = client.resolve("/index.html").await;
    server.await.unwrap();

    let (bytes, mime) = result.unwrap().expect("should get content");
    assert_eq!(mime, "text/html");
    assert_eq!(bytes, content);
}

#[tokio::test]
async fn test_content_backend_resolve_jsonrpc_error_returns_none() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("test-content-err.sock");

    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32601, "message": "Not found" },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = ContentBackendClient {
        endpoint: ContentEndpoint::Unix(sock_path),
        request_id: std::sync::atomic::AtomicU64::new(1),
    };
    let result = client.resolve("/missing").await;
    server.await.unwrap();

    assert!(
        result.unwrap().is_none(),
        "JSON-RPC error should return Ok(None)"
    );
}

#[tokio::test]
async fn test_content_backend_resolve_via_tcp() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let content = b"<p>TCP mesh content</p>";

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(req["method"], "content.resolve");

        use base64::Engine as _;
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "content": base64::engine::general_purpose::STANDARD.encode(content),
                "mime_type": "text/html"
            },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = ContentBackendClient {
        endpoint: ContentEndpoint::Tcp(addr.to_string()),
        request_id: std::sync::atomic::AtomicU64::new(1),
    };
    let result = client.resolve("/mesh/page.html").await;
    server.await.unwrap();

    let (bytes, mime) = result.unwrap().expect("should get content via TCP");
    assert_eq!(mime, "text/html");
    assert_eq!(bytes, content);
}
#[tokio::test]
async fn test_content_fallback_unavailable_returns_502() {
    use axum::body::Body;

    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(std::path::PathBuf::from(
            "/tmp/nonexistent-content-test.sock",
        )),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    let req = axum::http::Request::builder()
        .uri("/some/page.html")
        .body(Body::empty())
        .unwrap();
    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let resp = content_fallback(req, client, nb_cfg, 0).await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn test_content_backend_installs_fallback() {
    use axum::body::Body;

    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(std::path::PathBuf::from(
            "/tmp/nonexistent-content-test.sock",
        )),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    let client_clone = Arc::clone(&client);
    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let data_service = Arc::new(DataService::new());
    let app = Router::new()
        .route("/health", get(health_handler))
        .fallback(move |req: axum::extract::Request| {
            content_fallback(req, Arc::clone(&client_clone), Arc::clone(&nb_cfg), 0)
        })
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app.clone(), req).await.unwrap();
    assert_eq!(resp.status(), 200, "API route should still work");

    let req = axum::http::Request::builder()
        .uri("/unknown/path")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(
        resp.status(),
        502,
        "content fallback should return 502 when socket unavailable"
    );
}
// ── Content backend: success paths ─────────────────────────────────────

#[tokio::test]
async fn test_content_fallback_success_returns_body() {
    use base64::Engine as _;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("test-fb-success.sock");

    let listener = UnixListener::bind(&sock_path).unwrap();

    let content_bytes = b"<h1>Page content</h1>";

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(req["method"], "content.resolve");

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "content": base64::engine::general_purpose::STANDARD.encode(content_bytes),
                "mime_type": "text/html"
            },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(sock_path),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    let nb_config = Arc::new(crate::notebook_render::NotebookRenderConfig::default());

    let req = axum::http::Request::builder()
        .uri("/page.html")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = content_fallback(req, client, nb_config, 3600).await;
    server.await.unwrap();

    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], content_bytes);
}

#[tokio::test]
async fn test_content_fallback_not_found_returns_404() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("test-fb-404.sock");

    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32601, "message": "Not found" },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(sock_path),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    let nb_config = Arc::new(crate::notebook_render::NotebookRenderConfig::default());

    let req = axum::http::Request::builder()
        .uri("/missing-page")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = content_fallback(req, client, nb_config, 0).await;
    server.await.unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_content_fallback_notebook_mime_renders_html() {
    use base64::Engine as _;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("test-fb-nbmime.sock");

    let listener = UnixListener::bind(&sock_path).unwrap();

    let notebook_json = serde_json::json!({
        "nbformat": 4,
        "nbformat_minor": 5,
        "metadata": { "title": "Test Notebook" },
        "cells": [{
            "cell_type": "markdown",
            "source": ["# Hello World"],
            "metadata": {}
        }]
    });
    let notebook_bytes = serde_json::to_vec(&notebook_json).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "content": base64::engine::general_purpose::STANDARD.encode(&notebook_bytes),
                "mime_type": "application/x-ipynb+json"
            },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(sock_path),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    let nb_config = Arc::new(crate::notebook_render::NotebookRenderConfig::default());

    let req = axum::http::Request::builder()
        .uri("/abc123def456")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = content_fallback(req, client, nb_config, 0).await;
    server.await.unwrap();

    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/html"),
        "notebook MIME should render as HTML"
    );
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(
        html.contains("Test Notebook"),
        "should contain notebook title"
    );
    assert!(html.contains("Hello World"), "should contain cell content");
}

#[tokio::test]
async fn test_content_index_success_returns_provider_content() {
    use base64::Engine as _;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("test-ci-success.sock");

    let listener = UnixListener::bind(&sock_path).unwrap();

    let content_bytes = b"<html><body>Welcome</body></html>";

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(req["params"]["path"], "/");

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "content": base64::engine::general_purpose::STANDARD.encode(content_bytes),
                "mime_type": "text/html"
            },
            "id": req["id"]
        });
        let mut resp_line = serde_json::to_string(&resp).unwrap();
        resp_line.push('\n');
        writer.write_all(resp_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    });

    let client = Arc::new(ContentBackendClient {
        endpoint: ContentEndpoint::Unix(sock_path),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });

    let resp = super::content_backend::content_index(client).await;
    server.await.unwrap();

    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], content_bytes);
}
