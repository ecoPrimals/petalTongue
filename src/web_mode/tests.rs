// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code uses unwrap/expect for brevity"
)]

use super::content_backend::{ContentBackendClient, ContentEndpoint, content_fallback};
use super::handlers::*;
use super::*;

use std::sync::Arc;

use axum::{Router, extract::State, response::IntoResponse, routing::get};

use crate::data_service::DataService;

#[tokio::test]
async fn test_status_endpoint() {
    let response = status_handler().await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_primals_endpoint() {
    let data_service = Arc::new(DataService::new());
    let response = primals_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_health_endpoint() {
    let response = health_handler().await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_index_endpoint_returns_html() {
    let html = index_handler().await;
    assert!(!html.0.is_empty());
}

#[test]
fn test_bind_address_parse() {
    use petal_tongue_core::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_LOOPBACK_HOST};
    let bind = format!("{DEFAULT_LOOPBACK_HOST}:{DEFAULT_HEADLESS_PORT}");
    let addr: std::net::SocketAddr = bind.parse().expect("valid bind");
    assert_eq!(addr.port(), DEFAULT_HEADLESS_PORT);
}

#[test]
fn test_invalid_bind_address() {
    let result: Result<std::net::SocketAddr, _> = "not-an-address".parse();
    assert!(result.is_err());
}

#[test]
fn test_bind_address_default_format() {
    use petal_tongue_core::constants::{DEFAULT_BIND_HOST, DEFAULT_WEB_PORT};
    let bind = format!("{DEFAULT_BIND_HOST}:{DEFAULT_WEB_PORT}");
    let addr: std::net::SocketAddr = bind.parse().expect("valid default bind");
    assert_eq!(addr.port(), DEFAULT_WEB_PORT);
    assert!(addr.ip().is_unspecified());
}

#[test]
fn test_bind_address_loopback_with_port() {
    use petal_tongue_core::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_LOOPBACK_HOST};
    let bind = format!("{DEFAULT_LOOPBACK_HOST}:{DEFAULT_HEADLESS_PORT}");
    let addr: std::net::SocketAddr = bind.parse().expect("valid loopback");
    assert_eq!(addr.port(), DEFAULT_HEADLESS_PORT);
    assert!(addr.ip().is_loopback());
}

#[tokio::test]
async fn test_snapshot_endpoint() {
    let data_service = Arc::new(DataService::new());
    let response = snapshot_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_status_response_body() {
    let response = status_handler().await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["mode"], "web");
    assert_eq!(json["pure_rust"], true);
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn test_health_response_body() {
    let response = health_handler().await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_primals_response_structure() {
    let data_service = Arc::new(DataService::new());
    let response = primals_handler(State(data_service)).await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["primals"].is_array());
    assert!(json["timestamp"].is_number());
}

#[tokio::test]
async fn test_snapshot_response_structure() {
    let data_service = Arc::new(DataService::new());
    let response = snapshot_handler(State(data_service)).await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["primals"].is_array());
    assert!(json["edges"].is_array());
    assert!(json["timestamp"].is_number());
}

#[tokio::test]
async fn test_router_construction() {
    use axum::body::Body;

    let data_service = Arc::new(DataService::new());
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_router_status_endpoint() {
    use axum::body::Body;

    let data_service = Arc::new(DataService::new());
    let app = Router::new()
        .route("/api/status", get(status_handler))
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_events_sse_returns_event_stream() {
    use axum::body::Body;

    let data_service = Arc::new(DataService::new());
    let app = Router::new()
        .route("/api/events", get(events_sse_handler))
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .uri("/api/events")
        .body(Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), 200);
    let ct = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/event-stream"),
        "Expected text/event-stream, got: {ct}"
    );
}

fn test_config(bind: &str) -> WebConfig<'_> {
    WebConfig {
        bind,
        scenario: None,
        docroot: None,
        backend: "filesystem",
        workers: 4,
        strip_sources: false,
        cache_ttl_secs: 0,
        spa: false,
        allowed_origins: Vec::new(),
    }
}

#[tokio::test]
async fn test_run_invalid_bind_address() {
    let data_service = Arc::new(DataService::new());
    let result = run(test_config("not-a-valid-address"), data_service).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("parse") || err_msg.contains("bind"));
}

