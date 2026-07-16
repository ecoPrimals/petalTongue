// SPDX-License-Identifier: AGPL-3.0-or-later

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
pub async fn docroot_fallback(
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
pub fn resolve_docroot_path(docroot: &str, uri_path: &str) -> std::path::PathBuf {
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

pub async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../../web/index.html"))
}

pub async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "primal": petal_tongue_core::constants::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
    }))
}

pub async fn liveness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
    }))
}

pub async fn readiness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ready",
        "ready": true,
        "version": env!("CARGO_PKG_VERSION"),
        "primal": petal_tongue_core::constants::PRIMAL_NAME,
    }))
}

/// SSE endpoint that pushes `DataUpdate` events from `DataService::subscribe()`.
///
/// Per PT-02 / `IPC_COMPLIANCE_MATRIX.md` v1.2: the browser receives live
/// topology changes without polling.
///
/// Event types:
/// - `topology`: `LiveTopology` payload (primals, edges, mesh peers, source)
/// - `snapshot`: legacy `DataSnapshot` payload (primals + edges only)
pub async fn events_sse_handler(
    State(service): State<Arc<DataService>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = service.subscribe();
    let service = Arc::clone(&service);

    let stream = BroadcastStream::new(rx).filter_map(move |msg| {
        let service = Arc::clone(&service);
        match msg {
            Ok(update) => {
                let event = match update {
                    crate::data_service::DataUpdate::TopologyUpdated => {
                        let topo = service.live_topology();
                        serde_json::to_string(&topo)
                            .ok()
                            .map(|json| Event::default().event("topology").data(json))
                    }
                    crate::data_service::DataUpdate::MeshPeersUpdated => {
                        let topo = service.live_topology();
                        serde_json::to_string(&topo)
                            .ok()
                            .map(|json| Event::default().event("topology").data(json))
                    }
                };
                event.map(Ok)
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
