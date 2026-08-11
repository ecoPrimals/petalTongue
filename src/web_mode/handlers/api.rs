// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::Duration;

use axum::{Json, extract::State, response::IntoResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use petal_tongue_core::transport::{TransportEndpoint, connect_transport};

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

const CONTENT_STATS_TIMEOUT: Duration = Duration::from_secs(3);

fn resolve_rhizocrypt_sock() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("BIOMEOS_RUNTIME_DIR") {
        return std::path::PathBuf::from(dir).join("rhizocrypt.sock");
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let p = std::path::PathBuf::from(xdg).join("biomeos/rhizocrypt.sock");
        if p.exists() {
            return p;
        }
    }
    std::path::PathBuf::from("/run/membrane/rhizocrypt.sock")
}

fn resolve_pseudospore_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("PSEUDOSPORE_BUNDLES") {
        return std::path::PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home)
            .join("Development/ecoPrimals/infra/sporePrint/static/pseudospore-bundles");
    }
    std::path::PathBuf::from("/var/lib/ecoPrimals/pseudospore-bundles")
}

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
    let sock = resolve_rhizocrypt_sock();
    let sock_str = sock.to_string_lossy();
    let sessions = rpc_query(&sock_str, "dag.session.list", serde_json::json!({})).await?;

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
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let size = session
            .get("size_bytes")
            .and_then(serde_json::Value::as_u64)
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
            rpc_query(&sock_str, "health.metrics", serde_json::json!({})).await
        {
            total_size = metrics
                .get("storage_bytes")
                .or_else(|| metrics.get("disk_usage"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            if total_objects == 0 {
                total_objects = metrics
                    .get("total_events")
                    .or_else(|| metrics.get("object_count"))
                    .and_then(serde_json::Value::as_u64)
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
    let dir = resolve_pseudospore_dir();
    let path = dir.as_path();

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
            let name = entry.file_name().to_string_lossy().into_owned();
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
                "is_dir": meta.as_ref().is_some_and(std::fs::Metadata::is_dir),
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

const FEDERATION_TIMEOUT: Duration = Duration::from_secs(5);

/// `/api/content/federation` — federated data braids view.
///
/// Combines local CAS stats (rhizoCrypt) with data-layer gossip entries
/// from swarmVine, providing a mesh-wide content availability picture.
/// As gates inject `cas.have` and `braid.head` gossip entries, they
/// appear here automatically — no SSH required.
pub async fn content_federation_handler() -> impl IntoResponse {
    let result = tokio::time::timeout(FEDERATION_TIMEOUT, build_federation_view()).await;

    match result {
        Ok(Ok(view)) => Json(view),
        Ok(Err(e)) => {
            tracing::debug!("federation view failed: {e}");
            Json(serde_json::json!({
                "status": "partial",
                "error": e.to_string(),
                "local": null,
                "gossip": null,
            }))
        }
        Err(_) => Json(serde_json::json!({
            "status": "timeout",
            "error": "federation query exceeded 5s",
        })),
    }
}

async fn build_federation_view(
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let local_fut = tokio::time::timeout(CONTENT_STATS_TIMEOUT, query_content_stats());
    let gossip_fut = tokio::time::timeout(CONTENT_STATS_TIMEOUT, query_swarmvine_data());

    let (local_result, gossip_result) = tokio::join!(local_fut, gossip_fut);

    let local = match local_result {
        Ok(Ok(v)) => v,
        _ => serde_json::json!({ "status": "unavailable" }),
    };

    let gossip = match gossip_result {
        Ok(Ok(v)) => v,
        _ => serde_json::json!({ "status": "unavailable" }),
    };

    let remote_count = gossip
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    let mut gates = vec!["sporeGate".to_string()];
    if let Some(entries) = gossip.get("entries").and_then(serde_json::Value::as_array) {
        for entry in entries {
            if let Some(gate) = entry.get("origin_gate").and_then(serde_json::Value::as_str) {
                let g = gate.to_string();
                if !gates.contains(&g) {
                    gates.push(g);
                }
            }
        }
    }

    Ok(serde_json::json!({
        "status": "ok",
        "local": local,
        "gossip": {
            "data_entries": gossip.get("count").and_then(serde_json::Value::as_u64).unwrap_or(0),
            "remote_gates": remote_count,
            "entries": gossip.get("entries"),
        },
        "federation": {
            "gates_visible": gates,
            "gate_count": gates.len(),
            "transport": "tower_atomic",
        },
    }))
}

/// Query swarmVine's data-topic gossip table for cross-gate CAS entries.
async fn query_swarmvine_data(
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let socket = discover_swarmvine_socket()
        .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
            "swarmVine socket not found".into()
        })?;

    let path_str = socket.to_string_lossy().into_owned();
    rpc_query(&path_str, "gossip.query", serde_json::json!({ "topic": "data" })).await
}

fn discover_swarmvine_socket() -> Option<std::path::PathBuf> {
    petal_tongue_core::gossip_injection::discover_swarmvine_socket()
}

async fn send_uds_raw(
    path: &str,
    payload: &[u8],
    btsp: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = TransportEndpoint::uds(path);
    let mut stream = connect_transport(&endpoint).await?;

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
