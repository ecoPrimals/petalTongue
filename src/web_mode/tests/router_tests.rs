// SPDX-License-Identifier: AGPL-3.0-or-later

use super::handlers::*;

use std::sync::Arc;

use axum::{Router, routing::get};

use crate::data_service::DataService;

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
