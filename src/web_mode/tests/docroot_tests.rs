// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::{build_cors_layer, security_headers_middleware};
use super::handlers::*;

use std::sync::Arc;

use axum::{Router, routing::get};

use crate::data_service::DataService;

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
// ── Security headers middleware ────────────────────────────────────────

#[tokio::test]
async fn test_security_headers_applied() {
    use axum::body::Body;

    let app = Router::new()
        .route("/test", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(security_headers_middleware));

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
