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
