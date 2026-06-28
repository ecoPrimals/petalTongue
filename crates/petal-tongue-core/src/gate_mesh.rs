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
    /// GPU compute target (e.g. "sm_70", "sm_120"), None if no GPU.
    pub gpu_target: Option<&'static str>,
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
        lan_ip: Some("192.168.4.3"),
        wg_ip: Some("10.13.37.2"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Compute node + Sovereign CI + Nest",
        kderm_layer: "Peptidoglycan",
        gpu_target: None,
        x: 200.0,
        y: 150.0,
    },
    MeshNode {
        id: "eastGate",
        label: "eastGate",
        zone: "backbone",
        lan_ip: Some("192.168.4.5"),
        wg_ip: Some("10.13.37.5"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 13,
        role: "Overwatch + Meta primals",
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
        enrollment: GateEnrollment::Sovereign,
        nucleus_count: 0,
        role: "Hobby (Windows, RTX 5090)",
        kderm_layer: "Public",
        gpu_target: Some("sm_120"),
        x: 650.0,
        y: 200.0,
    },
    MeshNode {
        id: "ironGate",
        label: "ironGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: Some("10.13.37.7"),
        enrollment: GateEnrollment::Enrolled,
        nucleus_count: 12,
        role: "Node compute + GPU (RTX 5070)",
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
        role: "Tower primals + sporePrint",
        kderm_layer: "Outer membrane",
        gpu_target: None,
        x: 150.0,
        y: 400.0,
    },
    MeshNode {
        id: "strandGate",
        label: "strandGate",
        zone: "house2",
        lan_ip: None,
        wg_ip: None,
        enrollment: GateEnrollment::Public,
        nucleus_count: 0,
        role: "Relay pending",
        kderm_layer: "Public",
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
#[must_use]
pub fn count_by_enrollment(status: GateEnrollment) -> usize {
    all_nodes().filter(|n| n.enrollment == status).count()
}

/// Count of nodes that are at least mesh-live (enrolled or mesh_live).
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

// ── NUCLEUS Composition ──────────────────────────────────────────────────

/// A primal in the NUCLEUS composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusPrimal {
    /// Primal identifier (e.g. "bearDog").
    pub id: &'static str,
    /// Short role description.
    pub role: &'static str,
    /// Primary gate assignment.
    pub gate: &'static str,
}

/// An atomic grouping of primals.
#[derive(Debug, Clone, Serialize)]
pub struct NucleusAtomic {
    /// Atomic name (e.g. "Tower Atomic").
    pub name: &'static str,
    /// Primals in this atomic.
    pub primals: &'static [NucleusPrimal],
}

/// Tower Atomic — trust, transport, defense.
pub const TOWER_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "bearDog",
        role: "Crypto identity, BTSP auth, TLS",
        gate: "flockGate",
    },
    NucleusPrimal {
        id: "songBird",
        role: "Mesh routing, STUN/TURN, relay",
        gate: "flockGate",
    },
    NucleusPrimal {
        id: "skunkBat",
        role: "Threat detection, MethodGate",
        gate: "flockGate",
    },
];

/// Node Atomic — compute, fleet, shaders.
pub const NODE_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "toadStool",
        role: "Fleet management, dispatch",
        gate: "ironGate",
    },
    NucleusPrimal {
        id: "barraCuda",
        role: "GPU compute, LSTM, Vulkan",
        gate: "ironGate",
    },
    NucleusPrimal {
        id: "coralReef",
        role: "Shader pipelines, SPIR-V",
        gate: "ironGate",
    },
];

/// Nest Atomic — storage, provenance.
pub const NEST_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "nestGate",
        role: "Content-addressed storage",
        gate: "sporeGate",
    },
    NucleusPrimal {
        id: "rhizoCrypt",
        role: "DAG sessions, Merkle roots",
        gate: "sporeGate",
    },
    NucleusPrimal {
        id: "loamSpine",
        role: "Ledger commits, spine",
        gate: "sporeGate",
    },
    NucleusPrimal {
        id: "sweetGrass",
        role: "Provenance braids",
        gate: "sporeGate",
    },
];

/// Meta — orchestration, AI, visualization.
pub const META_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "biomeOS",
        role: "Composition orchestrator",
        gate: "eastGate",
    },
    NucleusPrimal {
        id: "squirrel",
        role: "AI dispatch, Ollama",
        gate: "eastGate",
    },
    NucleusPrimal {
        id: "petalTongue",
        role: "Visualization, dashboards",
        gate: "eastGate",
    },
];

/// All 4 NUCLEUS atomics.
pub const NUCLEUS_ATOMICS: &[NucleusAtomic] = &[
    NucleusAtomic {
        name: "Tower Atomic",
        primals: TOWER_ATOMIC,
    },
    NucleusAtomic {
        name: "Node Atomic",
        primals: NODE_ATOMIC,
    },
    NucleusAtomic {
        name: "Nest Atomic",
        primals: NEST_ATOMIC,
    },
    NucleusAtomic {
        name: "Meta",
        primals: META_ATOMIC,
    },
];

/// Total primal count across all atomics.
#[must_use]
pub fn nucleus_primal_count() -> usize {
    NUCLEUS_ATOMICS.iter().map(|a| a.primals.len()).sum()
}
