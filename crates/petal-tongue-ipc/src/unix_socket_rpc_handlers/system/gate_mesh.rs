// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gate mesh status JSON-RPC handler.
//!
//! Exposes the `gate.mesh.status` method for runtime mesh topology queries.
//! Uses `MeshTopologySource` trait for topology resolution — prefers manifest
//! data over static fallback.

use super::super::RpcHandlers;
use crate::json_rpc::JsonRpcResponse;
use petal_tongue_core::gate_mesh::{self, MeshTopologySource};
use serde_json::json;

/// Handle `gate.mesh.status`: return current mesh enrollment and connectivity.
///
/// Resolution order: manifest file > static fallback (offline-topology feature).
pub fn get_gate_mesh_status(_handlers: &RpcHandlers, id: serde_json::Value) -> JsonRpcResponse {
    let manifest_source = gate_mesh::ManifestMeshTopology::discover();
    let manifest_nodes = manifest_source.nodes();

    let (nodes, links, source_label) = if manifest_nodes.is_empty() {
        let static_source = gate_mesh::StaticMeshTopology;
        (static_source.nodes(), static_source.links(), "offline")
    } else {
        let links = manifest_source.links();
        (manifest_nodes, links, "manifest")
    };

    let gate_json: Vec<serde_json::Value> = nodes
        .iter()
        .map(|node| {
            json!({
                "id": node.id,
                "zone": node.zone,
                "wg_ip": node.wg_ip,
                "enrollment": node.enrollment,
                "nucleus_count": node.nucleus_count,
            })
        })
        .collect();

    let link_json: Vec<serde_json::Value> = links
        .iter()
        .map(|link| {
            json!({
                "from": link.from,
                "to": link.to,
                "latency_ms": link.latency_ms,
            })
        })
        .collect();

    let enrolled = nodes
        .iter()
        .filter(|n| n.enrollment == gate_mesh::GateEnrollment::Enrolled)
        .count();
    let mesh_live = nodes
        .iter()
        .filter(|n| {
            matches!(
                n.enrollment,
                gate_mesh::GateEnrollment::Enrolled | gate_mesh::GateEnrollment::MeshLive
            )
        })
        .count();

    let result = json!({
        "gates": gate_json,
        "links": link_json,
        "enrolled_count": enrolled,
        "mesh_live_count": mesh_live,
        "total_count": nodes.len(),
        "source": source_label,
    });

    JsonRpcResponse::success(id, result)
}
