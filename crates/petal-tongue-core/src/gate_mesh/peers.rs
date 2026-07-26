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
    pub capabilities: Vec<&'static str>,
}

/// The local gate identity for perspective-relative peer derivation.
const LOCAL_GATE_ID: &str = "eastGate";

/// Derive mesh peers from static topology (offline fallback).
///
/// Uses node attributes (zone, enrollment, role, GPU) to infer connectivity
/// rather than hardcoding per-name behavior. Derives perspective from
/// `eastGate` as the local observation point.
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
            let (status, transport, latency) = derive_connectivity(node);
            let capabilities = derive_capabilities(node);
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

/// Derive connectivity from node attributes (zone, WG, enrollment).
fn derive_connectivity(node: &MeshNode) -> (PeerStatus, &'static str, u32) {
    if node.id == LOCAL_GATE_ID {
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
        match node.zone {
            "WAN" => (PeerStatus::Relayed, "WG via golgi", 32),
            "VPS" => (PeerStatus::Connected, "WG overlay", 11),
            _ => (PeerStatus::Connected, "WG overlay", 15),
        }
    } else {
        (PeerStatus::Pending, "LAN direct (mesh pending)", 1)
    }
}

/// Derive capabilities from node attributes (role, GPU, nucleus count).
fn derive_capabilities(node: &MeshNode) -> Vec<&'static str> {
    let mut caps = Vec::with_capacity(4);

    if node.gpu_target.is_some() {
        caps.push("compute.gpu");
    }

    if node.role.contains("hub") || node.role.contains("relay") || node.role.contains("Forgejo") {
        caps.push("wg.relay");
    }
    if node.role.contains("Forgejo") || node.role.contains("CI") || node.role.contains("depot") {
        caps.push("ci.build");
    }
    if node.role.contains("Forgejo") || node.role.contains("git") {
        caps.push("git.serve");
    }
    if node.role.contains("proxy") || node.role.contains("Tower") || node.role.contains("entry") {
        caps.push("http.proxy");
    }
    if node.role.contains("Overwatch") || node.role.contains("primalSpring") {
        caps.push("overwatch");
    }
    if node.role.contains("Visualization") || node.role.contains("viz") || node.id == LOCAL_GATE_ID
    {
        caps.push("viz.serve");
    }
    if node.role.contains("trust") || node.role.contains("anchor") {
        caps.push("trust.anchor");
    }
    if node.role.contains("Jupyter") {
        caps.push("jupyter.serve");
    }
    if node.role.contains("security") || node.role.contains("BTSP") || node.role.contains("skunkBat") {
        caps.push("security.advisory");
    }
    if node.role.contains("Windows") || node.zone == "windows" {
        caps.push("mesh.windows");
    }
    if node.role.contains("cascade") || node.role.contains("timer") {
        caps.push("cascade.timer");
    }
    if node.role.contains("cellular") || node.zone == "mobile" {
        caps.push("cellular.relay");
    }
    if node.role.contains("Nest") || node.role.contains("CAS") || node.role.contains("storage") {
        caps.push("mesh.hub");
    }

    caps
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

    #[test]
    fn connectivity_derived_from_attributes_not_names() {
        let test_node = &super::super::MeshNode {
            id: "newTestGate",
            label: "newTestGate",
            zone: "backbone",
            lan_ip: Some("192.168.4.100"),
            wg_ip: Some("10.13.37.99"),
            enrollment: GateEnrollment::Enrolled,
            nucleus_count: 5,
            role: "Test GPU compute node",
            kderm_layer: "Cytoplasm",
            gpu_target: Some("sm_90"),
            x: 0.0,
            y: 0.0,
        };
        let (status, transport, latency) = derive_connectivity(test_node);
        assert_eq!(status, PeerStatus::Connected);
        assert_eq!(transport, "LAN direct");
        assert_eq!(latency, 1);

        let caps = derive_capabilities(test_node);
        assert!(caps.contains(&"compute.gpu"));
    }

    #[test]
    fn mobile_node_uses_adb() {
        let mobile = &super::super::MeshNode {
            id: "mobileGate",
            label: "mobileGate",
            zone: "mobile",
            lan_ip: None,
            wg_ip: None,
            enrollment: GateEnrollment::Enrolled,
            nucleus_count: 3,
            role: "Portable trust anchor",
            kderm_layer: "Outer membrane",
            gpu_target: None,
            x: 0.0,
            y: 0.0,
        };
        let (status, transport, latency) = derive_connectivity(mobile);
        assert_eq!(status, PeerStatus::Connected);
        assert_eq!(transport, "ADB USB");
        assert_eq!(latency, 0);

        let caps = derive_capabilities(mobile);
        assert!(caps.contains(&"trust.anchor"));
        assert!(caps.contains(&"cellular.relay"));
    }

    #[test]
    fn wan_node_is_relayed() {
        let wan = &super::super::MeshNode {
            id: "remoteGate",
            label: "remoteGate",
            zone: "WAN",
            lan_ip: None,
            wg_ip: Some("10.13.37.50"),
            enrollment: GateEnrollment::Enrolled,
            nucleus_count: 13,
            role: "Remote node",
            kderm_layer: "Outer membrane",
            gpu_target: None,
            x: 0.0,
            y: 0.0,
        };
        let (status, transport, _latency) = derive_connectivity(wan);
        assert_eq!(status, PeerStatus::Relayed);
        assert_eq!(transport, "WG via golgi");
    }

    #[test]
    fn vps_node_is_connected_via_overlay() {
        let vps = &super::super::MeshNode {
            id: "vpsNode",
            label: "vpsNode",
            zone: "VPS",
            lan_ip: None,
            wg_ip: Some("10.13.37.1"),
            enrollment: GateEnrollment::Enrolled,
            nucleus_count: 18,
            role: "WG hub + Forgejo + relay + depot",
            kderm_layer: "Periplasm",
            gpu_target: None,
            x: 0.0,
            y: 0.0,
        };
        let (status, transport, _latency) = derive_connectivity(vps);
        assert_eq!(status, PeerStatus::Connected);
        assert_eq!(transport, "WG overlay");

        let caps = derive_capabilities(vps);
        assert!(caps.contains(&"wg.relay"));
        assert!(caps.contains(&"ci.build"));
        assert!(caps.contains(&"git.serve"));
    }
}
