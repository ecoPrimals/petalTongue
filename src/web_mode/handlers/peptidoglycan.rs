// SPDX-License-Identifier: AGPL-3.0-or-later
//! Peptidoglycan layer handlers — nestgate.io Phase 2 + Phase 3 CAS.
//!
//! Serves depot browsing, provenance chain inspection, and federated CAS
//! content retrieval via HTTP.
//!
//! Data sources:
//! - Depot: local filesystem (`ECOP_DEPOT_PATH` or plasmidBin default)
//! - Provenance: per-architecture `BLAKE3SUMS` files (b3sum standard format)
//! - CAS: nestGate `content.get` via UDS JSON-RPC (Phase 3)
//! - Federation: songBird `content.locate` for cross-gate resolution (Phase 3)
//!
//! Routes:
//! - `GET /depot/`              — architecture overview
//! - `GET /depot/{arch}`        — binaries for architecture
//! - `GET /depot/{arch}/{name}` — single binary provenance
//! - `GET /provenance/`         — provenance chain overview
//! - `GET /provenance/{hash}`   — single object provenance tree
//! - `GET /cas/{hash}`          — content-addressed retrieval (Phase 3)
//! - `GET /cas/{hash}/provenance` — braid provenance for content (Phase 3)

use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn depot_base() -> PathBuf {
    std::env::var("ECOP_DEPOT_PATH").map_or_else(
        |_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/opt/ecoPrimals".to_owned());
            PathBuf::from(home)
                .join("Development/ecoPrimals/infra/plasmidBin/primals")
        },
        PathBuf::from,
    )
}

/// Load BLAKE3 checksums for a specific architecture from per-arch `BLAKE3SUMS`.
///
/// Format: `<hash>  <filename>` (b3sum standard output).
fn load_arch_checksums(arch: &str) -> BTreeMap<String, String> {
    let path = depot_base().join(arch).join("BLAKE3SUMS");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    content
        .lines()
        .filter_map(|line| {
            let (hash, name) = line.split_once("  ")?;
            Some((name.to_owned(), hash.to_owned()))
        })
        .collect()
}

/// Load all checksums across all architectures.
fn load_all_checksums() -> BTreeMap<String, BTreeMap<String, String>> {
    let base = depot_base();
    let Ok(entries) = std::fs::read_dir(&base) else {
        return BTreeMap::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            let checksums = load_arch_checksums(&name);
            if checksums.is_empty() {
                return None;
            }
            Some((name, checksums))
        })
        .collect()
}

/// `GET /depot/` — architecture overview.
pub(crate) async fn depot_index_handler() -> impl IntoResponse {
    let base = depot_base();
    let mut architectures = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if name.starts_with('.') {
                    continue;
                }
                let binary_count = std::fs::read_dir(&path)
                    .map(|rd| rd.flatten().filter(|e| {
                        let n = e.file_name().to_string_lossy().into_owned();
                        e.path().is_file() && !n.starts_with('.') && n != "BLAKE3SUMS"
                    }).count())
                    .unwrap_or(0);
                let total_size: u64 = std::fs::read_dir(&path)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|e| e.path().is_file())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum();
                architectures.push(serde_json::json!({
                    "target": name,
                    "binary_count": binary_count,
                    "total_size_bytes": total_size,
                    "total_size_human": human_size(total_size),
                }));
            }
        }
    }
    architectures.sort_by(|a, b| {
        let an = a["target"].as_str().unwrap_or("");
        let bn = b["target"].as_str().unwrap_or("");
        an.cmp(bn)
    });

    let all_checksums = load_all_checksums();
    let has_checksums = !all_checksums.is_empty();

    Json(serde_json::json!({
        "layer": "peptidoglycan",
        "surface": "nestgate.io",
        "architecture_count": architectures.len(),
        "architectures": architectures,
        "checksums_available": has_checksums,
        "provenance_architectures": all_checksums.keys().collect::<Vec<_>>(),
    }))
}

