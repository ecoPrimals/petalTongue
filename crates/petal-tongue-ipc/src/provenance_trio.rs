// SPDX-License-Identifier: AGPL-3.0-only
//! Provenance trio client for visualization session lineage.
//!
//! Connects petalTongue visualization sessions to the provenance trio:
//!
//! - **rhizoCrypt** (ephemeral DAG): Creates session vertices for active visualizations
//! - **sweetGrass** (attribution): Records data source contributions
//! - **loamSpine** (permanent ledger): Archives exported visualizations
//!
//! All primals are discovered by capability, not by name. If any primal in the
//! trio is unavailable, petalTongue continues operating without provenance
//! (primal sovereignty).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::hash::{Hash, Hasher};
use tracing::{debug, warn};

/// Provenance session tracking a visualization's lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceSession {
    /// Session identifier (matches `RenderSession` id)
    pub session_id: String,
    /// rhizoCrypt vertex ID (if registered)
    pub vertex_id: Option<String>,
    /// sweetGrass braid ID (if created)
    pub braid_id: Option<String>,
    /// loamSpine entry ID (if archived)
    pub spine_entry_id: Option<String>,
}

/// Client for the provenance trio, discovered by capability.
pub struct ProvenanceTrioClient {
    /// Socket path for ephemeral DAG primal (capability: `dag.session`)
    ephemeral_socket: Option<String>,
    /// Socket path for attribution primal (capability: `braid.create`)
    attribution_socket: Option<String>,
    /// Socket path for permanent ledger primal (capability: `spine.create`)
    ledger_socket: Option<String>,
    request_id: std::sync::atomic::AtomicU64,
}

