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
/// In production, this data would come from a discovery service's `mesh.peers`
/// capability call. This derivation provides the offline fallback for visualization.
#[derive(Debug, Clone, Serialize)]
pub struct MeshPeer {
    /// Gate identifier.
    pub gate_id: String,
    /// Current connectivity status.
    pub status: PeerStatus,
    /// Transport type (LAN direct, WG overlay, relay, ADB).
    pub transport: String,
    /// Latency in ms (0 = local, `u32::MAX` = unreachable).
    pub latency_ms: u32,
    /// Capabilities this peer advertises.
    pub capabilities: Vec<String>,
}

/// Derive mesh peers from topology nodes (offline fallback).
///
/// Uses node attributes (zone, enrollment, role, GPU) to infer connectivity
/// rather than hardcoding per-name behavior. The `local_gate_id` determines
/// the observation perspective (discovered from env `PETALTONGUE_GATE_ID`
/// or defaults to hostname).
///
/// When a discovery service is available, callers should prefer the live
/// `mesh.peers` capability call over this derivation.
#[must_use]
pub fn derive_mesh_peers(nodes: &[MeshNode]) -> Vec<MeshPeer> {
    let local_gate_id = std::env::var("PETALTONGUE_GATE_ID").unwrap_or_default();
    nodes
        .iter()
        .filter(|n| {
            matches!(
                n.enrollment,
                GateEnrollment::Enrolled | GateEnrollment::MeshLive
            )
        })
        .map(|node| {
            let (status, transport, latency) = derive_connectivity(node, &local_gate_id);
            let capabilities = derive_capabilities(node);
            MeshPeer {
                gate_id: node.id.clone(),
                status,
                transport: transport.into(),
                latency_ms: latency,
                capabilities,
            }
        })
        .collect()
}

/// Derive connectivity from node attributes (zone, WG, enrollment).
fn derive_connectivity(node: &MeshNode, local_gate_id: &str) -> (PeerStatus, &'static str, u32) {
    if !local_gate_id.is_empty() && node.id == local_gate_id {
        return (PeerStatus::Connected, "local", 0);
    }

    match node.enrollment {
        GateEnrollment::MeshLive | GateEnrollment::Sovereign | GateEnrollment::Public => {
            (PeerStatus::Pending, "pending enrollment", u32::MAX)
        }
        GateEnrollment::Offline => (PeerStatus::Unreachable, "offline", u32::MAX),
        GateEnrollment::Enrolled => derive_enrolled_connectivity(node),
    }
}

/// Derive connectivity for an enrolled node based on zone and transport.
fn derive_enrolled_connectivity(node: &MeshNode) -> (PeerStatus, &'static str, u32) {
    if node.zone == "mobile" {
        return (PeerStatus::Connected, "ADB USB", 0);
    }

    if node.lan_ip.is_some() {
        return (PeerStatus::Connected, "LAN direct", 1);
    }

    if node.wg_ip.is_some() {
        match node.zone.as_str() {
            "WAN" => (PeerStatus::Relayed, "WG via golgi", 32),
            "VPS" => (PeerStatus::Connected, "WG overlay", 11),
            _ => (PeerStatus::Connected, "WG overlay", 15),
        }
    } else {
        (PeerStatus::Pending, "LAN direct (mesh pending)", 1)
    }
}

/// Derive capabilities from node attributes (role, GPU, nucleus count).
fn derive_capabilities(node: &MeshNode) -> Vec<String> {
    let mut caps = Vec::with_capacity(4);

    if node.gpu_target.is_some() {
        caps.push("compute.gpu".into());
    }

    if node.role.contains("hub") || node.role.contains("relay") || node.role.contains("Forgejo") {
        caps.push("wg.relay".into());
    }
    if node.role.contains("Forgejo") || node.role.contains("CI") || node.role.contains("depot") {
        caps.push("ci.build".into());
    }
    if node.role.contains("Forgejo") || node.role.contains("git") {
        caps.push("git.serve".into());
    }
    if node.role.contains("proxy") || node.role.contains("Tower") || node.role.contains("entry") {
        caps.push("http.proxy".into());
    }
    if node.role.contains("Overwatch") || node.role.contains("primalSpring") {
        caps.push("overwatch".into());
    }
    if node.role.contains("Visualization") || node.role.contains("viz") {
        caps.push("viz.serve".into());
    }
    if node.role.contains("trust") || node.role.contains("anchor") {
        caps.push("trust.anchor".into());
    }
    if node.role.contains("Jupyter") {
        caps.push("jupyter.serve".into());
    }
    if node.role.contains("security") || node.role.contains("BTSP") {
        caps.push("security.advisory".into());
    }
    if node.role.contains("Windows") || node.zone == "windows" {
        caps.push("mesh.windows".into());
    }
    if node.role.contains("cascade") || node.role.contains("timer") {
        caps.push("cascade.timer".into());
    }
    if node.role.contains("cellular") || node.zone == "mobile" {
        caps.push("cellular.relay".into());
    }
    if node.role.contains("Nest") || node.role.contains("CAS") || node.role.contains("storage") {
        caps.push("mesh.hub".into());
    }

    caps
}

