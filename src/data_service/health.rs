// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-primal health liveness queries via G66 transport abstraction.
//!
//! G65 primals enforce riboCipher transport signal (0xEC prefix).
//! Some primals (beardog) require a full BTSP handshake on their main
//! socket but offer a `-default.sock` for plaintext health checks.
//!
//! Strategy: try BTSP-framed query first; on connection reset / EOF,
//! retry with plain JSON-RPC (handles coralReef's G65 plain mode).
//!
//! **G72 evolution**: Primal endpoints are now discovered at runtime by scanning
//! the biomeOS socket directories. No hardcoded peer knowledge.

use petal_tongue_core::transport::{TransportEndpoint, connect_transport};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const QUERY_TIMEOUT: Duration = Duration::from_secs(3);

/// Directories to scan for primal sockets at runtime.
///
/// Resolved dynamically: `BIOMEOS_RUNTIME_DIR` (env override) → `XDG_RUNTIME_DIR/biomeos` →
/// `/run/membrane` (system-wide biomeOS).
fn socket_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::with_capacity(3);

    if let Ok(dir) = std::env::var("BIOMEOS_RUNTIME_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join("biomeos"));
    }
    dirs.push(PathBuf::from("/run/membrane"));
    dirs
}

/// Discovered primal endpoint for health probing.
#[derive(Debug)]
struct DiscoveredEndpoint {
    name: String,
    path: PathBuf,
}

/// Scan socket directories for primal sockets (`.sock` files).
///
/// Extracts primal name from socket filename. Prefers `-default.sock` variants
/// for health checks (avoids BTSP-only main sockets where alternatives exist).
fn discover_primal_sockets() -> Vec<DiscoveredEndpoint> {
    let mut endpoints = Vec::new();
    let mut seen_primals: std::collections::HashSet<String> = std::collections::HashSet::new();

    for dir in socket_search_dirs() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !is_candidate_socket(&path) {
                continue;
            }

            let name = extract_primal_name(&path);
            if name.is_empty() {
                continue;
            }

            let is_default = path
                .file_name()
                .map_or(false, |f| f.to_string_lossy().contains("-default"));

            if seen_primals.contains(&name) && !is_default {
                continue;
            }

            if is_default {
                endpoints.retain(|e: &DiscoveredEndpoint| e.name != name);
            }

            seen_primals.insert(name.clone());
            endpoints.push(DiscoveredEndpoint { name, path });
        }
    }

    endpoints
}

fn is_candidate_socket(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str());
    if ext != Some("sock") {
        return false;
    }
    let name = path.file_name().map(|f| f.to_string_lossy());
    let name = match name {
        Some(n) => n,
        None => return false,
    };
    // Skip tarpc sockets (binary RPC, not JSON-RPC health)
    !name.contains(".tarpc.") && !name.contains(".negotiate.")
}

fn extract_primal_name(path: &Path) -> String {
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
    let stem = match stem {
        Some(s) => s,
        None => return String::new(),
    };

    // Strip common suffixes: "-default", "-e8b62b6e" (family hash), "-nat0"
    let name = stem
        .split('-')
        .next()
        .unwrap_or(&stem);

    // Handle longer primal names with internal dashes (e.g., "sweetgrass", "loamspine")
    // by checking if the full stem without the known suffixes is a better name
    let cleaned = stem
        .trim_end_matches("-default")
        .trim_end_matches("-desktop-nucleus");

    // If the cleaned version has no hyphens or matches known patterns, use it
    // Otherwise, take just the first segment
    let candidate = if cleaned.contains('-') {
        // Check if remaining dashes are part of a family hash pattern (8 hex chars)
        let parts: Vec<&str> = cleaned.rsplitn(2, '-').collect();
        if parts.len() == 2 && parts[0].len() == 8 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
            parts[1].to_string()
        } else {
            cleaned.to_string()
        }
    } else {
        cleaned.to_string()
    };

    if candidate.is_empty() { name.to_string() } else { candidate }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrimalHealth {
    pub primal: String,
    pub alive: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

async fn query_health(primal: &str, endpoint: &TransportEndpoint) -> PrimalHealth {
    let result = tokio::time::timeout(QUERY_TIMEOUT, async {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 1
        });
        let payload = serde_json::to_vec(&request)?;

        let resp = send_rpc(endpoint, &payload, true).await;
        if let Ok(val) = resp {
            return Ok(val);
        }

        send_rpc(endpoint, &payload, false).await
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
            let alive = resp.get("alive").and_then(serde_json::Value::as_bool).unwrap_or(false)
                || resp
                    .get("status")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s == "alive" || s == "ok");
            let status = resp
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or(if alive { "alive" } else { "unknown" })
                .to_string();
            let version = resp
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from);

            PrimalHealth {
                primal: primal.to_string(),
                alive,
                status,
                version,
                error: None,
            }
        }
        Ok(Err(e)) => PrimalHealth {
            primal: primal.to_string(),
            alive: false,
            status: "error".to_string(),
            version: None,
            error: Some(e.to_string()),
        },
        Err(_) => PrimalHealth {
            primal: primal.to_string(),
            alive: false,
            status: "timeout".to_string(),
            version: None,
            error: Some("health query timed out".to_string()),
        },
    }
}

