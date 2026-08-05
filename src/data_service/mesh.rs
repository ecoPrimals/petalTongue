// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::gate_mesh::MeshTopologySource;
use petal_tongue_core::{GraphEngine, gate_mesh};

use super::types::{LiveEdge, LiveMeshPeer, LivePrimal, LiveTopology};

const SONGBIRD_SOCKET: &str = "/run/membrane/songbird.sock";

/// Get current mesh peer state via manifest topology (static fallback).
///
/// Loads topology from `ecosystem_manifest.toml` at runtime and derives
/// peer connectivity.
#[must_use]
pub fn mesh_peers() -> Vec<gate_mesh::MeshPeer> {
    let source = gate_mesh::ManifestMeshTopology::discover();
    let nodes = source.nodes();
    gate_mesh::derive_mesh_peers(&nodes)
}

/// Query songBird `mesh.peers` via UDS JSON-RPC for live peer state.
///
/// Returns `None` if the socket is unavailable or the RPC fails,
/// allowing callers to fall back to static topology.
pub async fn query_songbird_peers() -> Option<serde_json::Value> {
    use tokio::net::UnixStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = UnixStream::connect(SONGBIRD_SOCKET).await.ok()?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "mesh.peers",
        "params": {},
        "id": 1
    });
    let payload = serde_json::to_vec(&request).ok()?;

    stream.write_all(&payload).await.ok()?;
    stream.shutdown().await.ok()?;

    let mut buf = Vec::with_capacity(8192);
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        stream.read_to_end(&mut buf),
    )
    .await
    .ok()?
    .ok()?;

    let response: serde_json::Value = serde_json::from_slice(&buf).ok()?;
    response.get("result").cloned()
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

    let source = if has_api && !primals.is_empty() {
        "neural_api"
    } else {
        "static_fallback"
    };

    LiveTopology {
        source,
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
            gate_id: p.gate_id,
            status: format!("{:?}", p.status),
            transport: p.transport,
            latency_ms: if p.latency_ms == u32::MAX {
                None
            } else {
                Some(p.latency_ms)
            },
            capabilities: p.capabilities,
        })
        .collect()
}

/// Convert songBird live peer data to `LiveMeshPeer` format.
pub fn songbird_peers_to_live(result: &serde_json::Value) -> Vec<LiveMeshPeer> {
    result
        .get("peers")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .map(|p| LiveMeshPeer {
            gate_id: p["node_id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            status: "connected".to_string(),
            transport: p["path_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            latency_ms: p["priority"].as_u64().map(|v| v as u32),
            capabilities: Vec::new(),
        })
        .collect()
}
