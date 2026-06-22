// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gate mesh topology — shared canonical data.
//!
//! Single source of truth for the gate mesh topology. Both the visualization
//! scene builder and the `gate.mesh.status` IPC handler consume this data.
//!
//! In production, this static data is the offline/capability-registry fallback.
//! When `gate.mesh.live` capability is discovered at runtime, live state
//! overrides these defaults.

use serde::{Deserialize, Serialize};

/// Gate enrollment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateEnrollment {
    /// Fully enrolled: SSH + NUCLEUS 13/13 + WireGuard + Cascade.
    Enrolled,
    /// WireGuard overlay active but NUCLEUS not yet deployed.
    MeshLive,
    /// On sovereign relay, not yet SSH/WG enrolled.
    Sovereign,
    /// Still on public relay.
    Public,
    /// Hardware offline or unreachable.
    Offline,
}

/// A gate or VPS node in the mesh topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshNode {
    /// Unique gate identifier (e.g. "sporeGate").
    pub id: &'static str,
    /// Human-readable label for display.
    pub label: &'static str,
    /// Cytoplasm zone (backbone, WAN, house2, VPS).
    pub zone: &'static str,
    /// WireGuard IP on the 10.13.37.0/24 overlay, if enrolled.
    pub wg_ip: Option<&'static str>,
    /// Current enrollment pipeline status.
    pub enrollment: GateEnrollment,
    /// Number of NUCLEUS primals running (0–13+).
    pub nucleus_count: u8,
    /// Ecosystem role (e.g. "Build authority", "Overwatch", "Tower").
    pub role: &'static str,
    /// K-Derm membrane layer assignment.
    pub kderm_layer: &'static str,
    /// X position for topology visualization layout.
    pub x: f64,
    /// Y position for topology visualization layout.
    pub y: f64,
}

/// A WireGuard link between mesh nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshLink {
    /// Source node ID.
    pub from: &'static str,
    /// Destination node ID.
    pub to: &'static str,
    /// Measured latency in milliseconds.
    pub latency_ms: u32,
}

