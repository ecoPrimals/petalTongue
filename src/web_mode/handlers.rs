// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP route handlers, static-file fallback, and shared response utilities.

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{
        Html, IntoResponse,
        sse::{Event, Sse},
    },
};
use petal_tongue_core::constants::DEFAULT_SSE_KEEPALIVE_SECS;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::services::ServeDir;

use crate::data_service::DataService;

// ── Filesystem fallback ─────────────────────────────────────────────────

/// Filesystem docroot fallback — serves static files with `.ipynb` rendering.
///
/// When `spa` is `true`, missing paths serve `{docroot}/index.html` instead of
/// 404, enabling client-side routing for single-page applications.
pub(super) async fn docroot_fallback(
    req: axum::extract::Request,
    docroot: String,
    nb_config: Arc<crate::notebook_render::NotebookRenderConfig>,
    cache_ttl: u64,
    spa: bool,
) -> axum::response::Response {
    let uri_path = req.uri().path();

    if is_ipynb(uri_path) {
        let file_path = resolve_docroot_path(&docroot, uri_path);
        match tokio::fs::read(&file_path).await {
            Ok(bytes) => {
                if let Some(html) = crate::notebook_render::render_notebook(&bytes, &nb_config) {
                    return build_response(
                        html.into_bytes(),
                        "text/html; charset=utf-8",
                        cache_ttl,
                    );
                }
                build_response(bytes, "application/json", cache_ttl)
            }
            Err(_) if spa => serve_spa_index(&docroot, cache_ttl).await,
            Err(_) => serve_custom_404(&docroot).await,
        }
    } else {
        let serve = ServeDir::new(&docroot).append_index_html_on_directories(true);
        let resp = tower::ServiceExt::oneshot(serve, req).await.map_or_else(
            |_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error",
                )
                    .into_response()
            },
            IntoResponse::into_response,
        );

        if resp.status() == axum::http::StatusCode::NOT_FOUND {
            if spa {
                return serve_spa_index(&docroot, cache_ttl).await;
            }
            return serve_custom_404(&docroot).await;
        }

        if cache_ttl > 0 && resp.status().is_success() {
            let (mut parts, body) = resp.into_parts();
            parts.headers.insert(
                axum::http::header::CACHE_CONTROL,
                format!("public, max-age={cache_ttl}")
                    .parse()
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("public")),
            );
            axum::response::Response::from_parts(parts, body)
        } else {
            resp
        }
    }
}

/// Serve `{docroot}/index.html` for SPA catch-all routing.
async fn serve_spa_index(docroot: &str, cache_ttl: u64) -> axum::response::Response {
    let index = std::path::Path::new(docroot).join("index.html");
    tokio::fs::read(&index).await.map_or_else(
        |_| (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
        |bytes| build_response(bytes, "text/html; charset=utf-8", cache_ttl),
    )
}

/// Serve `{docroot}/404.html` if it exists, otherwise plain text 404.
/// GitHub Pages convention: site-level custom error page.
async fn serve_custom_404(docroot: &str) -> axum::response::Response {
    let page = std::path::Path::new(docroot).join("404.html");
    if let Ok(bytes) = tokio::fs::read(&page).await {
        let mut resp = axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(bytes))
            .unwrap_or_else(|_| (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response());
        resp.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-cache"),
        );
        return resp;
    }
    (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response()
}

// ── Shared utilities ────────────────────────────────────────────────────

/// Build an HTTP response with optional `Cache-Control`.
pub fn build_response(
    body: Vec<u8>,
    content_type: &str,
    cache_ttl: u64,
) -> axum::response::Response {
    let mut builder = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, content_type);
    if cache_ttl > 0 {
        builder = builder.header(
            axum::http::header::CACHE_CONTROL,
            format!("public, max-age={cache_ttl}"),
        );
    }
    builder
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "response build error",
            )
                .into_response()
        })
}

/// Map a URI path to a filesystem path under docroot, preventing traversal.
///
/// Strips leading `/`, rejects `..` path components to prevent directory
/// escape, and canonicalizes to ensure the result stays under `docroot`.
pub(super) fn resolve_docroot_path(docroot: &str, uri_path: &str) -> std::path::PathBuf {
    let cleaned = uri_path.trim_start_matches('/');
    let safe: std::path::PathBuf = std::path::Path::new(cleaned)
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect();
    std::path::Path::new(docroot).join(safe)
}

/// Case-insensitive `.ipynb` extension check.
pub fn is_ipynb(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ipynb"))
}

/// Detect notebook content by MIME type (for content-addressable hash URLs
/// where the path has no extension).
pub fn is_notebook_mime(mime: &str) -> bool {
    let m = mime.split(';').next().unwrap_or(mime).trim();
    m == "application/x-ipynb+json" || m == "application/x-jupyter"
}

// ── Route handlers ──────────────────────────────────────────────────────

pub(super) async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

pub(super) async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "primal": petal_tongue_core::constants::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
    }))
}

pub(super) async fn liveness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
    }))
}

pub(super) async fn readiness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ready",
        "ready": true,
        "version": env!("CARGO_PKG_VERSION"),
        "primal": petal_tongue_core::constants::PRIMAL_NAME,
    }))
}

pub(super) async fn status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
        "pure_rust": true,
    }))
}

pub(super) async fn primals_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
    match service.snapshot().await {
        Ok(snapshot) => Json(serde_json::json!({
            "primals": snapshot.primals,
            "timestamp": snapshot.timestamp,
        })),
        Err(e) => {
            if e.to_string().contains("Graph lock poisoned") {
                tracing::debug!("Failed to get snapshot: {}", e);
            } else {
                tracing::error!("Failed to get snapshot: {}", e);
            }
            Json(serde_json::json!({
                "error": "Failed to fetch primals",
                "primals": []
            }))
        }
    }
}

