// SPDX-License-Identifier: AGPL-3.0-or-later
//! Topology JSON-RPC handler.

use super::super::RpcHandlers;
use crate::json_rpc::JsonRpcResponse;
use serde_json::json;

/// Handle topology.get: return graph nodes and edges
#[expect(
    clippy::significant_drop_tightening,
    reason = "graph used in json! macro for nodes/edges"
)]
pub fn get_topology(handlers: &RpcHandlers, id: serde_json::Value) -> JsonRpcResponse {
    let graph = handlers
        .graph
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let topology = json!({
        "nodes": graph.nodes().iter().map(|node| {
            json!({
                "id": node.info.id,
                "name": node.info.name,
                "type": node.info.primal_type,
                "capabilities": node.info.capabilities,
                "health": format!("{:?}", node.info.health),
                "position": {
                    "x": node.position.x,
                    "y": node.position.y,
                    "z": node.position.z
                }
            })
        }).collect::<Vec<_>>(),
        "edges": graph.edges().iter().map(|edge| {
            json!({
                "from": edge.from,
                "to": edge.to,
                "type": edge.edge_type
            })
        }).collect::<Vec<_>>()
    });

    JsonRpcResponse::success(id, topology)
}
