// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gate mesh topology — shared canonical data.
//!
//! Offline fallback for the gate mesh topology. Both the visualization
//! scene builder and the `gate.mesh.status` IPC handler consume this data.
//!
//! **Authoritative source**: `ecosystem_manifest.toml` `[gates.*]` section,
//! specifically `wg_ip`, `zone`, and `roles` fields. The constants below
//! (`GATES`, `VPS_NODES`) are compile-time snapshots for offline rendering
//! when the manifest is unavailable.
//!
//! At runtime, prefer live state from `gate.mesh.live` capability discovery
//! or the manifest reader (`EcosystemManifest::mesh_ip_for()`).

pub mod kderm;
pub mod nucleus;
pub mod peers;

pub use kderm::{HardeningControl, HardeningStatus, KDermLayer, HARDENING_CONTROLS, KDERM_LAYERS};
pub use nucleus::{
    NucleusAtomic, NucleusPrimal, META_ATOMIC, NEST_ATOMIC, NODE_ATOMIC, NUCLEUS_ATOMICS,
    TOWER_ATOMIC, nucleus_primal_count,
};
pub use peers::{MeshPeer, PeerStatus, derive_mesh_peers};

use serde::{Deserialize, Serialize};

/// Gate enrollment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateEnrollment {
    /// Fully enrolled: SSH + `NUCLEUS` 13/13 + `WireGuard` + Cascade.
    Enrolled,
    /// `WireGuard` overlay active but `NUCLEUS` not yet deployed.
    MeshLive,
    /// On sovereign relay, not yet SSH/WG enrolled.
    Sovereign,
    /// Still on public relay.
    Public,
    /// Hardware offline or unreachable.
    Offline,
}

impl GateEnrollment {
    /// Returns a stable string representation without heap allocation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enrolled => "Enrolled",
            Self::MeshLive => "MeshLive",
            Self::Sovereign => "Sovereign",
            Self::Public => "Public",
            Self::Offline => "Offline",
        }
    }
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
    /// LAN IP on the 192.168.4.0/22 network, if local.
    pub lan_ip: Option<&'static str>,
    /// `WireGuard` IP on the 10.13.37.0/24 overlay, if enrolled.
    pub wg_ip: Option<&'static str>,
    /// Current enrollment pipeline status.
    pub enrollment: GateEnrollment,
    /// Number of `NUCLEUS` primals running (0–13+).
    pub nucleus_count: u8,
    /// Ecosystem role (e.g. "Build authority", "Overwatch", "Tower").
    pub role: &'static str,
    /// K-Derm membrane layer assignment.
    pub kderm_layer: &'static str,
    /// GPU compute target (e.g. `sm_70`, `sm_120`), None if no GPU.
    pub gpu_target: Option<&'static str>,
    /// X position for topology visualization layout.
    pub x: f64,
    /// Y position for topology visualization layout.
    pub y: f64,
}