#[tokio::test]
async fn test_run_empty_bind_address() {
    let data_service = Arc::new(DataService::new());
    let result = run(test_config(""), data_service).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_invalid_port() {
    let data_service = Arc::new(DataService::new());
    let result = run(test_config("127.0.0.1:999999"), data_service).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_primals_handler_snapshot_error() {
    let data_service = Arc::new(DataService::new());
    let graph = data_service.graph();
    let _ = std::thread::spawn(move || {
        let _guard = graph.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let response = primals_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().is_some());
    assert!(json["primals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_snapshot_handler_snapshot_error() {
    let data_service = Arc::new(DataService::new());
    let graph = data_service.graph();
    let _ = std::thread::spawn(move || {
        let _guard = graph.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let response = snapshot_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().is_some());
}

#[tokio::test]
async fn test_index_handler_contains_doctype() {
    let html = index_handler().await;
    assert!(html.0.contains("<!DOCTYPE html>"));
    assert!(html.0.contains("petalTongue"));
}

#[tokio::test]
async fn test_run_invalid_docroot_rejects() {
    let data_service = Arc::new(DataService::new());
    let mut cfg = test_config("127.0.0.1:0");
    cfg.docroot = Some("/nonexistent/docroot/path".to_string());
    let result = run(cfg, data_service).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("docroot"),
        "error should mention docroot: {msg}"
    );
}

#[tokio::test]
async fn test_docroot_fallback_serves_static_files() {
    use axum::body::Body;
    use tower_http::services::ServeDir;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("index.html"), "<html>hello</html>").expect("write");
    std::fs::write(tmp.path().join("style.css"), "body {}").expect("write");

    let data_service = Arc::new(DataService::new());
    let app = Router::new()
        .route("/health", get(health_handler))
        .fallback_service(ServeDir::new(tmp.path()).append_index_html_on_directories(true))
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .uri("/style.css")
        .body(Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app.clone(), req).await.unwrap();
    assert_eq!(response.status(), 200);

    let req = axum::http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app.clone(), req).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(
        std::str::from_utf8(&body).unwrap().contains("hello"),
        "/ should serve index.html from docroot"
    );
}

#[tokio::test]
async fn test_api_routes_take_precedence_over_docroot() {
    use axum::body::Body;
    use tower_http::services::ServeDir;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("api")).expect("mkdir");
    std::fs::write(tmp.path().join("api/status"), "shadowed").expect("write");

    let data_service = Arc::new(DataService::new());
    let app = Router::new()
        .route("/api/status", get(status_handler))
        .fallback_service(ServeDir::new(tmp.path()).append_index_html_on_directories(true))
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json["mode"], "web",
        "API route should take precedence over docroot file"
    );
}

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

#[test]
fn test_resolve_docroot_path_prevents_traversal() {
    let docroot = "/srv/content";
    let safe = resolve_docroot_path(docroot, "/docs/page.html");
    assert_eq!(
        safe,
        std::path::PathBuf::from("/srv/content/docs/page.html")
    );

    let escaped = resolve_docroot_path(docroot, "/../../../etc/passwd");
    assert_eq!(
        escaped,
        std::path::PathBuf::from("/srv/content/etc/passwd"),
        ".. components stripped, remaining segments stay under docroot"
    );

    let double = resolve_docroot_path(docroot, "/a/../b/c");
    assert!(
        !double.to_string_lossy().contains(".."),
        "parent components should be stripped: {}",
        double.display()
    );
    assert_eq!(
        double,
        std::path::PathBuf::from("/srv/content/a/b/c"),
        "only Normal components are kept"
    );
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

#[tokio::test]
async fn test_docroot_renders_ipynb_as_html() {
    use axum::body::Body;

    let tmp = tempfile::tempdir().expect("tempdir");
    let nb = serde_json::json!({
        "nbformat": 4, "nbformat_minor": 5,
        "metadata": { "title": "Integration Test" },
        "cells": [{
            "cell_type": "markdown",
            "source": ["# Heading"],
            "metadata": {}
        }]
    });
    std::fs::write(
        tmp.path().join("demo.ipynb"),
        serde_json::to_vec(&nb).expect("ser"),
    )
    .expect("write");

    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let docroot = tmp.path().to_string_lossy().into_owned();
    let app = Router::new()
        .fallback(move |req: axum::extract::Request| {
            docroot_fallback(req, docroot, nb_cfg, 0, false)
        })
        .with_state(Arc::new(DataService::new()));

    let req = axum::http::Request::builder()
        .uri("/demo.ipynb")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/html"),
        "notebook should be served as HTML"
    );
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(html.contains("Integration Test"));
    assert!(html.contains("Heading"));
}

#[tokio::test]
async fn test_docroot_cache_control_header() {
    use axum::body::Body;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("style.css"), "body {}").expect("write");

    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let docroot = tmp.path().to_string_lossy().into_owned();
    let app = Router::new()
        .fallback(move |req: axum::extract::Request| {
            docroot_fallback(req, docroot, nb_cfg, 3600, false)
        })
        .with_state(Arc::new(DataService::new()));

    let req = axum::http::Request::builder()
        .uri("/style.css")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let cc = resp
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        cc.contains("max-age=3600"),
        "Cache-Control should contain ttl: {cc}"
    );
}