/// `GET /depot/{arch}` — binaries for a specific architecture.
pub(crate) async fn depot_arch_handler(Path(arch): Path<String>) -> impl IntoResponse {
    let dir = depot_base().join(&arch);
    if !dir.is_dir() {
        return Json(serde_json::json!({
            "error": "architecture_not_found",
            "target": arch,
            "available": list_architectures(),
        }));
    }

    let checksums = load_arch_checksums(&arch);
    let mut binaries = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name.starts_with('.') || name == "BLAKE3SUMS" {
                continue;
            }
            let meta = std::fs::metadata(&path).ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

            let blake3 = checksums.get(&name).cloned();

            binaries.push(serde_json::json!({
                "name": name,
                "size_bytes": size,
                "size_human": human_size(size),
                "blake3": blake3,
            }));
        }
    }
    binaries.sort_by(|a, b| {
        let an = a["name"].as_str().unwrap_or("");
        let bn = b["name"].as_str().unwrap_or("");
        an.cmp(bn)
    });

    Json(serde_json::json!({
        "target": arch,
        "binary_count": binaries.len(),
        "binaries": binaries,
    }))
}

/// `GET /depot/{arch}/{name}` — single binary provenance details.
pub(crate) async fn depot_binary_handler(
    Path((arch, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let path = depot_base().join(&arch).join(&name);
    if !path.is_file() {
        return Json(serde_json::json!({
            "error": "binary_not_found",
            "target": arch,
            "name": name,
        }));
    }

    let meta = std::fs::metadata(&path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    let checksums = load_arch_checksums(&arch);
    let blake3 = checksums.get(&name).cloned();

    Json(serde_json::json!({
        "name": name,
        "target": arch,
        "size_bytes": size,
        "size_human": human_size(size),
        "blake3": blake3,
        "modified_epoch": modified,
        "depot_url": format!("https://depot.primals.eco/primals/{arch}/{name}"),
        "verification": "b3sum --check BLAKE3SUMS",
    }))
}

/// `GET /provenance/` — provenance chain overview.
pub(crate) async fn provenance_index_handler() -> impl IntoResponse {
    let all_checksums = load_all_checksums();
    let mut arch_summaries = Vec::new();

    for (arch, entries) in &all_checksums {
        arch_summaries.push(serde_json::json!({
            "target": arch,
            "tracked_binaries": entries.len(),
        }));
    }
    arch_summaries.sort_by(|a, b| {
        let an = a["target"].as_str().unwrap_or("");
        let bn = b["target"].as_str().unwrap_or("");
        an.cmp(bn)
    });

    Json(serde_json::json!({
        "layer": "peptidoglycan",
        "surface": "nestgate.io",
        "provenance_source": "BLAKE3SUMS (per-architecture b3sum files)",
        "architecture_count": arch_summaries.len(),
        "architectures": arch_summaries,
        "phase": "Phase 2 — local provenance. Phase 3 adds federated CAS + content.locate mesh queries.",
    }))
}

/// `GET /provenance/{hash}` — lookup by BLAKE3 hash (prefix match supported).
pub(crate) async fn provenance_hash_handler(Path(hash): Path<String>) -> impl IntoResponse {
    let all_checksums = load_all_checksums();
    let mut matches = Vec::new();

    for (arch, entries) in &all_checksums {
        for (binary, checksum) in entries {
            if checksum == &hash || checksum.starts_with(&hash) {
                let path = depot_base().join(arch).join(binary);
                let size = std::fs::metadata(&path).ok().map(|m| m.len());
                matches.push(serde_json::json!({
                    "binary": binary,
                    "target": arch,
                    "blake3": checksum,
                    "size_bytes": size,
                    "depot_url": format!("https://depot.primals.eco/primals/{arch}/{binary}"),
                }));
            }
        }
    }

    if matches.is_empty() {
        Json(serde_json::json!({
            "error": "hash_not_found",
            "hash": hash,
            "note": "Phase 3 will query federated CAS across all gates via songBird content.locate",
        }))
    } else {
        Json(serde_json::json!({
            "hash": hash,
            "matches": matches,
            "match_count": matches.len(),
        }))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PHASE 3 — CAS Content Retrieval + Federated Provenance
// ═══════════════════════════════════════════════════════════════════════════════

const CAS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

fn resolve_nestgate_sock() -> PathBuf {
    let membrane = "/run/user/1000/membrane";
    if let Ok(entries) = std::fs::read_dir(membrane) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("nestgate-") && name.ends_with(".sock") {
                return entry.path();
            }
        }
    }
    PathBuf::from(format!("{membrane}/nestgate.sock"))
}

fn resolve_songbird_sock() -> PathBuf {
    let membrane = "/run/user/1000/membrane";
    if let Ok(entries) = std::fs::read_dir(membrane) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("songbird-") && name.ends_with(".sock") {
                return entry.path();
            }
        }
    }
    PathBuf::from(format!("{membrane}/songbird.sock"))
}

fn resolve_sweetgrass_sock() -> PathBuf {
    let membrane = "/run/user/1000/membrane";
    if let Ok(entries) = std::fs::read_dir(membrane) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("sweetgrass-") && name.ends_with(".sock") {
                return entry.path();
            }
        }
    }
    PathBuf::from(format!("{membrane}/sweetgrass.sock"))
}

