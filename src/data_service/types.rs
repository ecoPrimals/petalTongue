// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{PrimalInfo, TopologyEdge};

/// Data update notification
#[derive(Clone, Debug)]
pub enum DataUpdate {
    /// Graph topology updated
    TopologyUpdated,
    /// Mesh peer status changed (future: triggered by songBird IPC)
    #[expect(dead_code, reason = "wired for songBird mesh.peers live push")]
    MeshPeersUpdated,
}

/// Complete data snapshot for UI consumption
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DataSnapshot {
    /// Discovered primals
    pub primals: Vec<PrimalInfo>,

    /// Topology edges
    pub edges: Vec<TopologyEdge>,

    /// Timestamp (as seconds since UNIX epoch)
    pub timestamp: u64,
}

/// Live topology snapshot for TOPO-VIS visualization.
///
/// Combines Neural API discovery data with static mesh peer state.
#[derive(Clone, Debug, serde::Serialize)]
pub struct LiveTopology {
    /// Data source: `"neural_api"` or `"static_fallback"`
    pub source: &'static str,
    pub primal_count: usize,
    pub edge_count: usize,
    pub mesh_peer_count: usize,
    pub primals: Vec<LivePrimal>,
    pub edges: Vec<LiveEdge>,
    pub mesh_peers: Vec<LiveMeshPeer>,
    pub timestamp: u64,
}

/// A primal discovered via Neural API.
#[derive(Clone, Debug, serde::Serialize)]
pub struct LivePrimal {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub health: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
}

/// A topology edge (capability invocation path).
#[derive(Clone, Debug, serde::Serialize)]
pub struct LiveEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub capability: Option<String>,
    pub weight: Option<f64>,
}

/// A mesh peer from songBird/gate topology.
#[derive(Clone, Debug, serde::Serialize)]
pub struct LiveMeshPeer {
    pub gate_id: String,
    pub status: String,
    pub transport: String,
    pub latency_ms: Option<u32>,
    pub capabilities: Vec<String>,
}
