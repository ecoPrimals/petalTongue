// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gate mesh topology — types, traits, and offline fallback data.
//!
//! ## Architecture
//!
//! Types and traits are always available. Static topology data (compile-time
//! snapshots of gate IPs, NUCLEUS assignments, WG links) is gated behind the
//! `offline-topology` feature. Production code should consume topology through
//! the [`MeshTopologySource`] trait; the static fallback is one implementation.
//!
//! **Authoritative source**: `ecosystem_manifest.toml` `[gates.*]` section,
//! `gate.mesh.live` capability discovery, or songBird `mesh.peers` IPC.

pub mod kderm;
#[cfg(feature = "offline-topology")]
pub mod nucleus;
pub mod peers;

pub use kderm::{HARDENING_CONTROLS, HardeningControl, HardeningStatus, KDERM_LAYERS, KDermLayer};
#[cfg(feature = "offline-topology")]
pub use nucleus::{
    META_ATOMIC, NEST_ATOMIC, NODE_ATOMIC, NUCLEUS_ATOMICS, NucleusAtomic, NucleusPrimal,
    TOWER_ATOMIC, nucleus_primal_count,
};
#[cfg(feature = "offline-topology")]
pub use peers::derive_mesh_peers;
pub use peers::{MeshPeer, PeerStatus};

use serde::{Deserialize, Serialize};

/// Gate enrollment status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    #[default]
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
///
/// Uses owned `String` fields so topology can be loaded from manifest files,
/// IPC responses, or discovery at runtime — not just compile-time constants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshNode {
    /// Unique gate identifier (e.g. "sporeGate").
    pub id: String,
    /// Human-readable label for display.
    pub label: String,
    /// Cytoplasm zone (backbone, WAN, house2, VPS).
    #[serde(default)]
    pub zone: String,
    /// LAN IP on the 192.168.4.0/22 network, if local.
    #[serde(default)]
    pub lan_ip: Option<String>,
    /// `WireGuard` IP on the 10.13.37.0/24 overlay, if enrolled.
    #[serde(default)]
    pub wg_ip: Option<String>,
    /// Current enrollment pipeline status.
    #[serde(default)]
    pub enrollment: GateEnrollment,
    /// Number of `NUCLEUS` primals running (0–13+).
    #[serde(default)]
    pub nucleus_count: u8,
    /// Ecosystem role (e.g. "Build authority", "Overwatch", "Tower").
    #[serde(default)]
    pub role: String,
    /// K-Derm membrane layer assignment.
    #[serde(default)]
    pub kderm_layer: String,
    /// GPU compute target (e.g. `sm_70`, `sm_120`), None if no GPU.
    #[serde(default)]
    pub gpu_target: Option<String>,
    /// X position for topology visualization layout.
    #[serde(default)]
    pub x: f64,
    /// Y position for topology visualization layout.
    #[serde(default)]
    pub y: f64,
}

impl MeshNode {
    /// Convenience constructor for static/offline topology data.
    #[cfg(feature = "offline-topology")]
    #[must_use]
    #[expect(clippy::too_many_arguments, reason = "static topology builder")]
    pub fn static_node(
        id: &str,
        zone: &str,
        lan_ip: Option<&str>,
        wg_ip: Option<&str>,
        enrollment: GateEnrollment,
        nucleus_count: u8,
        role: &str,
        kderm_layer: &str,
        gpu_target: Option<&str>,
        x: f64,
        y: f64,
    ) -> Self {
        Self {
            id: id.into(),
            label: id.into(),
            zone: zone.into(),
            lan_ip: lan_ip.map(Into::into),
            wg_ip: wg_ip.map(Into::into),
            enrollment,
            nucleus_count,
            role: role.into(),
            kderm_layer: kderm_layer.into(),
            gpu_target: gpu_target.map(Into::into),
            x,
            y,
        }
    }
}

