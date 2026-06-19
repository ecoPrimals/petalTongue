// SPDX-License-Identifier: AGPL-3.0-or-later

use super::handlers::*;

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::data_service::DataService;

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
// ── Liveness & Readiness handlers ──────────────────────────────────────

#[tokio::test]
async fn test_liveness_handler() {
    let resp = liveness_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "alive");
}

#[tokio::test]
async fn test_readiness_handler() {
    let resp = readiness_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "ready");
    assert_eq!(v["ready"], true);
    assert!(v["version"].as_str().is_some());
    assert!(v["primal"].as_str().is_some());
}
