// SPDX-License-Identifier: AGPL-3.0-or-later
//! Peptidoglycan layer handlers — nestgate.io Phase 2.
//!
//! Serves depot browsing and provenance chain inspection via HTTP.
//! Data sources:
//! - Depot: local filesystem (`ECOP_DEPOT_PATH` or plasmidBin default)
//! - Provenance: per-architecture `BLAKE3SUMS` files (b3sum standard format)
//!
//! Routes:
//! - `GET /depot/`              — architecture overview
//! - `GET /depot/{arch}`        — binaries for architecture
//! - `GET /depot/{arch}/{name}` — single binary provenance
//! - `GET /provenance/`         — provenance chain overview
//! - `GET /provenance/{hash}`   — single object provenance tree

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