/// Trait for runtime mesh topology resolution.
///
/// Production implementations should load from `ecosystem_manifest.toml`,
/// query discovery services, or receive topology via IPC. The
/// `offline-topology` feature provides a static compile-time fallback.
pub trait MeshTopologySource: Send + Sync {
    /// All known gate/VPS nodes.
    fn nodes(&self) -> Vec<MeshNode>;
    /// Known `WireGuard` overlay links between nodes.
    fn links(&self) -> Vec<MeshLink>;
    /// Count of nodes matching a given enrollment status.
    fn count_by_enrollment(&self, status: GateEnrollment) -> usize {
        self.nodes()
            .iter()
            .filter(|n| n.enrollment == status)
            .count()
    }
    /// Count of nodes that are mesh-active (enrolled or `MeshLive`).
    fn mesh_active_count(&self) -> usize {
        self.nodes()
            .iter()
            .filter(|n| {
                matches!(
                    n.enrollment,
                    GateEnrollment::Enrolled | GateEnrollment::MeshLive
                )
            })
            .count()
    }
    /// Nodes with GPU compute capability.
    fn gpu_nodes(&self) -> Vec<MeshNode> {
        self.nodes()
            .into_iter()
            .filter(|n| n.gpu_target.is_some())
            .collect()
    }
}

/// A `WireGuard` link between mesh nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshLink {
    /// Source node ID.
    pub from: String,
    /// Destination node ID.
    pub to: String,
    /// Measured latency in milliseconds.
    #[serde(default)]
    pub latency_ms: u32,
}

/// Topology loaded at runtime from `ecosystem_manifest.toml`.
///
/// This is the preferred production topology source — reads gate data from the
/// manifest file that ships alongside the binary, rather than compiling static
/// topology into the binary itself. Falls back to empty topology if the file
/// is missing or unparseable.
#[derive(Debug, Clone, Default)]
pub struct ManifestMeshTopology {
    nodes_cache: Vec<MeshNode>,
    links_cache: Vec<MeshLink>,
}

impl ManifestMeshTopology {
    /// Load topology from a TOML manifest file.
    ///
    /// Parses `[gates.*]` sections into `MeshNode` instances and assigns
    /// automatic layout positions. Returns an empty topology on parse failure.
    #[must_use]
    pub fn from_file(path: &std::path::Path) -> Self {
        let Ok(content) = std::fs::read_to_string(path) else {
            tracing::warn!(
                "Manifest not found at {}, using empty topology",
                path.display()
            );
            return Self::default();
        };
        Self::from_toml(&content)
    }

