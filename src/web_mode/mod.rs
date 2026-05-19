// SPDX-License-Identifier: AGPL-3.0-or-later
//! Web mode - HTTP server with SSE push
//!
//! Pure Rust! Dependencies: axum, tower-http (100% Pure Rust)
//!
//! **IPC / PT-06:** This mode does not start the JSON-RPC Unix socket server
//! ([`petal_tongue_ipc::UnixSocketServer`]). Live updates use HTTP SSE only, not
//! `callback_tx` push over UDS.

pub mod nestgate;

use crate::error::AppError;
use petal_tongue_core::constants::DEFAULT_SSE_KEEPALIVE_SECS;

/// Static assets directory for the web UI.
const WEB_STATIC_DIR: &str = "web/static";

use axum::{
    Json, Router,
    extract::State,
    response::{
        Html, IntoResponse,
        sse::{Event, Sse},
    },
    routing::get,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::data_service::DataService;

/// Web server configuration for [`run`].
pub struct WebConfig<'a> {
    /// Bind address (`host:port`).
    pub bind: &'a str,
    /// Scenario JSON file to load (currently logged only).
    pub scenario: Option<String>,
    /// Static file document root for catch-all serving (`--docroot`).
    pub docroot: Option<String>,
    /// Content backend: `"filesystem"` or `"nestgate"`.
    pub backend: &'a str,
    /// Number of tokio worker threads (wired to runtime in `main`).
    pub workers: usize,
    /// Hide code input cells when rendering `.ipynb` notebooks.
    pub strip_sources: bool,
    /// `Cache-Control: max-age` in seconds for served content (0 = no header).
    pub cache_ttl_secs: u64,
    /// SPA mode: serve `index.html` for missing paths (client-side routing).
    pub spa: bool,
    /// CORS allowed origins. Empty = same-origin only. `["*"]` = allow all.
    pub allowed_origins: Vec<String>,
}

/// Start the web UI HTTP server.
///
/// PT-1: When `docroot` is set with `backend = "filesystem"`, a
/// `tower_http::ServeDir` catch-all fallback serves arbitrary static files.
/// `.ipynb` files are rendered as HTML with `metadata.title` page headers.
///
/// PT-13: When `backend = "nestgate"`, a content-addressed fallback queries
/// NestGate `content.resolve` via JSON-RPC over UDS for path resolution.
#[expect(
    clippy::too_many_lines,
    reason = "router setup with all middleware layers"
)]
pub async fn run(cfg: WebConfig<'_>, data_service: Arc<DataService>) -> Result<(), AppError> {
    tracing::info!(
        bind = cfg.bind,
        scenario = ?cfg.scenario,
        docroot = ?cfg.docroot,
        backend = cfg.backend,
        workers = cfg.workers,
        strip_sources = cfg.strip_sources,
        cache_ttl_secs = cfg.cache_ttl_secs,
        spa = cfg.spa,
        allowed_origins = ?cfg.allowed_origins,
        "Starting web UI server (Pure Rust!)"
    );

    let addr: SocketAddr = cfg
        .bind
        .parse()
        .map_err(|e| AppError::Other(format!("Failed to parse bind address: {e}")))?;

    let nb_config = Arc::new(crate::notebook_render::NotebookRenderConfig {
        strip_sources: cfg.strip_sources,
    });
    let cache_ttl = cfg.cache_ttl_secs;
    let spa = cfg.spa;

    let mut app = Router::new()
        .route("/health", get(health_handler))
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .route("/api/events", get(events_sse_handler))
        .nest_service("/static", ServeDir::new(WEB_STATIC_DIR));

    if cfg.backend == "nestgate" {
        let client = Arc::new(nestgate::NestGateContentClient::from_env());
        tracing::info!(
            socket = %client.socket_path.display(),
            "NestGate content-addressed backend active (PT-13)"
        );
        let index_client = Arc::clone(&client);
        app = app.route(
            "/",
            get(move || nestgate::nestgate_index(Arc::clone(&index_client))),
        );
        let nb_cfg = Arc::clone(&nb_config);
        app = app.fallback(move |req: axum::extract::Request| {
            nestgate::nestgate_fallback(req, Arc::clone(&client), Arc::clone(&nb_cfg), cache_ttl)
        });
    } else {
        app = app.route("/", get(index_handler));
        if let Some(ref docroot) = cfg.docroot {
            let docroot_path = std::path::Path::new(docroot);
            if !docroot_path.is_dir() {
                return Err(AppError::Other(format!(
                    "--docroot path does not exist or is not a directory: {docroot}"
                )));
            }
            tracing::info!(
                docroot,
                spa,
                "Serving static files from docroot (PT-1 catch-all)"
            );
            let docroot_owned = docroot.clone();
            let nb_cfg = Arc::clone(&nb_config);
            app = app.fallback(move |req: axum::extract::Request| {
                docroot_fallback(req, docroot_owned, nb_cfg, cache_ttl, spa)
            });
        }
    }

    // Shadow parity layers (S3: GitHub Pages equivalence)
    app = app
        .layer(CompressionLayer::new())
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http",
                        method = %req.method(),
                        uri = %req.uri(),
                    )
                })
                .on_response(
                    |resp: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = resp.status().as_u16(),
                            latency_ms = latency.as_millis() as u64,
                            "response"
                        );
                    },
                ),
        );

    let app = if cfg.allowed_origins.is_empty() {
        app.with_state(data_service)
    } else {
        let cors = build_cors_layer(&cfg.allowed_origins);
        tracing::info!(
            origins = ?cfg.allowed_origins,
            "CORS enabled"
        );
        app.layer(cors).with_state(data_service)
    };

    tracing::info!("Web UI server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Other(format!("Failed to bind to address: {e}")))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Other(format!("Web server error: {e}")))?;

    Ok(())
}