#[test]
fn test_is_ipynb() {
    assert!(is_ipynb("/path/to/notebook.ipynb"));
    assert!(is_ipynb("/path/to/notebook.IPYNB"));
    assert!(!is_ipynb("/path/to/file.json"));
    assert!(!is_ipynb("/path/to/file"));
}

#[test]
fn test_is_notebook_mime() {
    assert!(is_notebook_mime("application/x-ipynb+json"));
    assert!(is_notebook_mime("application/x-jupyter"));
    assert!(is_notebook_mime("application/x-ipynb+json; charset=utf-8"));
    assert!(!is_notebook_mime("application/json"));
    assert!(!is_notebook_mime("text/html"));
    assert!(!is_notebook_mime("application/octet-stream"));
}

#[test]
fn test_resolve_docroot_path() {
    let path = resolve_docroot_path("/srv/www", "/docs/page.html");
    assert_eq!(path, std::path::PathBuf::from("/srv/www/docs/page.html"));
}

#[tokio::test]
async fn test_spa_serves_index_for_missing_path() {
    use axum::body::Body;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("index.html"), "<html>SPA</html>").expect("write");
    std::fs::write(tmp.path().join("style.css"), "body {}").expect("write");

    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let docroot = tmp.path().to_string_lossy().into_owned();
    let app = Router::new()
        .fallback(move |req: axum::extract::Request| {
            docroot_fallback(req, docroot, nb_cfg, 0, true)
        })
        .with_state(Arc::new(DataService::new()));

    let req = axum::http::Request::builder()
        .uri("/nonexistent/route")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app.clone(), req).await.unwrap();
    assert_eq!(
        resp.status(),
        200,
        "SPA should serve index.html for missing paths"
    );
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(
        std::str::from_utf8(&body).unwrap().contains("SPA"),
        "body should be index.html content"
    );

    let req = axum::http::Request::builder()
        .uri("/style.css")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), 200, "existing files should still be served");
}

#[tokio::test]
async fn test_non_spa_returns_404() {
    use axum::body::Body;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("index.html"), "<html>ok</html>").expect("write");

    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let docroot = tmp.path().to_string_lossy().into_owned();
    let app = Router::new()
        .fallback(move |req: axum::extract::Request| {
            docroot_fallback(req, docroot, nb_cfg, 0, false)
        })
        .with_state(Arc::new(DataService::new()));

    let req = axum::http::Request::builder()
        .uri("/nonexistent/route")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), 404, "non-SPA mode should return 404");
}

#[test]
fn test_cors_layer_wildcard() {
    let layer = build_cors_layer(&["*".to_string()]);
    drop(layer);
}

#[test]
fn test_cors_layer_specific_origins() {
    let layer = build_cors_layer(&[
        "https://primals.eco".to_string(),
        "http://localhost:3000".to_string(),
    ]);
    drop(layer);
}

#[tokio::test]
async fn test_cors_preflight_responds() {
    use axum::body::Body;

    let data_service = Arc::new(DataService::new());
    let cors = build_cors_layer(&["*".to_string()]);
    let app = Router::new()
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(data_service);

    let req = axum::http::Request::builder()
        .method(axum::http::Method::OPTIONS)
        .uri("/health")
        .header("origin", "https://example.com")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    let acao = resp
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(acao, "*", "wildcard CORS should respond with *");
}

// ── Liveness & Readiness handlers ──────────────────────────────────────

#[tokio::test]
async fn test_liveness_handler() {
    let resp = liveness_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "alive");
}