    /// Load topology from a TOML string.
    #[must_use]
    pub fn from_toml(content: &str) -> Self {
        let Ok(table) = content.parse::<toml::Table>() else {
            tracing::warn!("Failed to parse manifest TOML, using empty topology");
            return Self::default();
        };

        let gates_table = table.get("gates").and_then(toml::Value::as_table);
        let mut nodes = Vec::new();

        if let Some(gates) = gates_table {
            for (i, (gate_id, gate_val)) in gates.iter().enumerate() {
                let empty = toml::Table::new();
                let gate = gate_val.as_table().unwrap_or(&empty);
                let enrollment = gate.get("enrollment").and_then(toml::Value::as_str).map_or(
                    GateEnrollment::Offline,
                    |s| match s {
                        "enrolled" => GateEnrollment::Enrolled,
                        "mesh_live" => GateEnrollment::MeshLive,
                        "sovereign" => GateEnrollment::Sovereign,
                        "public" => GateEnrollment::Public,
                        _ => GateEnrollment::Offline,
                    },
                );

                #[expect(
                    clippy::cast_precision_loss,
                    reason = "gate index fits f64 for layout positioning"
                )]
                let node = MeshNode {
                    id: gate_id.clone(),
                    label: gate_id.clone(),
                    zone: gate
                        .get("zone")
                        .and_then(toml::Value::as_str)
                        .unwrap_or("")
                        .to_owned(),
                    lan_ip: gate
                        .get("lan_ip")
                        .and_then(toml::Value::as_str)
                        .map(String::from),
                    wg_ip: gate
                        .get("wg_ip")
                        .and_then(toml::Value::as_str)
                        .map(String::from),
                    enrollment,
                    nucleus_count: gate
                        .get("nucleus_count")
                        .and_then(toml::Value::as_integer)
                        .and_then(|v| u8::try_from(v).ok())
                        .unwrap_or(0),
                    role: gate
                        .get("role")
                        .and_then(toml::Value::as_str)
                        .unwrap_or("")
                        .to_owned(),
                    kderm_layer: gate
                        .get("kderm_layer")
                        .and_then(toml::Value::as_str)
                        .unwrap_or("")
                        .to_owned(),
                    gpu_target: gate
                        .get("gpu_target")
                        .and_then(toml::Value::as_str)
                        .map(String::from),
                    x: gate
                        .get("x")
                        .and_then(toml::Value::as_float)
                        .unwrap_or_else(|| (i as f64).mul_add(100.0, 150.0)),
                    y: gate
                        .get("y")
                        .and_then(toml::Value::as_float)
                        .unwrap_or_else(|| (i as f64).mul_add(50.0, 150.0)),
                };
                nodes.push(node);
            }
        }

        Self {
            nodes_cache: nodes,
            links_cache: Vec::new(),
        }
    }

    /// Resolve the manifest file using standard search paths.
    #[must_use]
    pub fn discover() -> Self {
        let candidates = [
            std::path::PathBuf::from("ecosystem_manifest.toml"),
            std::path::PathBuf::from("config/ecosystem_manifest.toml"),
        ];
        for path in &candidates {
            if path.exists() {
                return Self::from_file(path);
            }
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(path) = exe
                .parent()
                .map(|dir| dir.join("ecosystem_manifest.toml"))
                .filter(|p| p.exists())
        {
            return Self::from_file(&path);
        }
        tracing::debug!("No ecosystem_manifest.toml found, topology empty until discovery");
        Self::default()
    }
}

impl MeshTopologySource for ManifestMeshTopology {
    fn nodes(&self) -> Vec<MeshNode> {
        self.nodes_cache.clone()
    }

    fn links(&self) -> Vec<MeshLink> {
        self.links_cache.clone()
    }
}

/// Static mesh topology source — wraps compile-time gate data.
///
/// Available only with the `offline-topology` feature. Production deployments
/// should prefer `ManifestMeshTopology` or runtime discovery.
#[cfg(feature = "offline-topology")]
#[derive(Debug, Clone, Copy, Default)]
pub struct StaticMeshTopology;

#[cfg(feature = "offline-topology")]
impl MeshTopologySource for StaticMeshTopology {
    fn nodes(&self) -> Vec<MeshNode> {
        static_nodes()
    }

    fn links(&self) -> Vec<MeshLink> {
        static_links()
    }
}