/// Static gate topology (offline fallback).
pub const GATES: &[MeshNode] = &[
    MeshNode {
        id: "sporeGate",
        label: "sporeGate",
        zone: "backbone",
        wg_ip: Some("10.13.37.2"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Build authority + Nest provenance",
        kderm_layer: "Peptidoglycan",
        x: 200.0,
        y: 150.0,
    },
    MeshNode {
        id: "eastGate",
        label: "eastGate",
        zone: "backbone",
        wg_ip: Some("10.13.37.5"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Overwatch + Meta primals",
        kderm_layer: "Cytoplasm",
        x: 450.0,
        y: 100.0,
    },
    MeshNode {
        id: "northGate",
        label: "northGate",
        zone: "backbone",
        wg_ip: None,
        enrollment: GateEnrollment::Sovereign,
        nucleus_count: 0,
        role: "Hobby (Windows)",
        kderm_layer: "Public",
        x: 650.0,
        y: 200.0,
    },
    MeshNode {
        id: "ironGate",
        label: "ironGate",
        zone: "backbone",
        wg_ip: Some("10.13.37.7"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 12,
        role: "Node compute + GPU",
        kderm_layer: "Cytoplasm",
        x: 700.0,
        y: 350.0,
    },
    MeshNode {
        id: "flockGate",
        label: "flockGate",
        zone: "WAN",
        wg_ip: Some("10.13.37.6"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Tower primals + sporePrint",
        kderm_layer: "Outer membrane",
        x: 150.0,
        y: 400.0,
    },
    MeshNode {
        id: "strandGate",
        label: "strandGate",
        zone: "house2",
        wg_ip: None,
        enrollment: GateEnrollment::Public,
        nucleus_count: 0,
        role: "Relay pending",
        kderm_layer: "Public",
        x: 400.0,
        y: 450.0,
    },
    MeshNode {
        id: "southGate",
        label: "southGate",
        zone: "house2",
        wg_ip: None,
        enrollment: GateEnrollment::Public,
        nucleus_count: 0,
        role: "Relay pending",
        kderm_layer: "Public",
        x: 500.0,
        y: 500.0,
    },
    MeshNode {
        id: "swiftGate",
        label: "swiftGate",
        zone: "house2",
        wg_ip: None,
        enrollment: GateEnrollment::Public,
        nucleus_count: 0,
        role: "Omada-side WiFi",
        kderm_layer: "Public",
        x: 600.0,
        y: 480.0,
    },
    MeshNode {
        id: "fieldGate",
        label: "fieldGate",
        zone: "house2",
        wg_ip: None,
        enrollment: GateEnrollment::Offline,
        nucleus_count: 0,
        role: "CMOS dead",
        kderm_layer: "Offline",
        x: 750.0,
        y: 500.0,
    },
];

/// VPS/infrastructure nodes (always enrolled).
pub const VPS_NODES: &[MeshNode] = &[MeshNode {
    id: "golgi",
    label: "golgi (hub)",
    zone: "VPS",
    wg_ip: Some("10.13.37.1"),
    enrollment: GateEnrollment::Enrolled,
    nucleus_count: 18,
    role: "WG hub + Forgejo + relay + depot",
    kderm_layer: "Periplasm",
    x: 350.0,
    y: 280.0,
}];

/// Known WireGuard overlay links.
pub const WG_LINKS: &[MeshLink] = &[
    MeshLink {
        from: "golgi",
        to: "sporeGate",
        latency_ms: 12,
    },
    MeshLink {
        from: "golgi",
        to: "eastGate",
        latency_ms: 11,
    },
    MeshLink {
        from: "golgi",
        to: "flockGate",
        latency_ms: 32,
    },
    MeshLink {
        from: "golgi",
        to: "ironGate",
        latency_ms: 11,
    },
    MeshLink {
        from: "sporeGate",
        to: "eastGate",
        latency_ms: 1,
    },
    MeshLink {
        from: "sporeGate",
        to: "ironGate",
        latency_ms: 1,
    },
    MeshLink {
        from: "sporeGate",
        to: "flockGate",
        latency_ms: 72,
    },
];

/// All mesh nodes (gates + VPS).
pub fn all_nodes() -> impl Iterator<Item = &'static MeshNode> {
    GATES.iter().chain(VPS_NODES.iter())
}

/// Count of nodes matching a given enrollment status.
pub fn count_by_enrollment(status: GateEnrollment) -> usize {
    all_nodes().filter(|n| n.enrollment == status).count()
}

/// Count of nodes that are at least mesh-live (enrolled or mesh_live).
pub fn mesh_active_count() -> usize {
    all_nodes()
        .filter(|n| {
            matches!(
                n.enrollment,
                GateEnrollment::Enrolled | GateEnrollment::MeshLive
            )
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topology_consistency() {
        assert_eq!(GATES.len(), 9);
        assert_eq!(VPS_NODES.len(), 1);
        assert_eq!(WG_LINKS.len(), 7);
        assert_eq!(all_nodes().count(), 10);
    }

    #[test]
    fn enrollment_counts() {
        assert_eq!(count_by_enrollment(GateEnrollment::Enrolled), 5);
        assert_eq!(mesh_active_count(), 5);
    }

    #[test]
    fn all_links_reference_valid_nodes() {
        let ids: Vec<&str> = all_nodes().map(|n| n.id).collect();
        for link in WG_LINKS {
            assert!(ids.contains(&link.from), "bad from: {}", link.from);
            assert!(ids.contains(&link.to), "bad to: {}", link.to);
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let json = serde_json::to_string(&GATES[0]).unwrap();
        assert!(json.contains("sporeGate"));
        assert!(json.contains("enrolled"));
    }
}
