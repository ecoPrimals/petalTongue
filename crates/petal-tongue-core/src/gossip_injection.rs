// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gossip injection — announces petalTongue events to the swarmVine mesh.
//!
//! petalTongue's gossip injection points (ant colony scouts):
//!
//! | Topic | Event | When |
//! |-------|-------|------|
//! | `viz` | `viz.session.started` | Visualization session begins serving |
//! | `viz` | `viz.session.stopped` | Session ends |
//! | `surface` | `surface.web.live` | Web server starts on a gate |
//! | `surface` | `surface.scene.streaming` | `/ws/scene` has active subscribers |
//! | `content` | `content.serve.available` | Content serving available for a namespace |
//!
//! ## Protocol
//!
//! swarmVine accepts `gossip.spread` JSON-RPC calls:
//! ```json
//! { "method": "gossip.spread", "params": { "topic": "viz", "key": "...", "value": {...}, "ttl": 300 } }
//! ```
//!
//! Injection is best-effort and non-blocking. If swarmVine is unavailable,
//! the primal continues operating — gossip is opportunistic, not critical path.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Resolves the runtime socket directories for this platform.
///
/// Checks `BIOMEOS_RUNTIME_DIR` env var first, then `XDG_RUNTIME_DIR/ecoPrimals`,
/// and falls back to `/run/membrane` (the biomeOS system-wide socket dir).
fn socket_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::with_capacity(3);

    if let Ok(dir) = std::env::var("BIOMEOS_RUNTIME_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join("ecoPrimals"));
    }
    dirs.push(PathBuf::from("/run/membrane"));
    dirs
}

/// Timeout for gossip injection RPC calls.
const INJECT_TIMEOUT: Duration = Duration::from_millis(500);

/// Default TTL for gossip entries (seconds).
const DEFAULT_TTL_SECS: u64 = 300;

/// A gossip entry to spread through the swarmVine mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipEntry {
    /// Topic namespace (e.g., "viz", "surface", "content").
    pub topic: String,
    /// Unique key within the topic (e.g., "petaltongue:viz:session-42").
    pub key: String,
    /// Payload value — arbitrary JSON.
    pub value: serde_json::Value,
    /// Time-to-live in seconds (entry expires from mesh after this).
    pub ttl_secs: u64,
}

impl GossipEntry {
    /// Create a new gossip entry with default TTL.
    #[must_use]
    pub fn new(topic: impl Into<String>, key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            topic: topic.into(),
            key: key.into(),
            value,
            ttl_secs: DEFAULT_TTL_SECS,
        }
    }

    /// Override the TTL.
    #[must_use]
    pub const fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }
}

/// Discovers the swarmVine JSON-RPC socket on this gate.
///
/// Scans standard socket directories for `swarmvine-*.sock` (excluding tarpc sockets).
#[must_use]
pub fn discover_swarmvine_socket() -> Option<PathBuf> {
    for dir in socket_search_dirs() {
        if let Some(found) = discover_swarmvine_socket_in(&dir) {
            return Some(found);
        }
    }
    None
}

/// Discovers swarmVine socket in a given directory (testable).
#[must_use]
pub fn discover_swarmvine_socket_in(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("swarmvine-")
            && name_str.ends_with(".sock")
            && !name_str.contains("tarpc")
        {
            return Some(entry.path());
        }
    }
    None
}

/// Inject a gossip entry into the swarmVine mesh (best-effort, non-blocking).
///
/// Returns `Ok(())` if the entry was successfully sent, or an error description
/// if swarmVine is unavailable. Callers should log but never fail on gossip errors.
///
/// # Example
///
/// ```ignore
/// let entry = GossipEntry::new("viz", "petaltongue:session:live", json!({
///     "gate": "eastGate",
///     "session_id": "demo-1",
///     "modality": "webgl",
/// }));
/// if let Err(e) = inject_gossip(&entry).await {
///     tracing::debug!("gossip inject skipped: {e}");
/// }
/// ```
pub async fn inject_gossip(entry: &GossipEntry) -> Result<(), GossipError> {
    let socket = discover_swarmvine_socket().ok_or(GossipError::SocketNotFound)?;
    inject_gossip_via(&socket, entry).await
}

/// Inject gossip through a specific socket path.
pub async fn inject_gossip_via(socket: &Path, entry: &GossipEntry) -> Result<(), GossipError> {
    let params = serde_json::json!({
        "topic": entry.topic,
        "key": entry.key,
        "value": entry.value,
        "ttl": entry.ttl_secs,
    });

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "gossip.spread",
        "params": params,
        "id": 1,
    });

    let payload = serde_json::to_vec(&request).map_err(|_| GossipError::SerializeError)?;
    let path_str = socket.to_string_lossy();

    let result = tokio::time::timeout(INJECT_TIMEOUT, async {
        let endpoint = crate::transport::TransportEndpoint::uds(&*path_str);
        let mut stream = crate::transport::connect_transport(&endpoint).await?;

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        stream.write_all(&payload).await?;
        stream.shutdown().await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;
        Ok::<_, std::io::Error>(response)
    })
    .await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(GossipError::IoError(e.to_string())),
        Err(_) => Err(GossipError::Timeout),
    }
}