#[tokio::test]
async fn test_readiness_handler() {
    let resp = readiness_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "ready");
    assert_eq!(v["ready"], true);
    assert!(v["version"].as_str().is_some());
    assert!(v["primal"].as_str().is_some());
}

// ── Security headers middleware ────────────────────────────────────────

#[tokio::test]
async fn test_security_headers_applied() {
    use axum::body::Body;

    let app =
        Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(
                super::security_headers_middleware,
            ));

    let req = axum::http::Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
    assert_eq!(
        resp.headers().get("referrer-policy").unwrap(),
        "strict-origin-when-cross-origin"
    );
    assert!(
        resp.headers()
            .get("permissions-policy")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("camera=()")
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

// ── Content-direct backend ─────────────────────────────────────────────

#[tokio::test]
async fn test_content_direct_index_serves_root_page() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();
    std::fs::write(
        content_dir.join("_index.md"),
        "+++\ntitle = \"Home\"\n+++\n# Welcome\n\nThis is the home page.",
    )
    .unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let resp = super::content_direct::content_direct_index(Arc::clone(&state)).await;
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(
        html.contains("Welcome") || html.contains("Home"),
        "index should render markdown content"
    );
}

#[tokio::test]
async fn test_content_direct_index_fallback_when_no_index() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let resp = super::content_direct::content_direct_index(Arc::clone(&state)).await;
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(
        html.contains("Content-direct mode active"),
        "should serve fallback HTML"
    );
}

#[tokio::test]
async fn test_content_direct_nav_returns_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir_all(content_dir.join("docs")).unwrap();
    std::fs::write(
        content_dir.join("docs/_index.md"),
        "+++\ntitle = \"Documentation\"\nweight = 1\n+++\n",
    )
    .unwrap();
    std::fs::write(
        content_dir.join("docs/getting-started.md"),
        "+++\ntitle = \"Getting Started\"\n+++\n# Getting Started",
    )
    .unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let resp = super::content_direct::content_direct_nav(Arc::clone(&state)).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/json"));

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let nav: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(nav.is_array(), "nav should be a JSON array");
    let sections = nav.as_array().unwrap();
    assert!(!sections.is_empty(), "should discover docs section");
    assert_eq!(sections[0]["title"], "Documentation");
}

#[tokio::test]
async fn test_content_direct_viz_list_returns_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let resp = super::content_direct::content_direct_viz_list(Arc::clone(&state)).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/json"));

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let viz: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(viz.is_array(), "viz list should be a JSON array");
    let entries = viz.as_array().unwrap();
    assert!(entries.len() >= 2, "should have kderm + nucleus built-in");
}

#[tokio::test]
async fn test_content_direct_fallback_serves_markdown() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();
    std::fs::write(
        content_dir.join("about.md"),
        "+++\ntitle = \"About\"\n+++\n# About Page\n\nSome content here.",
    )
    .unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/about")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(
        html.contains("About") || html.contains("about"),
        "should render about page"
    );
}

#[tokio::test]
async fn test_content_direct_fallback_serves_static_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    let static_dir = tmp.path().join("static");
    std::fs::create_dir(&content_dir).unwrap();
    std::fs::create_dir(&static_dir).unwrap();
    std::fs::write(static_dir.join("style.css"), "body { color: red; }").unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/style.css")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 300).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/css"),
        "should serve CSS with correct MIME"
    );
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"body { color: red; }");
}

#[tokio::test]
async fn test_content_direct_fallback_returns_404_for_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/nonexistent")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_content_direct_modality_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();
    std::fs::write(
        content_dir.join("test.md"),
        "+++\ntitle = \"Test\"\n+++\n# Test Page",
    )
    .unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/test?modality=json")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("application/json"),
        "modality=json should return JSON"
    );
}

#[tokio::test]
async fn test_content_direct_viz_route_kderm_svg() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/viz/kderm-topology")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("svg"),
        "kderm-topology default format should be SVG"
    );
}

#[tokio::test]
async fn test_content_direct_viz_route_scene_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/viz/nucleus-composition?format=scene-json")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/json"));
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let scene: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(scene.is_object(), "scene-json should return valid JSON");
}

#[tokio::test]
async fn test_content_direct_viz_route_animation_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/viz/nucleus-composition?format=animation-json")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/json"));
}