/// A `WireGuard` link between mesh nodes.
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
        lan_ip: Some("192.168.4.3"),
        wg_ip: Some("10.13.37.2"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Public entry + Sovereign CI + Nest",
        kderm_layer: "Peptidoglycan",
        gpu_target: None,
        x: 200.0,
        y: 150.0,
    },
    MeshNode {
        id: "eastGate",
        label: "eastGate",
        zone: "backbone",
        lan_ip: Some("192.168.4.244"),
        wg_ip: Some("10.13.37.5"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Overwatch + primalSpring + cellMembrane",
        kderm_layer: "Cytoplasm",
        gpu_target: None,
        x: 450.0,
        y: 100.0,
    },
    MeshNode {
        id: "northGate",
        label: "northGate",
        zone: "backbone",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 14,
        role: "Windows mesh target (RTX 5090, songBird ready)",
        kderm_layer: "Public",
        gpu_target: Some("sm_120"),
        x: 650.0,
        y: 200.0,
    },
    MeshNode {
        id: "ironGate",
        label: "ironGate",
        zone: "house2",
        lan_ip: Some("192.168.4.237"),
        wg_ip: Some("10.13.37.7"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 12,
        role: "GPU compute + ABG (RTX 5070, JupyterHub)",
        kderm_layer: "Cytoplasm",
        gpu_target: Some("sm_70"),
        x: 700.0,
        y: 350.0,
    },
    MeshNode {
        id: "flockGate",
        label: "flockGate",
        zone: "WAN",
        lan_ip: None,
        wg_ip: Some("10.13.37.6"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Tower atomic home (songBird, bearDog, skunkBat)",
        kderm_layer: "Outer membrane",
        gpu_target: None,
        x: 150.0,
        y: 400.0,
    },
    MeshNode {
        id: "grapheneGate",
        label: "grapheneGate",
        zone: "mobile",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 14,
        role: "Portable trust anchor (Pixel 8a, Tower)",
        kderm_layer: "Outer membrane",
        gpu_target: None,
        x: 300.0,
        y: 450.0,
    },
    MeshNode {
        id: "strandGate",
        label: "strandGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::MeshLive,
        nucleus_count: 0,
        role: "CPU compute (64-core EPYC, STAR alignment)",
        kderm_layer: "Cytoplasm",
        gpu_target: None,
        x: 400.0,
        y: 450.0,
    },
    MeshNode {
        id: "southGate",
        label: "southGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Public,
        nucleus_count: 0,
        role: "Relay pending",
        kderm_layer: "Public",
        gpu_target: None,
        x: 500.0,
        y: 500.0,
    },
    MeshNode {
        id: "swiftGate",
        label: "swiftGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Public,
        nucleus_count: 0,
        role: "Omada-side WiFi",
        kderm_layer: "Public",
        gpu_target: None,
        x: 600.0,
        y: 480.0,
    },
    MeshNode {
        id: "fieldGate",
        label: "fieldGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Offline,
        nucleus_count: 0,
        role: "CMOS dead",
        kderm_layer: "Offline",
        gpu_target: None,
        x: 750.0,
        y: 500.0,
    },
    MeshNode {
        id: "westGate",
        label: "westGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Offline,
        nucleus_count: 0,
        role: "ZFS cold storage (AlphaFold data)",
        kderm_layer: "Offline",
        gpu_target: None,
        x: 850.0,
        y: 500.0,
    },
];

/// VPS/infrastructure nodes (always enrolled).
pub const VPS_NODES: &[MeshNode] = &[MeshNode {
    id: "golgi",
    label: "golgi (hub)",
    zone: "VPS",
    lan_ip: None,
    wg_ip: Some("10.13.37.1"),
    enrollment: GateEnrollment::Enrolled,
    nucleus_count: 18,
    role: "WG hub + Forgejo + relay + depot",
    kderm_layer: "Periplasm",
    gpu_target: None,
    x: 350.0,
    y: 280.0,
}];

/// Known `WireGuard` overlay links.
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
#[must_use]
pub fn count_by_enrollment(status: GateEnrollment) -> usize {
    all_nodes().filter(|n| n.enrollment == status).count()
}

/// Count of nodes that are at least mesh-live (enrolled or `mesh_live`).
#[must_use]
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

/// Nodes with GPU compute capability.
pub fn gpu_nodes() -> impl Iterator<Item = &'static MeshNode> {
    all_nodes().filter(|n| n.gpu_target.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topology_consistency() {
        assert_eq!(GATES.len(), 11);
        assert_eq!(VPS_NODES.len(), 1);
        assert_eq!(WG_LINKS.len(), 7);
        assert_eq!(all_nodes().count(), 12);
    }

    #[test]
    fn enrollment_counts() {
        assert_eq!(count_by_enrollment(GateEnrollment::Enrolled), 7);
        assert_eq!(mesh_active_count(), 8);
    }

    #[test]
    fn gpu_nodes_identified() {
        let gpu: Vec<&str> = gpu_nodes().map(|n| n.id).collect();
        assert!(gpu.contains(&"ironGate"));
        assert!(gpu.contains(&"northGate"));
        assert_eq!(gpu.len(), 2);
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