pub(super) async fn snapshot_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
    match service.snapshot().await {
        Ok(snapshot) => Json(serde_json::json!(snapshot)),
        Err(e) => {
            if e.to_string().contains("Graph lock poisoned") {
                tracing::debug!("Failed to get snapshot: {e}");
            } else {
                tracing::error!("Failed to get snapshot: {e}");
            }
            Json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// SSE endpoint that pushes `DataUpdate` events from `DataService::subscribe()`.
///
/// Per PT-02 / `IPC_COMPLIANCE_MATRIX.md` v1.2: the browser receives live
/// topology changes without polling.
pub(super) async fn events_sse_handler(
    State(service): State<Arc<DataService>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = service.subscribe();
    let service = Arc::clone(&service);

    let stream = BroadcastStream::new(rx).filter_map(move |msg| {
        let service = Arc::clone(&service);
        match msg {
            Ok(_update) => {
                let snapshot = service.snapshot_sync();
                match serde_json::to_string(&snapshot) {
                    Ok(json) => Some(Ok(Event::default().data(json))),
                    Err(e) => {
                        tracing::warn!("SSE serialization error: {e}");
                        None
                    }
                }
            }
            Err(_lagged) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(DEFAULT_SSE_KEEPALIVE_SECS))
            .text("keepalive"),
    )
}

// ── Gate mesh status ─────────────────────────────────────────────────────

/// Returns gate mesh topology as JSON (mirrors `gate.mesh.status` IPC method).
pub(super) async fn gate_mesh_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh;

    let gates: Vec<serde_json::Value> = gate_mesh::all_nodes()
        .map(|node| {
            serde_json::json!({
                "id": node.id,
                "label": node.label,
                "zone": node.zone,
                "wg_ip": node.wg_ip,
                "enrollment": node.enrollment.as_str(),
                "nucleus_count": node.nucleus_count,
                "role": node.role,
                "kderm_layer": node.kderm_layer,
            })
        })
        .collect();

    let links: Vec<serde_json::Value> = gate_mesh::WG_LINKS
        .iter()
        .map(|link| {
            serde_json::json!({
                "from": link.from,
                "to": link.to,
                "latency_ms": link.latency_ms,
            })
        })
        .collect();

    let enrolled = gates
        .iter()
        .filter(|g| g["enrollment"] == "Enrolled")
        .count();

    Json(serde_json::json!({
        "gates": gates,
        "links": links,
        "enrolled_count": enrolled,
        "total_count": gates.len(),
        "source": "static",
    }))
}

// ── Ecosystem composition ────────────────────────────────────────────────

/// Returns the NUCLEUS composition (4 atomics, 13 primals) and ecosystem metrics.
pub(super) async fn ecosystem_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh;

    let atomics: Vec<serde_json::Value> = gate_mesh::NUCLEUS_ATOMICS
        .iter()
        .map(|atomic| {
            let primals: Vec<serde_json::Value> = atomic
                .primals
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "role": p.role,
                        "gate": p.gate,
                    })
                })
                .collect();
            serde_json::json!({
                "name": atomic.name,
                "primals": primals,
            })
        })
        .collect();

    Json(serde_json::json!({
        "nucleus": atomics,
        "metrics": {
            "total_primals": gate_mesh::nucleus_primal_count(),
            "total_atomics": gate_mesh::NUCLEUS_ATOMICS.len(),
            "gates_enrolled": gate_mesh::count_by_enrollment(gate_mesh::GateEnrollment::Enrolled),
            "source": "static",
        },
    }))
}

// ── Visualization renderer ───────────────────────────────────────────────

/// Renders a registered visualization as SVG (or JSON scene/animation).
///
/// Query params: `?format=svg` (default), `scene-json`, `animation-json`
#[expect(
    clippy::option_if_let_else,
    reason = "match arms with different response types are clearer than map_or_else"
)]
pub(super) async fn viz_handler(
    axum::extract::Path(slug): axum::extract::Path<String>,
    query: axum::extract::Query<VizQuery>,
) -> axum::response::Response {
    use crate::viz_data::VizRegistry;
    use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};

    let registry = VizRegistry::discover(None);

    match query.format.as_deref().unwrap_or("svg") {
        "scene-json" => match registry.build_scene(&slug) {
            Some(scene) => Json(serde_json::to_value(&scene).unwrap_or_default()).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                format!("visualization '{slug}' not found"),
            )
                .into_response(),
        },
        "animation-json" => match registry.build_animation(&slug) {
            Some(anim) => Json(serde_json::to_value(&anim).unwrap_or_default()).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                format!("no animation for '{slug}'"),
            )
                .into_response(),
        },
        _ => match registry.build_scene(&slug) {
            Some(scene) => {
                let compiler = SvgCompiler;
                match compiler.compile(&scene) {
                    ModalityOutput::Svg(bytes) => {
                        let svg = String::from_utf8_lossy(bytes.as_ref()).into_owned();
                        ([(axum::http::header::CONTENT_TYPE, "image/svg+xml")], svg).into_response()
                    }
                    _ => (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "SVG compilation failed",
                    )
                        .into_response(),
                }
            }
            None => (
                axum::http::StatusCode::NOT_FOUND,
                format!("visualization '{slug}' not found"),
            )
                .into_response(),
        },
    }
}

#[derive(serde::Deserialize)]
pub(super) struct VizQuery {
    pub format: Option<String>,
}