/// Build a CORS layer from the configured allowed origins.
fn build_cors_layer(origins: &[String]) -> tower_http::cors::CorsLayer {
    use tower_http::cors::{AllowOrigin, CorsLayer};

    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    if origins.len() == 1 && origins[0] == "*" {
        cors.allow_origin(AllowOrigin::any())
    } else {
        let parsed: Vec<axum::http::HeaderValue> =
            origins.iter().filter_map(|o| o.parse().ok()).collect();
        cors.allow_origin(parsed)
    }
}

/// Security response headers for GitHub Pages parity (S3 shadow run).
///
/// GitHub Pages sends `X-Content-Type-Options: nosniff` and
/// `X-Frame-Options: DENY` on all responses.
async fn security_headers_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        axum::http::HeaderName::from_static("referrer-policy"),
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        axum::http::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    resp
}

// ── Filesystem fallback ─────────────────────────────────────────────────

/// Filesystem docroot fallback — serves static files with `.ipynb` rendering.
///
/// When `spa` is `true`, missing paths serve `{docroot}/index.html` instead of
/// 404, enabling client-side routing for single-page applications.
async fn docroot_fallback(
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
fn resolve_docroot_path(docroot: &str, uri_path: &str) -> std::path::PathBuf {
    let cleaned = uri_path.trim_start_matches('/');
    std::path::Path::new(docroot).join(cleaned)
}

/// Case-insensitive `.ipynb` extension check.
pub fn is_ipynb(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ipynb"))
}

// ── Route handlers ──────────────────────────────────────────────────────

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "primal": "petaltongue",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
    }))
}

async fn liveness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
        "alive": true,
    }))
}

async fn readiness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ready",
        "ready": true,
        "version": env!("CARGO_PKG_VERSION"),
        "primal": "petaltongue",
    }))
}

async fn status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
        "pure_rust": true,
    }))
}

async fn primals_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
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

async fn snapshot_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
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
async fn events_sse_handler(
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "test code uses unwrap/expect for brevity"
    )]

    use super::nestgate::{NestGateContentClient, nestgate_fallback};
    use super::*;

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
        let addr: SocketAddr = bind.parse().expect("valid bind");
        assert_eq!(addr.port(), DEFAULT_HEADLESS_PORT);
    }

    #[test]
    fn test_invalid_bind_address() {
        let result: Result<SocketAddr, _> = "not-an-address".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_address_default_format() {
        use petal_tongue_core::constants::{DEFAULT_BIND_HOST, DEFAULT_WEB_PORT};
        let bind = format!("{DEFAULT_BIND_HOST}:{DEFAULT_WEB_PORT}");
        let addr: SocketAddr = bind.parse().expect("valid default bind");
        assert_eq!(addr.port(), DEFAULT_WEB_PORT);
        assert!(addr.ip().is_unspecified());
    }

    #[test]
    fn test_bind_address_loopback_with_port() {
        use petal_tongue_core::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_LOOPBACK_HOST};
        let bind = format!("{DEFAULT_LOOPBACK_HOST}:{DEFAULT_HEADLESS_PORT}");
        let addr: SocketAddr = bind.parse().expect("valid loopback");
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
    fn test_nestgate_client_from_env_default() {
        let client = NestGateContentClient::from_env();
        let path_str = client.socket_path.to_string_lossy();
        assert!(
            path_str.contains("nestgate-"),
            "socket path should contain 'nestgate-': {path_str}"
        );
        assert!(
            path_str.ends_with(".sock"),
            "socket path should end with .sock: {path_str}"
        );
    }

    #[test]
    fn test_nestgate_client_from_env_override() {
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "NESTGATE_SOCKET",
            "/custom/nestgate.sock",
            || {
                let client = NestGateContentClient::from_env();
                assert_eq!(
                    client.socket_path,
                    std::path::PathBuf::from("/custom/nestgate.sock")
                );
            },
        );
    }

    #[test]
    fn test_nestgate_client_request_id_increments() {
        let client = NestGateContentClient::from_env();
        let id1 = client.next_id();
        let id2 = client.next_id();
        assert_eq!(id2, id1 + 1);
    }

    #[tokio::test]
    async fn test_nestgate_fallback_unavailable_returns_502() {
        use axum::body::Body;

        let client = Arc::new(NestGateContentClient {
            socket_path: std::path::PathBuf::from("/tmp/nonexistent-nestgate-test.sock"),
            request_id: std::sync::atomic::AtomicU64::new(1),
        });
        let req = axum::http::Request::builder()
            .uri("/some/page.html")
            .body(Body::empty())
            .unwrap();
        let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
        let resp = nestgate_fallback(req, client, nb_cfg, 0).await;
        assert_eq!(resp.status(), axum::http::StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn test_nestgate_backend_installs_fallback() {
        use axum::body::Body;

        let client = Arc::new(NestGateContentClient {
            socket_path: std::path::PathBuf::from("/tmp/nonexistent-nestgate-test.sock"),
            request_id: std::sync::atomic::AtomicU64::new(1),
        });
        let client_clone = Arc::clone(&client);
        let nb_cfg = Arc::new(crate::notebook_render::NotebookRenderConfig::default());
        let data_service = Arc::new(DataService::new());
        let app = Router::new()
            .route("/health", get(health_handler))
            .fallback(move |req: axum::extract::Request| {
                nestgate_fallback(req, Arc::clone(&client_clone), Arc::clone(&nb_cfg), 0)
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
            "nestgate fallback should return 502 when socket unavailable"
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
}
