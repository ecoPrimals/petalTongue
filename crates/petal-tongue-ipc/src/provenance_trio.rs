// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance trio client for visualization session lineage.
//!
//! Connects petalTongue visualization sessions to the provenance trio:
//!
//! - **ephemeral DAG** (`dag.session`): Creates session vertices for active visualizations
//! - **attribution** (`braid.create`): Records data source contributions
//! - **permanent ledger** (`spine.create`): Archives exported visualizations
//!
//! All primals are discovered by capability, not by name. If any primal in the
//! trio is unavailable, petalTongue continues operating without provenance
//! (primal sovereignty).

use petal_tongue_core::capability_names::primal_names;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use tracing::{debug, warn};

/// Error during a provenance trio RPC call.
#[derive(Debug, Error)]
pub(crate) enum ProvenanceRpcError {
    /// Failed to connect to the provenance socket.
    #[error("connect {path}: {source}")]
    Connect {
        path: String,
        source: std::io::Error,
    },
    /// JSON serialization failed.
    #[error("serialize: {0}")]
    Serialize(serde_json::Error),
    /// I/O error during request/response.
    #[error("{context}: {source}")]
    Io {
        context: &'static str,
        source: std::io::Error,
    },
    /// Failed to parse the JSON-RPC response.
    #[error("parse: {0}")]
    Parse(serde_json::Error),
    /// The provider returned a JSON-RPC error object.
    #[error("RPC error: {0}")]
    RpcError(Value),
    /// The provider response contained no `result` field.
    #[error("no result in response")]
    NoResult,
}

/// Provenance session tracking a visualization's lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceSession {
    /// Session identifier (matches `RenderSession` id)
    pub session_id: String,
    /// Ephemeral DAG vertex ID (if registered via `dag.session`)
    pub vertex_id: Option<String>,
    /// Attribution braid ID (if created via `braid.create`)
    pub braid_id: Option<String>,
    /// Permanent ledger entry ID (if archived via `spine.create`)
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
    /// - `dag.session` (ephemeral DAG provider)
    /// - `braid.create` (attribution provider)
    /// - `spine.create` (permanent ledger provider)
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
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Option<String> cannot be const"
    )]
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
    /// Creates an ephemeral DAG vertex and an attribution braid.
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

        // Register with ephemeral DAG provider (capability: dag.session)
        if let Some(socket) = &self.ephemeral_socket {
            match self
                .send_rpc(
                    socket,
                    "dag.vertex.create",
                    json!({
                        "session_id": session_id,
                        "label": title,
                        "domain": domain,
                        "primal": primal_names::PETALTONGUE,
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

        // Create attribution braid (capability: braid.create)
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

    async fn send_rpc(
        &self,
        socket: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, ProvenanceRpcError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream =
            UnixStream::connect(socket)
                .await
                .map_err(|e| ProvenanceRpcError::Connect {
                    path: socket.to_owned(),
                    source: e,
                })?;

        let (reader, mut writer) = stream.into_split();

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_id(),
        });

        let mut buf = serde_json::to_vec(&request).map_err(ProvenanceRpcError::Serialize)?;
        buf.push(b'\n');

        writer
            .write_all(&buf)
            .await
            .map_err(|e| ProvenanceRpcError::Io {
                context: "write",
                source: e,
            })?;

        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        buf.read_line(&mut line)
            .await
            .map_err(|e| ProvenanceRpcError::Io {
                context: "read",
                source: e,
            })?;

        let response: Value = serde_json::from_str(&line).map_err(ProvenanceRpcError::Parse)?;

        if let Some(error) = response.get("error") {
            return Err(ProvenanceRpcError::RpcError(error.clone()));
        }

        response
            .get("result")
            .cloned()
            .ok_or(ProvenanceRpcError::NoResult)
    }
}

/// Availability of provenance trio members.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrioAvailability {
    /// Ephemeral DAG provider (`dag.session`) is available.
    pub ephemeral_dag: bool,
    /// Attribution provider (`braid.create`) is available.
    pub attribution: bool,
    /// Permanent ledger provider (`spine.create`) is available.
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
/// Resolution:
/// 1. `<CAPABILITY_UPPER>_SOCKET` env var (e.g. `DAG_SESSION_SOCKET`)
/// 2. `PROVENANCE_TRIO_SOCKET` env var (shared for all trio operations)
/// 3. Scan DH-1 socket search dirs for `.sock` files whose stem
///    contains the capability domain prefix.
fn discover_capability_socket(capability: &str) -> Option<String> {
    let env_key = format!("{}_SOCKET", capability.replace('.', "_").to_uppercase());
    if let Ok(path) = std::env::var(&env_key)
        && std::path::Path::new(&path).exists()
    {
        return Some(path);
    }

    if let Ok(path) = std::env::var(petal_tongue_core::constants::PROVENANCE_TRIO_SOCKET)
        && std::path::Path::new(&path).exists()
    {
        return Some(path);
    }

    let biomeos_dir = petal_tongue_core::constants::resolve_biomeos_socket_dir();
    if let Ok(entries) = std::fs::read_dir(&biomeos_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sock") {
                let name = path.file_stem().unwrap_or_default().to_string_lossy();
                if capability_matches_socket_name(capability, &name) {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

/// Check if a socket name suggests it provides a given capability.
///
/// Uses the domain prefix (the part before the first `.`) from the
/// capability string. Provenance trio methods have well-known domain
/// aliases — `contribution.*` maps to `braid`, `entry.*` maps to `spine`.
fn capability_matches_socket_name(capability: &str, socket_name: &str) -> bool {
    let domain = capability.split('.').next().unwrap_or(capability);
    let effective_domain = match domain {
        "contribution" => "braid",
        "entry" => "spine",
        other => other,
    };
    socket_name.contains(effective_domain)
}

/// Compute a BLAKE3 hash of data (returns first 8 bytes as u64 for display).
fn blake3_hash(data: &[u8]) -> u64 {
    let hash = blake3::hash(data);
    let bytes = hash.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

#[cfg(test)]
#[path = "provenance_trio_tests.rs"]
mod tests;