/// Errors from gossip injection (non-fatal, always logged at debug level).
#[derive(Debug, thiserror::Error)]
pub enum GossipError {
    /// swarmVine socket not found in the standard directory.
    #[error("swarmVine socket not found")]
    SocketNotFound,
    /// Gossip injection timed out (swarmVine unresponsive).
    #[error("gossip inject timed out")]
    Timeout,
    /// IO error communicating with swarmVine.
    #[error("IO error: {0}")]
    IoError(String),
    /// Failed to serialize the gossip entry.
    #[error("serialization error")]
    SerializeError,
}

// ─── Convenience constructors for petalTongue-specific events ────────────────

/// Announce that a web surface is live on this gate.
#[must_use]
pub fn surface_web_live(gate: &str, bind_addr: &str) -> GossipEntry {
    GossipEntry::new(
        "surface",
        format!("petaltongue:web:{gate}"),
        serde_json::json!({
            "event": "surface.web.live",
            "gate": gate,
            "bind": bind_addr,
            "primal": "petalTongue",
            "capabilities": ["visualization", "content-serve", "scene-stream"],
        }),
    )
    .with_ttl(600)
}

/// Announce that scene streaming has active subscribers.
#[must_use]
pub fn scene_stream_active(gate: &str, session_id: &str, subscriber_count: usize) -> GossipEntry {
    GossipEntry::new(
        "viz",
        format!("petaltongue:scene:{gate}:{session_id}"),
        serde_json::json!({
            "event": "surface.scene.streaming",
            "gate": gate,
            "session_id": session_id,
            "subscribers": subscriber_count,
            "primal": "petalTongue",
        }),
    )
}

/// Announce that content serving is available for a namespace.
#[must_use]
pub fn content_serve_available(gate: &str, namespace: &str, object_count: u64) -> GossipEntry {
    GossipEntry::new(
        "content",
        format!("petaltongue:content:{gate}:{namespace}"),
        serde_json::json!({
            "event": "content.serve.available",
            "gate": gate,
            "namespace": namespace,
            "objects": object_count,
            "primal": "petalTongue",
        }),
    )
    .with_ttl(600)
}

/// Announce a visualization session lifecycle event.
#[must_use]
pub fn viz_session_event(gate: &str, session_id: &str, started: bool) -> GossipEntry {
    let event = if started {
        "viz.session.started"
    } else {
        "viz.session.stopped"
    };
    GossipEntry::new(
        "viz",
        format!("petaltongue:viz:{gate}:{session_id}"),
        serde_json::json!({
            "event": event,
            "gate": gate,
            "session_id": session_id,
            "primal": "petalTongue",
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gossip_entry_serialization() {
        let entry = GossipEntry::new("viz", "test-key", serde_json::json!({"hello": "world"}));
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["topic"], "viz");
        assert_eq!(json["key"], "test-key");
        assert_eq!(json["ttl_secs"], 300);
    }

    #[test]
    fn gossip_entry_custom_ttl() {
        let entry = GossipEntry::new("surface", "k", serde_json::json!(null)).with_ttl(60);
        assert_eq!(entry.ttl_secs, 60);
    }

    #[test]
    fn surface_web_live_entry() {
        let entry = surface_web_live("eastGate", "0.0.0.0:3000");
        assert_eq!(entry.topic, "surface");
        assert!(entry.key.contains("petaltongue:web:eastGate"));
        assert_eq!(entry.ttl_secs, 600);
        assert_eq!(entry.value["primal"], "petalTongue");
        assert_eq!(entry.value["bind"], "0.0.0.0:3000");
    }

    #[test]
    fn scene_stream_active_entry() {
        let entry = scene_stream_active("ironGate", "demo-1", 3);
        assert_eq!(entry.topic, "viz");
        assert!(entry.key.contains("petaltongue:scene:ironGate:demo-1"));
        assert_eq!(entry.value["subscribers"], 3);
    }

    #[test]
    fn content_serve_available_entry() {
        let entry = content_serve_available("westGate", "qcd", 42);
        assert_eq!(entry.topic, "content");
        assert_eq!(entry.value["namespace"], "qcd");
        assert_eq!(entry.value["objects"], 42);
    }

    #[test]
    fn viz_session_event_started() {
        let entry = viz_session_event("ironGate", "sess-1", true);
        assert_eq!(entry.value["event"], "viz.session.started");
    }

    #[test]
    fn viz_session_event_stopped() {
        let entry = viz_session_event("ironGate", "sess-1", false);
        assert_eq!(entry.value["event"], "viz.session.stopped");
    }

    #[test]
    fn discover_socket_in_empty_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(discover_swarmvine_socket_in(tmp.path()).is_none());
    }

    #[test]
    fn discover_socket_finds_json_rpc_socket() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("swarmvine-default.sock"), "").unwrap();
        std::fs::write(tmp.path().join("swarmvine-default.tarpc.sock"), "").unwrap();

        let found = discover_swarmvine_socket_in(tmp.path());
        assert!(found.is_some());
        let name = found.unwrap().file_name().unwrap().to_string_lossy().to_string();
        assert!(!name.contains("tarpc"));
        assert!(name.starts_with("swarmvine-"));
    }
}
