// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mesh peers — songBird `mesh.peers` concept.
//!
//! In production, mesh peer data comes from songBird's `mesh.peers` IPC call.
//! This module provides the offline derivation fallback for visualization when
//! songBird is not reachable. The derivation uses the static topology data
//! from the parent module.

use serde::Serialize;

use super::{GateEnrollment, MeshNode};

/// Peer connectivity status (mirrors songBird `peer.status` response).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerStatus {
    /// Active TCP connection (LAN direct or WG).
    Connected,
    /// Reachable via relay (golgi).
    Relayed,
    /// Enrolled but not currently reachable.
    Unreachable,
    /// Not yet enrolled in mesh.
    Pending,
}

/// A mesh peer as seen from this gate's perspective.
///
/// In production, this data would come from songBird's `mesh.peers` IPC call.
/// This static derivation provides the offline fallback for visualization.
#[derive(Debug, Clone, Serialize)]
pub struct MeshPeer {
    /// Gate identifier.
    pub gate_id: &'static str,
    /// Current connectivity status.
    pub status: PeerStatus,
    /// Transport type (LAN direct, WG overlay, relay, ADB).
    pub transport: &'static str,
    /// Latency in ms (0 = local, `u32::MAX` = unreachable).
    pub latency_ms: u32,
    /// Capabilities this peer advertises.
    pub capabilities: &'static [&'static str],
}

/// Derive mesh peers from static topology (offline fallback).
///
/// Simulates what `songBird mesh.peers` would return: all enrolled gates
/// with their connectivity from eastGate's perspective.
///
/// When songBird IPC is available, callers should prefer the live
/// `mesh.peers` capability call over this static derivation.
#[must_use]
pub fn derive_mesh_peers(nodes: impl Iterator<Item = &'static MeshNode>) -> Vec<MeshPeer> {
    nodes
        .filter(|n| {
            matches!(
                n.enrollment,
                GateEnrollment::Enrolled | GateEnrollment::MeshLive
            )
        })
        .map(|node| {
            let (status, transport, latency) = match node.id {
                "eastGate" => (PeerStatus::Connected, "local", 0),
                "sporeGate" | "ironGate" => (PeerStatus::Connected, "LAN direct", 1),
                "northGate" => (PeerStatus::Pending, "LAN direct (mesh pending)", 1),
                "flockGate" => (PeerStatus::Relayed, "WG via golgi", 32),
                "grapheneGate" => (PeerStatus::Connected, "ADB USB", 0),
                "golgi" => (PeerStatus::Connected, "WG overlay", 11),
                "strandGate" => (PeerStatus::Pending, "pending enrollment", u32::MAX),
                _ => (PeerStatus::Unreachable, "unknown", u32::MAX),
            };
            let capabilities: &'static [&'static str] = match node.id {
                "sporeGate" => &["mesh.hub", "ci.build", "git.serve"],
                "ironGate" => &["compute.gpu", "jupyter.serve"],
                "northGate" => &["compute.gpu", "mesh.windows"],
                "flockGate" => &["http.proxy", "security.advisory"],
                "grapheneGate" => &["trust.anchor", "cellular.relay"],
                "golgi" => &["wg.relay", "cascade.timer"],
                "eastGate" => &["overwatch", "viz.serve"],
                _ => &[],
            };
            MeshPeer {
                gate_id: node.id,
                status,
                transport,
                latency_ms: latency,
                capabilities,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::all_nodes;
    use super::*;

    #[test]
    fn derive_mesh_peers_returns_active_nodes() {
        let peers = derive_mesh_peers(all_nodes());
        assert!(peers.len() >= 6);
        let east = peers.iter().find(|p| p.gate_id == "eastGate").unwrap();
        assert_eq!(east.status, PeerStatus::Connected);
        assert_eq!(east.latency_ms, 0);
        let iron = peers.iter().find(|p| p.gate_id == "ironGate").unwrap();
        assert_eq!(iron.status, PeerStatus::Connected);
        assert!(!iron.capabilities.is_empty());
    }

    #[test]
    fn mesh_peer_serialization() {
        let peers = derive_mesh_peers(all_nodes());
        let json = serde_json::to_string(&peers).unwrap();
        assert!(json.contains("connected"));
        assert!(json.contains("eastGate"));
        assert!(json.contains("compute.gpu"));
    }

    #[test]
    fn northgate_is_pending() {
        let peers = derive_mesh_peers(all_nodes());
        let north = peers.iter().find(|p| p.gate_id == "northGate").unwrap();
        assert_eq!(north.status, PeerStatus::Pending);
        assert!(north.capabilities.contains(&"compute.gpu"));
    }
}
