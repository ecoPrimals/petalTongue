// ── Content-direct backend ─────────────────────────────────────────────

// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

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
