// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::Duration;

use axum::{Json, extract::State, response::IntoResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

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

/// Default path for pseudoSpore data bundles (override with `PSEUDOSPORE_BUNDLES`).
const DEFAULT_PSEUDOSPORE_DIR: &str = "/home/sporegate/Development/ecoPrimals/infra/sporePrint/static/pseudospore-bundles";

const RHIZOCRYPT_SOCK: &str = "/run/membrane/rhizocrypt.sock";
const CONTENT_STATS_TIMEOUT: Duration = Duration::from_secs(3);

/// `/api/content/stats` — query rhizoCrypt for local CAS statistics.
///
/// Returns `{ object_count, total_size, namespaces }` for the dashboard
/// Data Braids section. Falls back to `{ status: "unavailable" }` if
/// rhizoCrypt is unreachable.
pub async fn content_stats_handler() -> impl IntoResponse {
    let result = tokio::time::timeout(CONTENT_STATS_TIMEOUT, query_content_stats()).await;

    match result {
        Ok(Ok(stats)) => Json(stats),
        Ok(Err(e)) => {
            tracing::debug!("content stats query failed: {e}");
            Json(serde_json::json!({
                "status": "unavailable",
                "error": e.to_string()
            }))
        }
        Err(_) => Json(serde_json::json!({
            "status": "unavailable",
            "error": "timeout"
        })),
    }
}

async fn query_content_stats(
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let sessions = rpc_query(RHIZOCRYPT_SOCK, "dag.session.list", serde_json::json!({})).await?;

    let session_list = sessions
        .get("sessions")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();

    let mut total_objects: u64 = 0;
    let mut total_size: u64 = 0;
    let mut namespaces = Vec::new();

    for session in &session_list {
        let name = session
            .get("name")
            .or_else(|| session.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let count = session
            .get("event_count")
            .or_else(|| session.get("vertex_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let size = session
            .get("size_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        total_objects += count;
        total_size += size;
        if count > 0 {
            namespaces.push(serde_json::json!({ "name": name, "count": count }));
        }
    }

    // If session.list returned no size info, try health.metrics for storage stats
    if total_size == 0 {
        if let Ok(metrics) =
            rpc_query(RHIZOCRYPT_SOCK, "health.metrics", serde_json::json!({})).await
        {
            total_size = metrics
                .get("storage_bytes")
                .or_else(|| metrics.get("disk_usage"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if total_objects == 0 {
                total_objects = metrics
                    .get("total_events")
                    .or_else(|| metrics.get("object_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            }
        }
    }

    Ok(serde_json::json!({
        "object_count": total_objects,
        "total_size": total_size,
        "namespaces": namespaces,
        "sessions": session_list.len(),
        "source": "rhizocrypt"
    }))
}

/// Send a JSON-RPC request to a UDS socket, trying BTSP framing then plain.
async fn rpc_query(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let payload = serde_json::to_vec(&request)?;

    // Try BTSP-framed first
    if let Ok(val) = send_uds_raw(socket_path, &payload, true).await {
        return Ok(val);
    }
    // Fallback to plain JSON-RPC
    send_uds_raw(socket_path, &payload, false).await
}

/// `/api/pseudospore/bundles` — list available pseudoSpore data bundles.
pub async fn pseudospore_bundles_handler() -> impl IntoResponse {
    let dir = std::env::var("PSEUDOSPORE_BUNDLES")
        .unwrap_or_else(|_| DEFAULT_PSEUDOSPORE_DIR.to_string());
    let path = std::path::Path::new(&dir);

    if !path.is_dir() {
        return Json(serde_json::json!({
            "status": "unavailable",
            "error": "pseudospore bundles directory not found",
            "path": dir,
        }));
    }

    let mut bundles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("pseudospore-") {
                continue;
            }
            let meta = entry.metadata().ok();
            let readme_path = entry.path().join("README.md");
            let description = std::fs::read_to_string(&readme_path)
                .ok()
                .and_then(|s| s.lines().find(|l| !l.starts_with('#') && !l.is_empty()).map(String::from));
            let has_provenance = entry.path().join("provenance").is_dir();
            let has_data = entry.path().join("data").is_dir();

            bundles.push(serde_json::json!({
                "name": name,
                "has_provenance": has_provenance,
                "has_data": has_data,
                "description": description,
                "is_dir": meta.as_ref().is_some_and(|m| m.is_dir()),
            }));
        }
    }
    bundles.sort_by(|a, b| {
        a.get("name").and_then(|v| v.as_str()).cmp(&b.get("name").and_then(|v| v.as_str()))
    });

    let has_validate = path.join("validate.sh").is_file();

    Json(serde_json::json!({
        "bundles": bundles,
        "count": bundles.len(),
        "has_validate_sh": has_validate,
        "source_dir": dir,
    }))
}

async fn send_uds_raw(
    path: &str,
    payload: &[u8],
    btsp: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(path).await?;

    if btsp {
        let mut frame = vec![0xEC, 0x01];
        frame.extend_from_slice(payload);
        stream.write_all(&frame).await?;
    } else {
        stream.write_all(payload).await?;
    }
    stream.shutdown().await?;

    let mut buf = Vec::with_capacity(8192);
    stream.read_to_end(&mut buf).await?;

    let json_start = if buf.len() >= 2 && buf[0] == 0xEC && buf[1] == 0x01 {
        2
    } else {
        0
    };

    let text = std::str::from_utf8(&buf[json_start..])?;
    let mut decoder = serde_json::Deserializer::from_str(text).into_iter::<serde_json::Value>();
    while let Some(Ok(val)) = decoder.next() {
        if let Some(result) = val.get("result") {
            return Ok(result.clone());
        }
    }
    let val: serde_json::Value = serde_json::from_str(text)?;
    val.get("result")
        .cloned()
        .ok_or_else(|| "no result field in response".into())
}
