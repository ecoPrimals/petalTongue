// SPDX-License-Identifier: AGPL-3.0-or-later
//! Web mode - HTTP server with SSE push
//!
//! Pure Rust! Dependencies: axum, tower-http (100% Pure Rust)
//!
//! **IPC / PT-06:** This mode does not start the JSON-RPC Unix socket server
//! ([`petal_tongue_ipc::UnixSocketServer`]). Live updates use HTTP SSE only, not
//! `callback_tx` push over UDS.

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
use tower_http::services::ServeDir;

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
    /// Number of tokio worker threads (currently logged only; runtime is pre-built).
    pub workers: usize,
}

/// Start the web UI HTTP server.
///
/// PT-1: When `docroot` is set with `backend = "filesystem"`, a
/// `tower_http::ServeDir` catch-all fallback serves arbitrary static files.
///
/// PT-13: When `backend = "nestgate"`, a content-addressed fallback queries
/// NestGate `content.resolve` via JSON-RPC over UDS for path resolution.
pub async fn run(
    bind: &str,
    scenario: Option<String>,
    docroot: Option<String>,
    backend: &str,
    workers: usize,
    data_service: Arc<DataService>,
) -> Result<(), AppError> {
    let cfg = WebConfig {
        bind,
        scenario,
        docroot,
        backend,
        workers,
    };
    run_with_config(cfg, data_service).await
}

async fn run_with_config(
    cfg: WebConfig<'_>,
    data_service: Arc<DataService>,
) -> Result<(), AppError> {
    tracing::info!(
        bind = cfg.bind,
        scenario = ?cfg.scenario,
        docroot = ?cfg.docroot,
        backend = cfg.backend,
        workers = cfg.workers,
        "Starting web UI server (Pure Rust!)"
    );

    let addr: SocketAddr = cfg
        .bind
        .parse()
        .map_err(|e| AppError::Other(format!("Failed to parse bind address: {e}")))?;

    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .route("/api/events", get(events_sse_handler))
        .nest_service("/static", ServeDir::new(WEB_STATIC_DIR));

    match cfg.backend {
        "nestgate" => {
            let client = Arc::new(NestGateContentClient::from_env());
            tracing::info!(
                socket = %client.socket_path.display(),
                "🗄️  NestGate content-addressed backend active (PT-13)"
            );
            app = app.fallback(move |req: axum::extract::Request| {
                nestgate_fallback(req, Arc::clone(&client))
            });
        }
        _ => {
            if let Some(ref docroot) = cfg.docroot {
                let docroot_path = std::path::Path::new(docroot);
                if !docroot_path.is_dir() {
                    return Err(AppError::Other(format!(
                        "--docroot path does not exist or is not a directory: {docroot}"
                    )));
                }
                tracing::info!(
                    docroot,
                    "📂 Serving static files from docroot (PT-1 catch-all)"
                );
                app = app.fallback_service(
                    ServeDir::new(docroot).append_index_html_on_directories(true),
                );
            }
        }
    }

    let app = app.with_state(data_service);

    tracing::info!("🌐 Web UI server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Other(format!("Failed to bind to address: {e}")))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Other(format!("Web server error: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// PT-13: NestGate content-addressed backend
// ---------------------------------------------------------------------------

/// JSON-RPC client for NestGate `content.resolve` / `content.get`.
///
/// Socket discovery follows the ecosystem convention:
/// `NESTGATE_SOCKET` env → `$BIOMEOS_SOCKET_DIR/nestgate-{family}.sock`
/// → `$XDG_RUNTIME_DIR/biomeos/nestgate-default.sock`.
struct NestGateContentClient {
    socket_path: std::path::PathBuf,
    request_id: std::sync::atomic::AtomicU64,
}

impl NestGateContentClient {
    /// Resolve NestGate socket from the environment.
    fn from_env() -> Self {
        let socket_path = std::env::var("NESTGATE_SOCKET").map_or_else(
            |_| {
                let family = std::env::var("FAMILY_ID")
                    .or_else(|_| std::env::var("PETALTONGUE_FAMILY_ID"))
                    .unwrap_or_else(|_| "default".to_owned());
                let dir = std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {
                    let xdg =
                        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
                    let seg = petal_tongue_core::constants::ecosystem_runtime_dir_name();
                    format!("{xdg}/{seg}")
                });
                std::path::PathBuf::from(format!("{dir}/nestgate-{family}.sock"))
            },
            std::path::PathBuf::from,
        );
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Call `content.resolve` — returns `(content_bytes, mime_type)` or `None`.
    async fn resolve(&self, path: &str) -> Result<Option<(Vec<u8>, String)>, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("NestGate connect({}): {e}", self.socket_path.display()))?;

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "content.resolve",
            "params": { "path": path },
            "id": self.next_id(),
        });
        let mut line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        line.push('\n');
        stream
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("NestGate write: {e}"))?;
        stream
            .flush()
            .await
            .map_err(|e| format!("NestGate flush: {e}"))?;

        let (reader, _) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(reader);
        let mut resp_line = String::new();
        reader
            .read_line(&mut resp_line)
            .await
            .map_err(|e| format!("NestGate read: {e}"))?;

        let resp: serde_json::Value =
            serde_json::from_str(&resp_line).map_err(|e| format!("NestGate parse: {e}"))?;

        if resp.get("error").is_some() {
            return Ok(None);
        }

        let result = resp.get("result").ok_or("NestGate: no result field")?;
        let content_b64 = result
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("NestGate: missing content field")?;
        let mime = result
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream");

        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(content_b64)
            .map_err(|e| format!("NestGate base64 decode: {e}"))?;

        Ok(Some((bytes, mime.to_owned())))
    }
}