/// `GET /cas/{hash}` — content-addressed retrieval via nestGate CAS.
///
/// Resolution order:
/// 1. Local nestGate `content.exists` → `content.get`
/// 2. songBird `content.locate` (mesh federation, scope: "all")
/// 3. Return 404 with federation status
pub(crate) async fn cas_content_handler(Path(hash): Path<String>) -> impl IntoResponse {
    use axum::http::StatusCode;
    use axum::response::Response;
    use axum::body::Body;

    let result = tokio::time::timeout(CAS_TIMEOUT, cas_resolve_content(&hash)).await;

    match result {
        Ok(Ok(content_result)) => {
            if let Some(data) = content_result.get("data").and_then(|d| d.as_str()) {
                let bytes = base64_decode_or_raw(data);
                let mime = content_result
                    .get("mime_type")
                    .and_then(|m| m.as_str())
                    .unwrap_or("application/octet-stream");
                let size = content_result
                    .get("size")
                    .and_then(|s| s.as_u64());

                let mut builder = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime)
                    .header("X-Content-Hash", &hash)
                    .header("X-Content-Source", content_result
                        .get("source")
                        .and_then(|s| s.as_str())
                        .unwrap_or("local"));

                if let Some(s) = size {
                    builder = builder.header("Content-Length", s.to_string());
                }

                builder
                    .body(Body::from(bytes))
                    .unwrap_or_else(|_| {
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("response build error"))
                            .unwrap()
                    })
            } else if content_result.get("exists") == Some(&serde_json::json!(true)) {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&content_result).unwrap_or_default()))
                    .unwrap()
            } else {
                let locations = content_result.get("locations").cloned();
                let body = serde_json::json!({
                    "error": "content_not_found",
                    "hash": hash,
                    "layer": "peptidoglycan",
                    "federation_locations": locations,
                    "note": "Content not found on local CAS or mesh federation"
                });
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
                    .unwrap()
            }
        }
        Ok(Err(e)) => {
            let body = serde_json::json!({
                "error": "cas_error",
                "hash": hash,
                "detail": e,
            });
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
                .unwrap()
        }
        Err(_) => {
            let body = serde_json::json!({
                "error": "timeout",
                "hash": hash,
            });
            Response::builder()
                .status(StatusCode::GATEWAY_TIMEOUT)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
                .unwrap()
        }
    }
}

/// `GET /cas/{hash}/provenance` — braid provenance for content hash.
pub(crate) async fn cas_provenance_handler(Path(hash): Path<String>) -> impl IntoResponse {
    let result = tokio::time::timeout(CAS_TIMEOUT, cas_resolve_provenance(&hash)).await;

    match result {
        Ok(Ok(prov)) => Json(serde_json::json!({
            "hash": hash,
            "layer": "peptidoglycan",
            "provenance": prov,
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "error": "provenance_unavailable",
            "hash": hash,
            "detail": e,
        })),
        Err(_) => Json(serde_json::json!({
            "error": "timeout",
            "hash": hash,
        })),
    }
}

