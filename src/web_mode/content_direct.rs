// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filesystem-direct content backend for local validation.
//!
//! Reads raw markdown from a content directory, renders through the
//! DocumentNode pipeline, and serves the result. This mirrors what
//! `content-provider` does via NestGate IPC, but reads from disk directly.
//!
//! Usage: `petaltongue web --backend content-direct --docroot <content_dir>`
//!
//! The content directory should be the sporePrint `content/` folder.
//! A sibling `config.toml` is loaded for entity registry resolution.
//! A sibling `static/` directory is served for CSS/images.

use axum::response::IntoResponse;
use petal_tongue_scene::document::{EntityRegistryEntry, NavSection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::content_render;

/// Shared state for the content-direct backend.
pub struct ContentDirectState {
    pub content_dir: PathBuf,
    pub static_dir: Option<PathBuf>,
    pub registry: HashMap<String, EntityRegistryEntry>,
    pub nav: Vec<NavSection>,
}

impl ContentDirectState {
    /// Initialize from a content directory path.
    /// Expects `config.toml` as sibling of the content dir (or parent).
    pub fn new(content_dir: PathBuf) -> Self {
        let parent = content_dir.parent().unwrap_or(Path::new("."));
        let config_path = parent.join("config.toml");
        let static_dir = parent.join("static");

        let registry = if config_path.exists() {
            content_render::load_entity_registry(&config_path)
        } else {
            tracing::warn!(?config_path, "config.toml not found, entity resolution disabled");
            HashMap::new()
        };

        let nav = content_render::build_nav_tree(&content_dir);

        let static_dir = if static_dir.is_dir() {
            Some(static_dir)
        } else {
            None
        };

        tracing::info!(
            content_dir = %content_dir.display(),
            entities = registry.len(),
            nav_sections = nav.len(),
            static_dir = ?static_dir.as_ref().map(|p| p.display().to_string()),
            "Content-direct backend initialized"
        );

        Self { content_dir, static_dir, registry, nav }
    }

    /// Resolve a URL path to a markdown file on disk.
    fn resolve_content_path(&self, url_path: &str) -> Option<PathBuf> {
        let clean = url_path.trim_start_matches('/');

        // Try: content/<path>.md
        let direct = self.content_dir.join(format!("{clean}.md"));
        if direct.is_file() {
            return Some(direct);
        }

        // Try: content/<path>/index.md (Zola convention: _index.md)
        let index = self.content_dir.join(clean).join("_index.md");
        if index.is_file() {
            return Some(index);
        }

        // Try: content/<path>/index.md (standard)
        let std_index = self.content_dir.join(clean).join("index.md");
        if std_index.is_file() {
            return Some(std_index);
        }

        // Root index
        if clean.is_empty() || clean == "/" {
            let root_index = self.content_dir.join("_index.md");
            if root_index.is_file() {
                return Some(root_index);
            }
        }

        None
    }

    /// Resolve a static file path.
    /// Zola convention: `static/css/main.css` is served at `/css/main.css`.
    fn resolve_static_path(&self, url_path: &str) -> Option<PathBuf> {
        let static_dir = self.static_dir.as_ref()?;
        let clean = url_path.trim_start_matches('/');
        let path = static_dir.join(clean);
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    }
}

/// Index handler for the content-direct backend.
pub async fn content_direct_index(state: Arc<ContentDirectState>) -> axum::response::Response {
    let root_path = state.content_dir.join("_index.md");
    if root_path.is_file() {
        match std::fs::read_to_string(&root_path) {
            Ok(content) => render_content(&content, "/", &state),
            Err(_) => fallback_index().into_response(),
        }
    } else {
        fallback_index().into_response()
    }
}

/// Fallback handler — resolves content from the filesystem.
pub async fn content_direct_fallback(
    req: axum::extract::Request,
    state: Arc<ContentDirectState>,
    cache_ttl: u64,
) -> axum::response::Response {
    let path = req.uri().path().to_owned();
    let accept = req
        .headers()
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_owned();
    let query = req.uri().query().unwrap_or("").to_owned();

    // Static files first (Zola convention: static/foo served at /foo)
    if let Some(file_path) = state.resolve_static_path(&path) {
        return serve_static_file(&file_path, cache_ttl).await;
    }

    // Content resolution
    if let Some(file_path) = state.resolve_content_path(&path) {
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                return render_content_with_modality(&content, &path, &accept, &query, &state, cache_ttl);
            }
            Err(e) => {
                tracing::error!(path = %file_path.display(), error = %e, "Failed to read content");
            }
        }
    }

    (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response()
}

/// Render markdown content through the DocumentNode pipeline.
fn render_content(source: &str, path: &str, state: &ContentDirectState) -> axum::response::Response {
    render_content_with_modality(source, path, "text/html", "", state, 0)
}

/// Render with modality negotiation.
fn render_content_with_modality(
    source: &str,
    path: &str,
    accept: &str,
    query: &str,
    state: &ContentDirectState,
    cache_ttl: u64,
) -> axum::response::Response {
    use petal_tongue_scene::modality::document_compiler;

    let mut doc = content_render::parse_document(source, path);

    // Resolve shortcodes against registry
    if let petal_tongue_scene::document::DocumentNode::Page { body, .. } = &mut doc {
        content_render::resolve_shortcodes(body, &state.registry);
    }

    let wants_description = accept.contains("text/plain") || query.contains("modality=description");
    let wants_json = accept.contains("application/json") || query.contains("modality=json");

    if wants_json {
        let json = serde_json::to_string_pretty(&doc).unwrap_or_default();
        build_response(json.into_bytes(), "application/json; charset=utf-8", cache_ttl)
    } else if wants_description {
        let output = document_compiler::compile_to_description(&doc);
        match output {
            petal_tongue_scene::ModalityOutput::Description(bytes) => {
                build_response(bytes.to_vec(), "text/plain; charset=utf-8", cache_ttl)
            }
            _ => build_response(b"Rendering error".to_vec(), "text/plain", 0),
        }
    } else {
        let output = document_compiler::compile_to_html(&doc);
        match output {
            petal_tongue_scene::ModalityOutput::Svg(bytes) => {
                build_response(bytes.to_vec(), "text/html; charset=utf-8", cache_ttl)
            }
            _ => build_response(b"Rendering error".to_vec(), "text/plain", 0),
        }
    }
}

async fn serve_static_file(path: &Path, cache_ttl: u64) -> axum::response::Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => {
            let mime = mime_from_extension(path);
            build_response(bytes, mime, cache_ttl)
        }
        Err(_) => (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

fn mime_from_extension(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ico") => "image/x-icon",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn build_response(body: Vec<u8>, content_type: &str, cache_ttl: u64) -> axum::response::Response {
    use axum::http::{HeaderValue, header};

    let mut resp = axum::response::Response::new(axum::body::Body::from(body));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    if cache_ttl > 0 {
        let val = format!("public, max-age={cache_ttl}");
        if let Ok(hv) = HeaderValue::from_str(&val) {
            resp.headers_mut().insert(header::CACHE_CONTROL, hv);
        }
    }
    resp
}

fn fallback_index() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html><html><head><title>sporePrint (petalTongue)</title></head>
<body><h1>sporePrint</h1><p>Content-direct mode active. Navigate to a content path.</p></body></html>"#,
    )
}