/// Axum fallback handler that resolves content via NestGate.
async fn nestgate_fallback(
    req: axum::extract::Request,
    client: Arc<NestGateContentClient>,
) -> axum::response::Response {
    let path = req.uri().path().to_owned();
    match client.resolve(&path).await {
        Ok(Some((body, mime))) => axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, mime)
            .body(axum::body::Body::from(body))
            .unwrap_or_else(|_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "response build error",
                )
                    .into_response()
            }),
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
        Err(e) => {
            tracing::error!(path, error = %e, "NestGate content.resolve failed");
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("NestGate backend unavailable: {e}"),
            )
                .into_response()
        }
    }
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok"
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
    // Get real data from unified service!
    match service.snapshot().await {
        Ok(snapshot) => Json(serde_json::json!({
            "primals": snapshot.primals,
            "timestamp": snapshot.timestamp,
        })),
        Err(e) => {
            // GraphLockPoisoned often indicates test-induced state; use debug to avoid noisy test output
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
/// topology changes without polling.  Each SSE frame carries a full
/// `DataSnapshot` serialised as JSON so the client can replace its local state.
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

    #[tokio::test]
    async fn test_run_invalid_bind_address() {
        let data_service = Arc::new(DataService::new());
        let result = run(
            "not-a-valid-address",
            None,
            None,
            "filesystem",
            4,
            data_service,
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parse") || err_msg.contains("bind"));
    }

    #[tokio::test]
    async fn test_run_empty_bind_address() {
        let data_service = Arc::new(DataService::new());
        let result = run("", None, None, "filesystem", 4, data_service).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_invalid_port() {
        let data_service = Arc::new(DataService::new());
        let result = run(
            "127.0.0.1:999999",
            None,
            None,
            "filesystem",
            4,
            data_service,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primals_handler_snapshot_error() {
        // Use a dedicated DataService and poison its lock to verify error handling.
        // Run in single-threaded context to avoid poisoning affecting other tests.
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
        // Use a dedicated DataService and poison its lock to verify error handling.
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
        let result = run(
            "127.0.0.1:0",
            None,
            Some("/nonexistent/docroot/path".to_string()),
            "filesystem",
            4,
            data_service,
        )
        .await;
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
        let resp = nestgate_fallback(req, client).await;
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
        let data_service = Arc::new(DataService::new());
        let app = Router::new()
            .route("/health", get(health_handler))
            .fallback(move |req: axum::extract::Request| {
                nestgate_fallback(req, Arc::clone(&client_clone))
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
}
