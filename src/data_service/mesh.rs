// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{GraphEngine, gate_mesh};

use super::types::{LiveEdge, LiveMeshPeer, LivePrimal, LiveTopology};

/// Get current mesh peer state (songBird `mesh.peers` concept).
///
/// Currently returns statically derived peers from the gate topology.
/// When songBird IPC is available, this will query live peer state.
#[must_use]
pub fn mesh_peers() -> Vec<gate_mesh::MeshPeer> {
    gate_mesh::derive_mesh_peers(gate_mesh::all_nodes())
}

/// Get live topology for TOPO-VIS visualization.
///
/// Returns live Neural API data (primals + edges) when available,
/// with static gate mesh data as fallback. This is the primary source
/// for the `/api/topology/live` endpoint.
#[must_use]
pub fn live_topology(has_api: bool, graph: Option<&GraphEngine>) -> LiveTopology {
    const LATENCY_MS_TO_EDGE_WEIGHT_SCALE: f64 = 100.0;

    let (primals, edges) = graph.filter(|g| !g.nodes().is_empty()).map_or_else(
        || (Vec::new(), Vec::new()),
        |g| {
            let primals: Vec<LivePrimal> = g
                .nodes()
                .iter()
                .map(|node| LivePrimal {
                    id: node.info.id.to_string(),
                    name: node.info.name.clone(),
                    primal_type: node.info.primal_type.clone(),
                    health: format!("{:?}", node.info.health),
                    capabilities: node.info.capabilities.clone(),
                    endpoint: node.info.endpoint.clone(),
                })
                .collect();
            let edges: Vec<LiveEdge> = g
                .edges()
                .iter()
                .map(|e| LiveEdge {
                    from: e.from.to_string(),
                    to: e.to.to_string(),
                    edge_type: e.edge_type.clone(),
                    capability: e.capability.clone(),
                    weight: e.weight.or_else(|| {
                        e.metrics
                            .as_ref()
                            .and_then(|m| m.avg_latency_ms)
                            .map(|ms| ms / LATENCY_MS_TO_EDGE_WEIGHT_SCALE)
                    }),
                })
                .collect();
            (primals, edges)
        },
    );

    let mesh_peers = mesh_peers_live();

    LiveTopology {
        source: if has_api && !primals.is_empty() {
            "neural_api"
        } else {
            "static_fallback"
        },
        primal_count: primals.len(),
        edge_count: edges.len(),
        mesh_peer_count: mesh_peers.len(),
        primals,
        edges,
        mesh_peers,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    }
}

fn mesh_peers_live() -> Vec<LiveMeshPeer> {
    mesh_peers()
        .into_iter()
        .map(|p| LiveMeshPeer {
            gate_id: p.gate_id.to_owned(),
            status: format!("{:?}", p.status),
            transport: p.transport.to_owned(),
            latency_ms: if p.latency_ms == u32::MAX {
                None
            } else {
                Some(p.latency_ms)
            },
            capabilities: p.capabilities.iter().map(|&s| s.to_owned()).collect(),
        })
        .collect()
}