#[tokio::test]
async fn test_content_direct_viz_route_unknown_viz() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    let state = Arc::new(super::content_direct::ContentDirectState::new(content_dir));

    let req = axum::http::Request::builder()
        .uri("/viz/nonexistent-viz")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = super::content_direct::content_direct_fallback(req, Arc::clone(&state), 0).await;
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = std::str::from_utf8(&body).unwrap();
    assert!(
        text.contains("Unknown visualization"),
        "should report unknown viz"
    );
}

// ── Viz data: registry + scene builders ────────────────────────────────

#[test]
fn test_viz_registry_discover_without_static() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let available = reg.available();
    assert!(available.contains(&"kderm-topology"), "should have kderm");
    assert!(
        available.contains(&"nucleus-composition"),
        "should have nucleus"
    );
    assert!(
        !available.contains(&"entity-graph"),
        "entity-graph needs static dir"
    );
}

#[test]
fn test_viz_registry_discover_with_entity_graph() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph_dir = tmp.path().join("graph");
    std::fs::create_dir(&graph_dir).unwrap();
    std::fs::write(
        graph_dir.join("entity-graph.json"),
        r#"{"nodes":[],"edges":[]}"#,
    )
    .unwrap();

    let reg = crate::viz_data::VizRegistry::discover(Some(tmp.path()));
    assert!(
        reg.available().contains(&"entity-graph"),
        "should discover entity-graph with JSON file"
    );
    assert_eq!(reg.list().len(), 3, "should have 3 visualizations total");
}

#[test]
fn test_viz_registry_get_and_list() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let entry = reg.get("kderm-topology").unwrap();
    assert_eq!(entry.slug, "kderm-topology");
    assert!(entry.has_animation);
    assert!(entry.data_source.is_none());

    let nucleus = reg.get("nucleus-composition").unwrap();
    assert_eq!(nucleus.title, "NUCLEUS Atomics Composition");
    assert!(nucleus.has_animation);

    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn test_viz_registry_build_kderm_scene() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let scene = reg.build_scene("kderm-topology");
    assert!(scene.is_some(), "kderm scene should build");
    let scene = scene.unwrap();
    assert!(
        scene.node_count() > 1,
        "scene should have nodes beyond root"
    );
}

#[test]
fn test_viz_registry_build_nucleus_scene() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let scene = reg.build_scene("nucleus-composition");
    assert!(scene.is_some(), "nucleus scene should build");
    let scene = scene.unwrap();
    assert!(
        scene.node_count() > 1,
        "scene should have nodes beyond root"
    );
}

#[test]
fn test_viz_registry_build_unknown_returns_none() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    assert!(reg.build_scene("no-such-viz").is_none());
}

#[test]
fn test_viz_registry_build_kderm_animation() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let anim = reg.build_animation("kderm-topology");
    assert!(anim.is_some(), "kderm should have animation");
}

#[test]
fn test_viz_registry_build_nucleus_animation() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let anim = reg.build_animation("nucleus-composition");
    assert!(anim.is_some(), "nucleus should have animation");
}

#[test]
fn test_viz_registry_no_animation_for_entity_graph() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph_dir = tmp.path().join("graph");
    std::fs::create_dir(&graph_dir).unwrap();
    std::fs::write(
        graph_dir.join("entity-graph.json"),
        r#"{"nodes":[],"edges":[]}"#,
    )
    .unwrap();

    let reg = crate::viz_data::VizRegistry::discover(Some(tmp.path()));
    let anim = reg.build_animation("entity-graph");
    assert!(anim.is_none(), "entity-graph should not have animation");
}

#[test]
fn test_build_nucleus_scene_directly() {
    let scene = crate::viz_data::build_nucleus_scene();
    assert!(scene.node_count() > 1);
}

#[test]
fn test_build_nucleus_expand_animation() {
    let anim = crate::viz_data::build_nucleus_expand_animation("tower-atomic");
    match anim {
        petal_tongue_scene::animation::Sequence::Sequential(anims) => {
            assert!(!anims.is_empty(), "animation should have keyframes");
        }
        _ => panic!("expected Sequential"),
    }
}

#[test]
fn test_build_kderm_scene_directly() {
    let scene = crate::viz_data::build_kderm_scene();
    assert!(scene.node_count() > 1);
}

