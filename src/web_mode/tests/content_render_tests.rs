// ── Content render: site.rs ────────────────────────────────────────────

// SPDX-License-Identifier: AGPL-3.0-or-later

use super::handlers::*;

use std::sync::Arc;

use axum::Router;

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
