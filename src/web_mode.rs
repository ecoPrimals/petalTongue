// SPDX-License-Identifier: AGPL-3.0-or-later
//! Web mode - HTTP server with SSE push
//!
//! Pure Rust! Dependencies: axum, tower-http (100% Pure Rust)
//!
//! **IPC / PT-06:** This mode does not start the JSON-RPC Unix socket server
//! ([`petal_tongue_ipc::UnixSocketServer`]). Live updates use HTTP SSE only, not
//! `callback_tx` push over UDS.

use crate::error::AppError;
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

pub async fn run(
    bind: &str,
    scenario: Option<String>,
    workers: usize,
    data_service: Arc<DataService>,
) -> Result<(), AppError> {
    tracing::info!(
        bind,
        scenario = ?scenario,
        workers,
        "Starting web UI server (Pure Rust!)"
    );

    let addr: SocketAddr = bind
        .parse()
        .map_err(|e| AppError::Other(format!("Failed to parse bind address: {e}")))?;

    tracing::info!("✅ Using shared DataService (zero duplication!)");

    // Build router with shared state
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .route("/api/events", get(events_sse_handler))
        .nest_service("/static", ServeDir::new("web/static"))
        .with_state(data_service);

    tracing::info!("🌐 Web UI server listening on http://{}", addr);

    // Start server (fully concurrent!)
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Other(format!("Failed to bind to address: {e}")))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Other(format!("Web server error: {e}")))?;

    Ok(())
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
            .interval(std::time::Duration::from_secs(15))
            .text("keepalive"),
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

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
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("valid bind");
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn test_invalid_bind_address() {
        let result: Result<SocketAddr, _> = "not-an-address".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_address_default_format() {
        // Format used by main when config fallback: "0.0.0.0:{port}"
        let addr: SocketAddr = "0.0.0.0:3000".parse().expect("valid default bind");
        assert_eq!(addr.port(), 3000);
        assert!(addr.ip().is_unspecified());
    }

    #[test]
    fn test_bind_address_loopback_with_port() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("valid loopback");
        assert_eq!(addr.port(), 8080);
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
        let result = run("not-a-valid-address", None, 4, data_service).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parse") || err_msg.contains("bind"));
    }

    #[tokio::test]
    async fn test_run_empty_bind_address() {
        let data_service = Arc::new(DataService::new());
        let result = run("", None, 4, data_service).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_invalid_port() {
        let data_service = Arc::new(DataService::new());
        let result = run("127.0.0.1:999999", None, 4, data_service).await;
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
}