#[test]
fn test_build_kderm_relay_animation() {
    let anim = crate::viz_data::build_kderm_relay_animation();
    match anim {
        petal_tongue_scene::animation::Sequence::Sequential(anims) => {
            assert!(!anims.is_empty(), "animation should have keyframes");
        }
        _ => panic!("expected Sequential"),
    }
}

// ── Content render: site.rs ────────────────────────────────────────────

#[test]
fn test_load_entity_registry_from_config() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path().join("config.toml");
    std::fs::write(
        &config,
        r#"
[extra.entity_registry.beardog]
display = "BearDog"
emoji = "🐻"
kind = "primal"
description = "Cryptographic security"
"#,
    )
    .unwrap();

    let registry = crate::content_render::load_entity_registry(&config);
    assert_eq!(registry.len(), 1);
    let entry = registry.get("beardog").unwrap();
    assert_eq!(entry.display, "BearDog");
    assert_eq!(entry.emoji, "🐻");
    assert_eq!(entry.kind, "primal");
    assert_eq!(entry.description.as_deref(), Some("Cryptographic security"));
}

#[test]
fn test_load_entity_registry_missing_file() {
    let registry = crate::content_render::load_entity_registry(std::path::Path::new(
        "/nonexistent/config.toml",
    ));
    assert!(registry.is_empty());
}

#[test]
fn test_load_entity_registry_no_extra_section() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path().join("config.toml");
    std::fs::write(&config, "[site]\ntitle = \"Test\"").unwrap();

    let registry = crate::content_render::load_entity_registry(&config);
    assert!(registry.is_empty());
}

#[test]
fn test_build_nav_tree_from_content_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content_dir = tmp.path().join("content");
    std::fs::create_dir(&content_dir).unwrap();

    std::fs::create_dir(content_dir.join("primals")).unwrap();
    std::fs::write(
        content_dir.join("primals/_index.md"),
        "+++\ntitle = \"Primals\"\nweight = 1\n+++\n",
    )
    .unwrap();
    std::fs::write(
        content_dir.join("primals/beardog.md"),
        "+++\ntitle = \"BearDog\"\n+++\n# BearDog",
    )
    .unwrap();

    std::fs::create_dir(content_dir.join("springs")).unwrap();
    std::fs::write(
        content_dir.join("springs/_index.md"),
        "+++\ntitle = \"Springs\"\nweight = 2\n+++\n",
    )
    .unwrap();

    let nav = crate::content_render::build_nav_tree(&content_dir);
    assert_eq!(nav.len(), 2, "should have 2 sections");
    assert_eq!(nav[0].title, "Primals");
    assert_eq!(nav[1].title, "Springs");
    assert_eq!(nav[0].pages.len(), 1, "primals section should have 1 page");
    assert_eq!(nav[0].pages[0].title, "BearDog");
}

#[test]
fn test_build_nav_tree_empty_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let nav = crate::content_render::build_nav_tree(tmp.path());
    assert!(nav.is_empty());
}

#[test]
fn test_build_nav_tree_nonexistent_dir() {
    let nav = crate::content_render::build_nav_tree(std::path::Path::new("/nonexistent/dir"));
    assert!(nav.is_empty());
}

// ── Custom 404 page ────────────────────────────────────────────────────

#[tokio::test]
async fn test_docroot_custom_404_html() {
    use axum::body::Body;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        tmp.path().join("404.html"),
        "<html><body>Custom Not Found</body></html>",
    )
    .unwrap();

    let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
    let docroot = tmp.path().to_string_lossy().into_owned();
    let app = Router::new().fallback(move |req: axum::extract::Request| {
        docroot_fallback(req, docroot, nb_cfg, 0, false)
    });

    let req = axum::http::Request::builder()
        .uri("/nonexistent-page")
        .body(Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), 404);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(
        html.contains("Custom Not Found"),
        "should serve custom 404.html"
    );
}

// ── run() content-direct invalid docroot ───────────────────────────────

#[tokio::test]
async fn test_run_content_direct_invalid_docroot_rejects() {
    let mut cfg = test_config("127.0.0.1:0");
    cfg.backend = "content-direct";
    cfg.docroot = Some("/nonexistent/content/dir".to_owned());
    let ds = Arc::new(DataService::new());
    let result = super::run(cfg, ds).await;
    assert!(
        result.is_err(),
        "content-direct with invalid docroot should fail"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Web config") || err_msg.contains("docroot"),
        "error should mention docroot: {err_msg}"
    );
}