impl ProvenanceTrioClient {
    /// Discover the provenance trio by capability.
    ///
    /// Scans runtime sockets for primals providing:
    /// - `dag.session` (rhizoCrypt or equivalent)
    /// - `braid.create` (sweetGrass or equivalent)
    /// - `spine.create` (loamSpine or equivalent)
    #[must_use]
    pub fn discover() -> Self {
        Self {
            ephemeral_socket: discover_capability_socket("dag.session"),
            attribution_socket: discover_capability_socket("braid.create"),
            ledger_socket: discover_capability_socket("spine.create"),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Create with explicit socket paths (for testing).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Option<String> cannot be used in const fn
    pub fn with_sockets(
        ephemeral: Option<String>,
        attribution: Option<String>,
        ledger: Option<String>,
    ) -> Self {
        Self {
            ephemeral_socket: ephemeral,
            attribution_socket: attribution,
            ledger_socket: ledger,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Check which trio members are available.
    #[must_use]
    pub const fn availability(&self) -> TrioAvailability {
        TrioAvailability {
            ephemeral_dag: self.ephemeral_socket.is_some(),
            attribution: self.attribution_socket.is_some(),
            permanent_ledger: self.ledger_socket.is_some(),
        }
    }

    /// Begin provenance tracking for a visualization session.
    ///
    /// Creates an ephemeral DAG vertex and a sweetGrass braid.
    /// Returns a `ProvenanceSession` with IDs for further tracking.
    pub async fn begin_session(
        &self,
        session_id: &str,
        title: &str,
        domain: Option<&str>,
    ) -> ProvenanceSession {
        let mut session = ProvenanceSession {
            session_id: session_id.to_string(),
            vertex_id: None,
            braid_id: None,
            spine_entry_id: None,
        };

        // Register with ephemeral DAG (rhizoCrypt or equivalent)
        if let Some(socket) = &self.ephemeral_socket {
            match self
                .send_rpc(
                    socket,
                    "dag.vertex.create",
                    json!({
                        "session_id": session_id,
                        "label": title,
                        "domain": domain,
                        "primal": "petaltongue",
                        "vertex_type": "visualization_session",
                    }),
                )
                .await
            {
                Ok(result) => {
                    session.vertex_id = result
                        .get("vertex_id")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    debug!(
                        "Provenance: created ephemeral vertex {:?}",
                        session.vertex_id
                    );
                }
                Err(e) => warn!("Provenance: ephemeral DAG unavailable ({e})"),
            }
        }

        // Create attribution braid (sweetGrass or equivalent)
        if let Some(socket) = &self.attribution_socket {
            match self
                .send_rpc(
                    socket,
                    "braid.create",
                    json!({
                        "title": format!("viz:{title}"),
                        "domain": domain.unwrap_or("visualization"),
                    }),
                )
                .await
            {
                Ok(result) => {
                    session.braid_id = result
                        .get("braid_id")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    debug!(
                        "Provenance: created attribution braid {:?}",
                        session.braid_id
                    );
                }
                Err(e) => warn!("Provenance: attribution unavailable ({e})"),
            }
        }

        session
    }

    /// Record a data source contribution to a provenance session.
    ///
    /// Called when a spring pushes data to a visualization session.
    pub async fn record_contribution(
        &self,
        session: &ProvenanceSession,
        source_primal: &str,
        data_id: &str,
        channel_type: &str,
    ) {
        if let (Some(socket), Some(braid_id)) = (&self.attribution_socket, &session.braid_id) {
            let _ = self
                .send_rpc(
                    socket,
                    "contribution.record",
                    json!({
                        "braid_id": braid_id,
                        "agent": source_primal,
                        "entity": data_id,
                        "activity": "data_binding",
                        "role": "data_source",
                        "metadata": { "channel_type": channel_type },
                    }),
                )
                .await
                .map_err(|e| warn!("Provenance: contribution record failed ({e})"));
        }
    }

    /// Archive a visualization session to permanent storage.
    ///
    /// Called when a visualization is exported (SVG, audio, etc.).
    pub async fn archive_session(
        &self,
        session: &mut ProvenanceSession,
        export_format: &str,
        export_data: &[u8],
    ) {
        if let Some(socket) = &self.ledger_socket {
            match self
                .send_rpc(
                    socket,
                    "entry.append",
                    json!({
                        "spine_id": "petaltongue:exports",
                        "content_type": export_format,
                        "content_hash": format!("{:x}", blake3_hash(export_data)),
                        "content_length": export_data.len(),
                        "session_id": session.session_id,
                        "vertex_id": session.vertex_id,
                        "braid_id": session.braid_id,
                    }),
                )
                .await
            {
                Ok(result) => {
                    session.spine_entry_id = result
                        .get("entry_id")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    debug!("Provenance: archived to spine {:?}", session.spine_entry_id);
                }
                Err(e) => warn!("Provenance: permanent ledger unavailable ({e})"),
            }
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    async fn send_rpc(&self, socket: &str, method: &str, params: Value) -> Result<Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = UnixStream::connect(socket)
            .await
            .map_err(|e| format!("connect {socket}: {e}"))?;

        let (reader, mut writer) = stream.into_split();

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_id(),
        });

        let mut payload = serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
        payload.push('\n');

        writer
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| format!("write: {e}"))?;

        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        buf.read_line(&mut line)
            .await
            .map_err(|e| format!("read: {e}"))?;

        let response: Value = serde_json::from_str(&line).map_err(|e| format!("parse: {e}"))?;

        if let Some(error) = response.get("error") {
            return Err(format!("RPC error: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| "no result in response".to_string())
    }
}

/// Availability of provenance trio members.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrioAvailability {
    /// Ephemeral DAG (rhizoCrypt) is available.
    pub ephemeral_dag: bool,
    /// Attribution (sweetGrass) is available.
    pub attribution: bool,
    /// Permanent ledger (loamSpine) is available.
    pub permanent_ledger: bool,
}

impl TrioAvailability {
    /// Check if full provenance tracking is available.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.ephemeral_dag && self.attribution && self.permanent_ledger
    }

    /// Check if any provenance tracking is available.
    #[must_use]
    pub const fn is_partial(&self) -> bool {
        self.ephemeral_dag || self.attribution || self.permanent_ledger
    }
}

/// Discover a socket providing a given capability.
///
/// Uses runtime scanning of `$XDG_RUNTIME_DIR/biomeos/` and `/tmp/` for sockets
/// that advertise the requested capability. Returns `None` if no provider found.
fn discover_capability_socket(capability: &str) -> Option<String> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());

    // Check environment override: <CAPABILITY_UPPER>_SOCKET
    let env_key = format!("{}_SOCKET", capability.replace('.', "_").to_uppercase());
    if let Ok(path) = std::env::var(&env_key)
        && std::path::Path::new(&path).exists()
    {
        return Some(path);
    }

    // Scan biomeos directory for capability providers
    let biomeos_dir = format!("{runtime_dir}/biomeos");
    if let Ok(entries) = std::fs::read_dir(&biomeos_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sock") {
                // Socket found; would need to query capability.list to confirm
                // For now, use naming convention as hint
                let name = path.file_stem().unwrap_or_default().to_string_lossy();
                if capability_matches_socket_name(capability, &name) {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

/// Heuristic: does a socket name suggest it provides a capability?
fn capability_matches_socket_name(capability: &str, socket_name: &str) -> bool {
    match capability {
        "dag.session" | "dag.vertex.create" => {
            socket_name.contains("rhizocrypt") || socket_name.contains("rhizo")
        }
        "braid.create" | "contribution.record" => {
            socket_name.contains("sweetgrass") || socket_name.contains("sweet")
        }
        "spine.create" | "entry.append" => {
            socket_name.contains("loamspine") || socket_name.contains("loam")
        }
        _ => false,
    }
}

/// Compute a BLAKE3 hash of data (returns first 8 bytes as u64 for display).
fn blake3_hash(data: &[u8]) -> u64 {
    // Simple hash without pulling in blake3 crate -- use std hash as placeholder
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_session_serialization() {
        let session = ProvenanceSession {
            session_id: "test-123".to_string(),
            vertex_id: Some("v-abc".to_string()),
            braid_id: Some("b-def".to_string()),
            spine_entry_id: None,
        };
        let json = serde_json::to_string(&session).expect("serialize");
        let restored: ProvenanceSession = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.session_id, "test-123");
        assert_eq!(restored.vertex_id.as_deref(), Some("v-abc"));
        assert!(restored.spine_entry_id.is_none());
    }

    #[test]
    fn trio_availability_full() {
        let a = TrioAvailability {
            ephemeral_dag: true,
            attribution: true,
            permanent_ledger: true,
        };
        assert!(a.is_full());
        assert!(a.is_partial());
    }

    #[test]
    fn trio_availability_partial() {
        let a = TrioAvailability {
            ephemeral_dag: true,
            attribution: false,
            permanent_ledger: false,
        };
        assert!(!a.is_full());
        assert!(a.is_partial());
    }

    #[test]
    fn trio_availability_none() {
        let a = TrioAvailability {
            ephemeral_dag: false,
            attribution: false,
            permanent_ledger: false,
        };
        assert!(!a.is_full());
        assert!(!a.is_partial());
    }

    #[test]
    fn discover_returns_none_without_sockets() {
        let client = ProvenanceTrioClient::discover();
        let availability = client.availability();
        // In test environment, no primals running
        assert!(!availability.is_full());
    }

    #[test]
    fn with_sockets_sets_availability() {
        let client = ProvenanceTrioClient::with_sockets(
            Some("/tmp/rhizo.sock".to_string()),
            None,
            Some("/tmp/loam.sock".to_string()),
        );
        let a = client.availability();
        assert!(a.ephemeral_dag);
        assert!(!a.attribution);
        assert!(a.permanent_ledger);
    }

    #[tokio::test]
    async fn begin_session_without_trio_returns_empty() {
        let client = ProvenanceTrioClient::with_sockets(None, None, None);
        let session = client.begin_session("s1", "Test", None).await;
        assert_eq!(session.session_id, "s1");
        assert!(session.vertex_id.is_none());
        assert!(session.braid_id.is_none());
    }

    #[tokio::test]
    async fn record_contribution_without_attribution_is_noop() {
        let client = ProvenanceTrioClient::with_sockets(None, None, None);
        let session = ProvenanceSession {
            session_id: "s1".to_string(),
            vertex_id: None,
            braid_id: None,
            spine_entry_id: None,
        };
        // Should not panic
        client
            .record_contribution(&session, "neuralspring", "data-1", "timeseries")
            .await;
    }

    #[tokio::test]
    async fn archive_session_without_ledger_is_noop() {
        let client = ProvenanceTrioClient::with_sockets(None, None, None);
        let mut session = ProvenanceSession {
            session_id: "s1".to_string(),
            vertex_id: None,
            braid_id: None,
            spine_entry_id: None,
        };
        client.archive_session(&mut session, "svg", b"<svg/>").await;
        assert!(session.spine_entry_id.is_none());
    }

    #[test]
    fn capability_matches_rhizocrypt() {
        assert!(capability_matches_socket_name(
            "dag.session",
            "rhizocrypt-nat0"
        ));
        assert!(capability_matches_socket_name(
            "dag.vertex.create",
            "rhizo-family1"
        ));
        assert!(!capability_matches_socket_name("dag.session", "songbird"));
    }

    #[test]
    fn capability_matches_sweetgrass() {
        assert!(capability_matches_socket_name(
            "braid.create",
            "sweetgrass-nat0"
        ));
        assert!(capability_matches_socket_name(
            "contribution.record",
            "sweet-family1"
        ));
        assert!(!capability_matches_socket_name("braid.create", "loamspine"));
    }

    #[test]
    fn capability_matches_loamspine() {
        assert!(capability_matches_socket_name(
            "spine.create",
            "loamspine-nat0"
        ));
        assert!(capability_matches_socket_name(
            "entry.append",
            "loam-family1"
        ));
        assert!(!capability_matches_socket_name(
            "spine.create",
            "rhizocrypt"
        ));
    }

    #[test]
    fn blake3_hash_deterministic() {
        let h1 = blake3_hash(b"test data");
        let h2 = blake3_hash(b"test data");
        assert_eq!(h1, h2);
    }

    #[test]
    fn blake3_hash_different_for_different_data() {
        let h1 = blake3_hash(b"data A");
        let h2 = blake3_hash(b"data B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn provenance_session_default_values() {
        let session = ProvenanceSession {
            session_id: String::new(),
            vertex_id: None,
            braid_id: None,
            spine_entry_id: None,
        };
        assert!(session.vertex_id.is_none());
        assert!(session.braid_id.is_none());
        assert!(session.spine_entry_id.is_none());
    }

    #[test]
    fn trio_availability_serialization() {
        let a = TrioAvailability {
            ephemeral_dag: true,
            attribution: false,
            permanent_ledger: true,
        };
        let json = serde_json::to_string(&a).expect("serialize");
        let restored: TrioAvailability = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.ephemeral_dag);
        assert!(!restored.attribution);
    }
}
