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

    #[test]
    fn derive_mesh_peers_returns_active_nodes() {
        let peers = derive_mesh_peers();
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
        let peers = derive_mesh_peers();
        let json = serde_json::to_string(&peers).unwrap();
        assert!(json.contains("connected"));
        assert!(json.contains("eastGate"));
        assert!(json.contains("compute.gpu"));
    }

    #[test]
    fn kderm_layers_complete() {
        assert_eq!(KDERM_LAYERS.len(), 5);
        assert_eq!(KDERM_LAYERS[0].name, "Extracellular");
        assert_eq!(KDERM_LAYERS[4].name, "Cytoplasm");
        for layer in KDERM_LAYERS {
            assert!(!layer.components.is_empty());
            assert!(!layer.security.is_empty());
        }
    }

    #[test]
    fn hardening_controls_consistent() {
        let layer_names: Vec<&str> = KDERM_LAYERS.iter().map(|l| l.name).collect();
        for ctrl in HARDENING_CONTROLS {
            assert!(
                layer_names.contains(&ctrl.layer),
                "control {} references unknown layer: {}",
                ctrl.id,
                ctrl.layer
            );
        }
        let active = HARDENING_CONTROLS
            .iter()
            .filter(|c| c.status == HardeningStatus::Active)
            .count();
        assert!(active >= 4);
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

// ── Mesh Peers (songBird mesh.peers concept) ────────────────────────────

/// Peer connectivity status (mirrors songBird `peer.status` response).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Latency in ms (0 = local, u32::MAX = unreachable).
    pub latency_ms: u32,
    /// Capabilities this peer advertises.
    pub capabilities: &'static [&'static str],
}

/// Derive mesh peers from static topology (offline fallback).
///
/// Simulates what `songBird mesh.peers` would return: all enrolled gates
/// with their connectivity from eastGate's perspective.
#[must_use]
pub fn derive_mesh_peers() -> Vec<MeshPeer> {
    all_nodes()
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

// ── K-Derm Diderm Topology (live visualization) ─────────────────────────

/// K-Derm membrane layer in the diderm architecture.
///
/// Maps the bacterial cell envelope model to network defense topology.
/// Each layer has distinct security responsibilities and data flows.
#[derive(Debug, Clone, Serialize)]
pub struct KDermLayer {
    /// Layer name (e.g. "Outer membrane").
    pub name: &'static str,
    /// Concise description of what this layer does.
    pub role: &'static str,
    /// Components operating at this layer.
    pub components: &'static [&'static str],
    /// primals.eco routing path at this layer.
    pub path: &'static str,
    /// Security properties this layer provides.
    pub security: &'static [&'static str],
    /// Data flow direction (inbound, outbound, bidirectional).
    pub data_flow: &'static str,
}

/// The 5-layer K-Derm diderm topology (Wave 136b canonical).
///
/// Outer membrane data reinforces inner membrane. Cross-membrane validation
/// ensures integrity (content hash, timing baseline, TLS cert, DNS, routes).
pub const KDERM_LAYERS: &[KDermLayer] = &[
    KDermLayer {
        name: "Extracellular",
        role: "Public internet, hostile traffic",
        components: &["Crawlers", "Scanners", "Public DNS"],
        path: "Hostile traffic (unfiltered)",
        security: &["No trust", "Observable only"],
        data_flow: "inbound",
    },
    KDermLayer {
        name: "Outer membrane",
        role: "DDoS mitigation, CDN, TLS edge",
        components: &["Cloudflare proxy", "Caddy (TLS/CSP/HSTS)", "DNSSEC"],
        path: "lab.primals.eco → CF → Flint H1",
        security: &["DDoS absorb", "TLS termination", "WAF rules", "Cache"],
        data_flow: "bidirectional",
    },
    KDermLayer {
        name: "Periplasm",
        role: "Routing, relay, build pipeline",
        components: &[
            "golgi relay",
            "sporeGate CI",
            "WireGuard mesh",
            "songBird drawbridge",
        ],
        path: "WG overlay → sporeGate → LAN/WAN backends",
        security: &["Authenticated relay", "Signed builds", "Allowlist proxy"],
        data_flow: "bidirectional",
    },
    KDermLayer {
        name: "Plasma membrane",
        role: "Boundary enforcement, firewall",
        components: &[
            "Flint H1 (edge router)",
            "UFW per-gate",
            "fail2ban",
            "Port-forward rules",
        ],
        path: "NAT + firewall → per-gate ingress",
        security: &[
            "Stateful firewall",
            "Rate limiting",
            "Ban automation",
            "Port isolation",
        ],
        data_flow: "inbound",
    },
    KDermLayer {
        name: "Cytoplasm",
        role: "Sovereign compute, UDS IPC",
        components: &[
            "NUCLEUS primals",
            "Unix domain sockets",
            "songBird mesh",
            "bearDog TLS",
        ],
        path: "localhost-only services, mesh capability.call",
        security: &[
            "Zero external ports",
            "Capability-based routing",
            "Signed IPC",
            "Process isolation",
        ],
        data_flow: "internal",
    },
];

/// Hardening status for security controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardeningStatus {
    /// Fully active and validated.
    Active,
    /// Code landed, activation pending.
    Landed,
    /// In progress.
    InProgress,
    /// Planned but not started.
    Planned,
}

/// A security hardening control and its current state.
#[derive(Debug, Clone, Serialize)]
pub struct HardeningControl {
    /// Control identifier (e.g. "DNSSEC", "HSTS").
    pub id: &'static str,
    /// Which K-Derm layer this control operates at.
    pub layer: &'static str,
    /// Current deployment status.
    pub status: HardeningStatus,
    /// Brief description.
    pub description: &'static str,
}

/// Current hardening controls (Wave 136b state).
pub const HARDENING_CONTROLS: &[HardeningControl] = &[
    HardeningControl {
        id: "DNSSEC",
        layer: "Outer membrane",
        status: HardeningStatus::Active,
        description: "DS record at Porkbun, CF DNSSEC enabled (keyTag 2371, alg 13)",
    },
    HardeningControl {
        id: "HSTS",
        layer: "Outer membrane",
        status: HardeningStatus::Active,
        description: "Strict-Transport-Security on all domains",
    },
    HardeningControl {
        id: "CSP",
        layer: "Outer membrane",
        status: HardeningStatus::Active,
        description: "Content-Security-Policy headers (Caddy)",
    },
    HardeningControl {
        id: "fail2ban",
        layer: "Plasma membrane",
        status: HardeningStatus::Active,
        description: "Automated ban on repeated auth failures",
    },
    HardeningControl {
        id: "drawbridge",
        layer: "Periplasm",
        status: HardeningStatus::Active,
        description: "songBird external proxy allowlist (OSM, FEMA, USGS, ArcGIS)",
    },
    HardeningControl {
        id: "cascade-signing",
        layer: "Periplasm",
        status: HardeningStatus::Landed,
        description: "ed25519 signed cascade commits (code landed, activation pending)",
    },
    HardeningControl {
        id: "lab-auth-gate",
        layer: "Periplasm",
        status: HardeningStatus::Landed,
        description: "lab.primals.eco access control (songBird drawbridge → Caddy)",
    },
    HardeningControl {
        id: "bearDog-TLS",
        layer: "Cytoplasm",
        status: HardeningStatus::Landed,
        description: "bearDog ACME gateway replacing Caddy TLS termination",
    },
];
