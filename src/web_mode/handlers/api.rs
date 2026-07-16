// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};

use crate::data_service::DataService;

pub async fn status_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
        "pure_rust": true,
        "neural_api": service.has_neural_api(),
    }))
}

pub async fn primals_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
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

pub async fn snapshot_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
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
