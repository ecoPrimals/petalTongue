// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordination backend handlers — reads CAS coordination data on shared filesystem.
//!
//! These handlers expose the coordination manifest (blurbs, waves, heads,
//! FRAGOs, topology, depot) via HTTP API. The data source is discovered
//! at runtime via `COORD_STORAGE_PATH` env or XDG defaults.
//!
//! Storage base resolution is shared with the CAS content pipeline
//! ([`crate::content_render::resolve_storage_base`]).

use axum::{Json, response::IntoResponse};

fn storage_base() -> std::path::PathBuf {
    crate::content_render::resolve_storage_base()
}

fn manifest_path() -> std::path::PathBuf {
    let family = std::env::var("FAMILY_ID").unwrap_or_else(|_| String::from("default"));
    storage_base()
        .join("datasets")
        .join(family)
        .join("_coordination")
        .join("manifest.json")
}

fn artifact_path(hash: &str) -> std::path::PathBuf {
    let family = std::env::var("FAMILY_ID").unwrap_or_else(|_| String::from("default"));
    storage_base()
        .join("datasets")
        .join(family)
        .join("_coordination")
        .join("artifacts")
        .join(hash)
}

fn load_manifest() -> serde_json::Value {
    let path = manifest_path();
    if !path.exists() {
        return serde_json::json!({
            "status": "no_data",
            "note": "No coordination data ingested yet. Run coord.ingest via content provider JSON-RPC.",
            "artifacts": {}, "heads": {}, "blurb_history": [], "frago_history": []
        });
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(
            || serde_json::json!({"status": "error", "note": "Failed to read manifest"}),
        )
}

/// Resolve depot directory path from environment or default.
fn depot_path() -> std::path::PathBuf {
    std::env::var("ECOP_DEPOT_PATH").map_or_else(
        |_| std::path::PathBuf::from("/opt/ecoPrimals/depot"),
        std::path::PathBuf::from,
    )
}

/// `GET /api/coord/blurbs` — current blurb + blurb history.
pub(super) async fn coord_blurbs_handler() -> impl IntoResponse {
    let manifest = load_manifest();
    let current = manifest["current_blurb"].as_str();
    let blurbs: Vec<&serde_json::Value> = manifest["blurb_history"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| h.as_str().and_then(|s| manifest["artifacts"].get(s)))
        .collect();

    let current_content = current.and_then(|h| std::fs::read_to_string(artifact_path(h)).ok());

    Json(serde_json::json!({
        "count": blurbs.len(),
        "current": current,
        "current_content": current_content,
        "blurbs": blurbs,
    }))
}

/// `GET /api/coord/waves` — current wave state + history.
pub(super) async fn coord_waves_handler() -> impl IntoResponse {
    let manifest = load_manifest();
    let current_hash = manifest["current_wave"].as_str();
    let current_content = current_hash.and_then(|h| std::fs::read_to_string(artifact_path(h)).ok());

    let history: Vec<serde_json::Value> = manifest["blurb_history"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| {
            let hash = h.as_str()?;
            let art = manifest["artifacts"].get(hash)?;
            Some(serde_json::json!({
                "wave": art["wave"],
                "hash": hash,
                "title": art["title"],
                "ingested_at": art["ingested_at"],
            }))
        })
        .collect();

    Json(serde_json::json!({
        "current_wave": current_hash,
        "current_content": current_content,
        "history": history,
    }))
}

/// `GET /api/coord/heads` — all gate HEAD states.
pub(super) async fn coord_heads_handler() -> impl IntoResponse {
    let manifest = load_manifest();
    let heads = manifest["heads"].as_object().cloned().unwrap_or_default();
    let entries: Vec<serde_json::Value> = heads
        .iter()
        .map(|(gate, hash_val)| {
            let hash = hash_val.as_str().unwrap_or("");
            serde_json::json!({
                "gate": gate,
                "hash": hash,
                "artifact": manifest["artifacts"].get(hash),
            })
        })
        .collect();

    Json(serde_json::json!({
        "count": entries.len(),
        "heads": entries,
    }))
}

/// `GET /api/coord/fragos` — FRAGO/AAR list.
pub(super) async fn coord_fragos_handler() -> impl IntoResponse {
    let manifest = load_manifest();
    let fragos: Vec<&serde_json::Value> = manifest["frago_history"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| h.as_str().and_then(|s| manifest["artifacts"].get(s)))
        .collect();

    Json(serde_json::json!({
        "count": fragos.len(),
        "fragos": fragos,
    }))
}

/// `GET /api/coord/topology` — mesh topology from coordination manifest.
pub(super) async fn coord_topology_handler() -> impl IntoResponse {
    let manifest = load_manifest();
    let gates: Vec<&str> = manifest["heads"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.keys().map(String::as_str))
        .collect();

    Json(serde_json::json!({
        "gates": gates,
        "head_count": gates.len(),
        "source": "coordination_manifest",
    }))
}

/// `GET /api/coord/depot` — depot binary inventory.
pub(super) async fn coord_depot_handler() -> impl IntoResponse {
    let path = depot_path();
    if !path.exists() {
        return Json(serde_json::json!({
            "status": "no_depot",
            "message": "No depot directory found"
        }));
    }
    let mut binaries = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() {
                let name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let meta = std::fs::metadata(&file_path).ok();
                binaries.push(serde_json::json!({
                    "name": name,
                    "size": meta.as_ref().map(std::fs::Metadata::len),
                }));
            }
        }
    }
    binaries.sort_by(|a, b| {
        let an = a["name"].as_str().unwrap_or("");
        let bn = b["name"].as_str().unwrap_or("");
        an.cmp(bn)
    });
    Json(serde_json::json!({
        "binary_count": binaries.len(),
        "binaries": binaries,
    }))
}