/// Build static gate node list (offline fallback data).
#[cfg(feature = "offline-topology")]
#[must_use]
#[expect(clippy::too_many_lines, reason = "static reference topology data")]
pub fn static_nodes() -> Vec<MeshNode> {
    vec![
        MeshNode::static_node(
            "sporeGate",
            "backbone",
            Some("192.168.4.3"),
            Some("10.13.37.2"),
            GateEnrollment::Enrolled,
            13,
            "Public entry + Sovereign CI + Nest",
            "Peptidoglycan",
            None,
            200.0,
            150.0,
        ),
        MeshNode::static_node(
            "eastGate",
            "backbone",
            Some("192.168.4.244"),
            Some("10.13.37.5"),
            GateEnrollment::Enrolled,
            13,
            "Overwatch + primalSpring + cellMembrane",
            "Cytoplasm",
            None,
            450.0,
            100.0,
        ),
        MeshNode::static_node(
            "northGate",
            "backbone",
            None,
            None,
            GateEnrollment::Enrolled,
            14,
            "Windows mesh target (RTX 5090)",
            "Public",
            Some("sm_120"),
            650.0,
            200.0,
        ),
        MeshNode::static_node(
            "ironGate",
            "house2",
            Some("192.168.4.237"),
            Some("10.13.37.7"),
            GateEnrollment::Enrolled,
            12,
            "GPU compute + ABG (RTX 5070, JupyterHub)",
            "Cytoplasm",
            Some("sm_70"),
            700.0,
            350.0,
        ),
        MeshNode::static_node(
            "flockGate",
            "WAN",
            None,
            Some("10.13.37.6"),
            GateEnrollment::Enrolled,
            13,
            "Tower atomic home",
            "Outer membrane",
            None,
            150.0,
            400.0,
        ),
        MeshNode::static_node(
            "grapheneGate",
            "mobile",
            None,
            None,
            GateEnrollment::Enrolled,
            14,
            "Portable trust anchor (Pixel 8a, Tower)",
            "Outer membrane",
            None,
            300.0,
            450.0,
        ),
        MeshNode::static_node(
            "strandGate",
            "house2",
            None,
            None,
            GateEnrollment::MeshLive,
            0,
            "CPU compute (64-core EPYC)",
            "Cytoplasm",
            None,
            400.0,
            450.0,
        ),
        MeshNode::static_node(
            "southGate",
            "house2",
            None,
            None,
            GateEnrollment::Public,
            0,
            "Relay pending",
            "Public",
            None,
            500.0,
            500.0,
        ),
        MeshNode::static_node(
            "swiftGate",
            "house2",
            None,
            None,
            GateEnrollment::Public,
            0,
            "Omada-side WiFi",
            "Public",
            None,
            600.0,
            480.0,
        ),
        MeshNode::static_node(
            "fieldGate",
            "house2",
            None,
            None,
            GateEnrollment::Offline,
            0,
            "CMOS dead",
            "Offline",
            None,
            750.0,
            500.0,
        ),
        MeshNode::static_node(
            "westGate",
            "house2",
            None,
            None,
            GateEnrollment::Offline,
            0,
            "ZFS cold storage (AlphaFold data)",
            "Offline",
            None,
            850.0,
            500.0,
        ),
        MeshNode::static_node(
            "golgi",
            "VPS",
            None,
            Some("10.13.37.1"),
            GateEnrollment::Enrolled,
            18,
            "WG hub + Forgejo + relay + depot",
            "Periplasm",
            None,
            350.0,
            280.0,
        ),
    ]
}

/// Build static WG link list (offline fallback data).
#[cfg(feature = "offline-topology")]
#[must_use]
pub fn static_links() -> Vec<MeshLink> {
    vec![
        MeshLink {
            from: "golgi".into(),
            to: "sporeGate".into(),
            latency_ms: 12,
        },
        MeshLink {
            from: "golgi".into(),
            to: "eastGate".into(),
            latency_ms: 11,
        },
        MeshLink {
            from: "golgi".into(),
            to: "flockGate".into(),
            latency_ms: 32,
        },
        MeshLink {
            from: "golgi".into(),
            to: "ironGate".into(),
            latency_ms: 11,
        },
        MeshLink {
            from: "sporeGate".into(),
            to: "eastGate".into(),
            latency_ms: 1,
        },
        MeshLink {
            from: "sporeGate".into(),
            to: "ironGate".into(),
            latency_ms: 1,
        },
        MeshLink {
            from: "sporeGate".into(),
            to: "flockGate".into(),
            latency_ms: 72,
        },
    ]
}

/// All static mesh nodes.
#[cfg(feature = "offline-topology")]
#[must_use]
pub fn all_nodes() -> Vec<MeshNode> {
    static_nodes()
}