async fn cas_resolve_content(hash: &str) -> Result<serde_json::Value, String> {
    let ng_sock = resolve_nestgate_sock();
    let ng_path = ng_sock.to_string_lossy();

    let exists = cas_rpc_query(&ng_path, "content.exists", serde_json::json!({ "hash": hash }))
        .await;

    if let Ok(ref result) = exists {
        if result.get("exists") == Some(&serde_json::json!(true)) {
            if let Ok(content) = cas_rpc_query(
                &ng_path,
                "content.get",
                serde_json::json!({ "hash": hash }),
            )
            .await
            {
                let mut response = content;
                response
                    .as_object_mut()
                    .map(|o| o.insert("source".to_string(), serde_json::json!("local")));
                return Ok(response);
            }
        }
    }

    let sb_sock = resolve_songbird_sock();
    let sb_path = sb_sock.to_string_lossy();

    let locate = cas_rpc_query(
        &sb_path,
        "content.locate",
        serde_json::json!({
            "hash": hash,
            "algorithm": "blake3",
            "scope": "all"
        }),
    )
    .await;

    match locate {
        Ok(loc_result) => {
            let locations = loc_result.get("locations").and_then(|l| l.as_array());
            if let Some(locs) = locations {
                if !locs.is_empty() {
                    let mut result = loc_result;
                    result.as_object_mut().map(|o| {
                        o.insert("exists".to_string(), serde_json::json!(true));
                        o.insert("source".to_string(), serde_json::json!("federation"))
                    });
                    return Ok(result);
                }
            }
            Ok(serde_json::json!({
                "exists": false,
                "hash": hash,
                "locations": [],
            }))
        }
        Err(e) => {
            tracing::debug!(hash, error = %e, "content.locate failed");
            Ok(serde_json::json!({
                "exists": false,
                "hash": hash,
                "locations": [],
                "locate_error": e,
            }))
        }
    }
}

async fn cas_resolve_provenance(hash: &str) -> Result<serde_json::Value, String> {
    let sg_sock = resolve_sweetgrass_sock();
    let sg_path = sg_sock.to_string_lossy();

    let braid = cas_rpc_query_ribocipher(
        &sg_path,
        "braid.get",
        serde_json::json!({ "data_hash": hash }),
    )
    .await;

    match braid {
        Ok(result) => Ok(result),
        Err(_) => {
            let verify = cas_rpc_query_ribocipher(
                &sg_path,
                "braid.verify",
                serde_json::json!({ "data_hash": hash }),
            )
            .await;
            verify.map_err(|e| format!("braid lookup failed: {e}"))
        }
    }
}

async fn cas_rpc_query(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let payload = serde_json::to_vec(&request).map_err(|e| e.to_string())?;

    cas_send_uds(socket_path, &payload, false)
        .await
        .map_err(|e| e.to_string())
}

async fn cas_rpc_query_ribocipher(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let payload = serde_json::to_vec(&request).map_err(|e| e.to_string())?;

    cas_send_uds(socket_path, &payload, true)
        .await
        .map_err(|e| e.to_string())
}

async fn cas_send_uds(
    path: &str,
    payload: &[u8],
    ribocipher: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let mut stream = UnixStream::connect(path).await?;

    if ribocipher {
        let mut frame = vec![0xEC, 0x01];
        frame.extend_from_slice(payload);
        stream.write_all(&frame).await?;
    } else {
        stream.write_all(payload).await?;
    }
    stream.shutdown().await?;

    let mut buf = Vec::with_capacity(65536);
    stream.read_to_end(&mut buf).await?;

    let json_start = if buf.len() >= 2 && buf[0] == 0xEC && buf[1] == 0x01 {
        2
    } else {
        0
    };

    let text = std::str::from_utf8(&buf[json_start..])?;
    let val: serde_json::Value = serde_json::from_str(text.trim())?;
    val.get("result")
        .cloned()
        .ok_or_else(|| {
            let err_msg = val
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("no result field");
            format!("RPC error: {err_msg}").into()
        })
}

fn base64_decode_or_raw(data: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap_or_else(|_| data.as_bytes().to_vec())
}

fn list_architectures() -> Vec<String> {
    let base = depot_base();
    std::fs::read_dir(&base)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') { None } else { Some(name) }
        })
        .collect()
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{size:.1}{unit}");
        }
        size /= 1024.0;
    }
    format!("{size:.1}TB")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_formatting() {
        assert_eq!(human_size(0), "0.0B");
        assert_eq!(human_size(512), "512.0B");
        assert_eq!(human_size(1024), "1.0KB");
        assert_eq!(human_size(1_048_576), "1.0MB");
        assert_eq!(human_size(16_979_000), "16.2MB");
    }

    #[test]
    fn depot_base_resolves() {
        let base = depot_base();
        assert!(base.to_string_lossy().contains("primals") || base.to_string_lossy().contains("depot"));
    }
}