#[cfg(test)]
mod tests {
    use super::super::all_nodes;
    use super::*;

    fn test_node(
        id: &str,
        zone: &str,
        lan_ip: Option<&str>,
        wg_ip: Option<&str>,
        enrollment: GateEnrollment,
        role: &str,
        gpu_target: Option<&str>,
    ) -> MeshNode {
        MeshNode {
            id: id.into(),
            label: id.into(),
            zone: zone.into(),
            lan_ip: lan_ip.map(Into::into),
            wg_ip: wg_ip.map(Into::into),
            enrollment,
            nucleus_count: 0,
            role: role.into(),
            kderm_layer: String::new(),
            gpu_target: gpu_target.map(Into::into),
            x: 0.0,
            y: 0.0,
        }
    }

    #[test]
    fn derive_mesh_peers_returns_active_nodes() {
        let nodes = all_nodes();
        let peers = derive_mesh_peers(&nodes);
        assert!(peers.len() >= 6);
        let iron = peers.iter().find(|p| p.gate_id == "ironGate").unwrap();
        assert_eq!(iron.status, PeerStatus::Connected);
        assert!(!iron.capabilities.is_empty());
    }

    #[test]
    fn mesh_peer_serialization() {
        let nodes = all_nodes();
        let peers = derive_mesh_peers(&nodes);
        let json = serde_json::to_string(&peers).unwrap();
        assert!(json.contains("ironGate"));
        assert!(json.contains("compute.gpu"));
    }

    #[test]
    fn northgate_is_pending() {
        let nodes = all_nodes();
        let peers = derive_mesh_peers(&nodes);
        let north = peers.iter().find(|p| p.gate_id == "northGate").unwrap();
        assert_eq!(north.status, PeerStatus::Pending);
        assert!(north.capabilities.iter().any(|c| c == "compute.gpu"));
    }

    #[test]
    fn connectivity_derived_from_attributes_not_names() {
        let node = test_node(
            "newTestGate",
            "backbone",
            Some("192.168.4.100"),
            Some("10.13.37.99"),
            GateEnrollment::Enrolled,
            "Test GPU compute node",
            Some("sm_90"),
        );
        let (status, transport, latency) = derive_connectivity(&node, "");
        assert_eq!(status, PeerStatus::Connected);
        assert_eq!(transport, "LAN direct");
        assert_eq!(latency, 1);

        let caps = derive_capabilities(&node);
        assert!(caps.iter().any(|c| c == "compute.gpu"));
    }

    #[test]
    fn mobile_node_uses_adb() {
        let mobile = test_node(
            "mobileGate",
            "mobile",
            None,
            None,
            GateEnrollment::Enrolled,
            "Portable trust anchor",
            None,
        );
        let (status, transport, latency) = derive_connectivity(&mobile, "");
        assert_eq!(status, PeerStatus::Connected);
        assert_eq!(transport, "ADB USB");
        assert_eq!(latency, 0);

        let caps = derive_capabilities(&mobile);
        assert!(caps.iter().any(|c| c == "trust.anchor"));
        assert!(caps.iter().any(|c| c == "cellular.relay"));
    }

    #[test]
    fn wan_node_is_relayed() {
        let wan = test_node(
            "remoteGate",
            "WAN",
            None,
            Some("10.13.37.50"),
            GateEnrollment::Enrolled,
            "Remote node",
            None,
        );
        let (status, transport, _latency) = derive_connectivity(&wan, "");
        assert_eq!(status, PeerStatus::Relayed);
        assert_eq!(transport, "WG via golgi");
    }

    #[test]
    fn vps_node_is_connected_via_overlay() {
        let vps = test_node(
            "vpsNode",
            "VPS",
            None,
            Some("10.13.37.1"),
            GateEnrollment::Enrolled,
            "WG hub + Forgejo + relay + depot",
            None,
        );
        let (status, transport, _latency) = derive_connectivity(&vps, "");
        assert_eq!(status, PeerStatus::Connected);
        assert_eq!(transport, "WG overlay");

        let caps = derive_capabilities(&vps);
        assert!(caps.iter().any(|c| c == "wg.relay"));
        assert!(caps.iter().any(|c| c == "ci.build"));
        assert!(caps.iter().any(|c| c == "git.serve"));
    }
}