/// Count of static nodes matching a given enrollment status.
#[cfg(feature = "offline-topology")]
#[must_use]
pub fn count_by_enrollment(status: GateEnrollment) -> usize {
    static_nodes()
        .iter()
        .filter(|n| n.enrollment == status)
        .count()
}

/// Count of static nodes that are at least mesh-live.
#[cfg(feature = "offline-topology")]
#[must_use]
pub fn mesh_active_count() -> usize {
    static_nodes()
        .iter()
        .filter(|n| {
            matches!(
                n.enrollment,
                GateEnrollment::Enrolled | GateEnrollment::MeshLive
            )
        })
        .count()
}

/// Static nodes with GPU compute capability.
#[cfg(feature = "offline-topology")]
#[must_use]
pub fn gpu_nodes() -> Vec<MeshNode> {
    static_nodes()
        .into_iter()
        .filter(|n| n.gpu_target.is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_topology_consistency() {
        let nodes = static_nodes();
        let links = static_links();
        assert_eq!(nodes.len(), 12);
        assert_eq!(links.len(), 7);
    }

    #[test]
    fn enrollment_counts() {
        assert_eq!(count_by_enrollment(GateEnrollment::Enrolled), 7);
        assert_eq!(mesh_active_count(), 8);
    }

    #[test]
    fn gpu_nodes_identified() {
        let gpu: Vec<String> = gpu_nodes().into_iter().map(|n| n.id).collect();
        assert!(gpu.contains(&"ironGate".to_owned()));
        assert!(gpu.contains(&"northGate".to_owned()));
        assert_eq!(gpu.len(), 2);
    }

    #[test]
    fn all_links_reference_valid_nodes() {
        let nodes = static_nodes();
        let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        for link in &static_links() {
            assert!(ids.contains(&link.from.as_str()), "bad from: {}", link.from);
            assert!(ids.contains(&link.to.as_str()), "bad to: {}", link.to);
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let nodes = static_nodes();
        let json = serde_json::to_string(&nodes[0]).expect("serialize gate");
        assert!(json.contains("sporeGate"));
        assert!(json.contains("enrolled"));
    }

    #[test]
    fn static_mesh_topology_trait() {
        let source = StaticMeshTopology;
        assert_eq!(source.nodes().len(), 12);
        assert_eq!(source.links().len(), 7);
        assert_eq!(source.count_by_enrollment(GateEnrollment::Enrolled), 7);
        assert_eq!(source.mesh_active_count(), 8);
        assert_eq!(source.gpu_nodes().len(), 2);
    }

    #[test]
    fn manifest_topology_from_toml() {
        let toml = r#"
[gates.testGate]
lan_ip = "192.168.1.1"
wg_ip = "10.13.37.99"
enrollment = "enrolled"
nucleus_count = 5
role = "test gate"
kderm_layer = "Cytoplasm"

[gates.offlineGate]
enrollment = "offline"
role = "offline test"
"#;
        let source = ManifestMeshTopology::from_toml(toml);
        let nodes = source.nodes();
        assert_eq!(nodes.len(), 2);
        let test = nodes.iter().find(|n| n.id == "testGate").unwrap();
        assert_eq!(test.lan_ip.as_deref(), Some("192.168.1.1"));
        assert_eq!(test.enrollment, GateEnrollment::Enrolled);
        assert_eq!(test.nucleus_count, 5);
        let offline = nodes.iter().find(|n| n.id == "offlineGate").unwrap();
        assert_eq!(offline.enrollment, GateEnrollment::Offline);
    }

    #[test]
    fn manifest_topology_empty_on_invalid() {
        let source = ManifestMeshTopology::from_toml("not valid toml {{{{");
        assert!(source.nodes().is_empty());
    }

    #[test]
    fn manifest_topology_empty_on_missing_gates() {
        let source = ManifestMeshTopology::from_toml("[ecosystem]\nname = \"test\"");
        assert!(source.nodes().is_empty());
    }
}