/// Send a JSON-RPC payload via transport abstraction, optionally with BTSP framing.
async fn send_rpc(
    endpoint: &TransportEndpoint,
    payload: &[u8],
    btsp: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = connect_transport(endpoint).await?;

    if btsp {
        let mut frame = vec![0xEC, 0x01];
        frame.extend_from_slice(payload);
        stream.write_all(&frame).await?;
    } else {
        stream.write_all(payload).await?;
    }
    stream.shutdown().await?;

    let mut buf = Vec::with_capacity(4096);
    stream.read_to_end(&mut buf).await?;

    let json_start = if buf.len() >= 2 && buf[0] == 0xEC && buf[1] == 0x01 {
        2
    } else {
        0
    };

    parse_first_result(&buf[json_start..])
}

fn parse_first_result(
    raw: &[u8],
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let text = std::str::from_utf8(raw)?;

    let mut decoder = serde_json::Deserializer::from_str(text).into_iter::<serde_json::Value>();

    while let Some(Ok(val)) = decoder.next() {
        if val.get("result").is_some() {
            return Ok(val["result"].clone());
        }
    }

    let val: serde_json::Value = serde_json::from_str(text)?;
    if let Some(result) = val.get("result") {
        return Ok(result.clone());
    }

    Err("no result field in response".into())
}

/// Query health.liveness on all discovered primals concurrently.
///
/// Scans biomeOS socket directories at call time — zero hardcoded peer knowledge.
pub async fn query_all_health() -> Vec<PrimalHealth> {
    let discovered = discover_primal_sockets();
    let mut set = tokio::task::JoinSet::new();

    for ep in discovered {
        let name = ep.name;
        let path_str = ep.path.to_string_lossy().into_owned();
        let endpoint = TransportEndpoint::uds(&path_str);
        set.spawn(async move { query_health(&name, &endpoint).await });
    }

    let mut results = Vec::with_capacity(set.len());
    while let Some(Ok(health)) = set.join_next().await {
        results.push(health);
    }
    results.sort_by(|a, b| a.primal.cmp(&b.primal));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_primal_name_simple() {
        let path = PathBuf::from("/run/membrane/sweetgrass.sock");
        assert_eq!(extract_primal_name(&path), "sweetgrass");
    }

    #[test]
    fn test_extract_primal_name_with_family_hash() {
        let path = PathBuf::from("/run/membrane/petaltongue-e8b62b6e.sock");
        assert_eq!(extract_primal_name(&path), "petaltongue");
    }

    #[test]
    fn test_extract_primal_name_default_suffix() {
        let path = PathBuf::from("/run/membrane/beardog-default.sock");
        assert_eq!(extract_primal_name(&path), "beardog");
    }

    #[test]
    fn test_extract_primal_name_desktop_nucleus() {
        let path = PathBuf::from("/run/membrane/songbird-desktop-nucleus.sock");
        assert_eq!(extract_primal_name(&path), "songbird");
    }

    #[test]
    fn test_is_candidate_socket_rejects_tarpc() {
        let path = PathBuf::from("/run/membrane/petaltongue.tarpc.sock");
        assert!(!is_candidate_socket(&path));
    }

    #[test]
    fn test_is_candidate_socket_rejects_negotiate() {
        let path = PathBuf::from("/run/membrane/petaltongue.negotiate.sock");
        assert!(!is_candidate_socket(&path));
    }

    #[test]
    fn test_is_candidate_socket_accepts_normal() {
        let path = PathBuf::from("/run/membrane/loamspine.sock");
        assert!(is_candidate_socket(&path));
    }

    #[test]
    fn test_discover_with_env_override() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        let tmp = TempDir::new().unwrap();
        let sock = tmp.path().join("testprimal.sock");
        std::fs::File::create(&sock).unwrap().write_all(b"").unwrap();

        let discovered = env_test_helpers::with_env_var(
            "BIOMEOS_RUNTIME_DIR",
            &tmp.path().to_string_lossy(),
            discover_primal_sockets,
        );

        assert!(discovered.iter().any(|e| e.name == "testprimal"));
    }

    #[test]
    fn test_default_socket_preferred() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        let tmp = TempDir::new().unwrap();
        std::fs::File::create(tmp.path().join("beardog.sock")).unwrap();
        std::fs::File::create(tmp.path().join("beardog-default.sock")).unwrap();

        let discovered = env_test_helpers::with_env_var(
            "BIOMEOS_RUNTIME_DIR",
            &tmp.path().to_string_lossy(),
            discover_primal_sockets,
        );

        let beardog: Vec<_> = discovered.iter().filter(|e| e.name == "beardog").collect();
        assert_eq!(beardog.len(), 1);
        assert!(beardog[0].path.to_string_lossy().contains("-default"));
    }
}
