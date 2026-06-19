// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gate mesh status JSON-RPC handler.
//!
//! Exposes the `gate.mesh.status` method for runtime mesh topology queries.
//! Consumes shared topology data from `petal_tongue_core::gate_mesh`.

use super::super::RpcHandlers;
use crate::json_rpc::JsonRpcResponse;
use petal_tongue_core::gate_mesh;
use serde_json::json;

/// Handle `gate.mesh.status`: return current mesh enrollment and connectivity.
pub fn get_gate_mesh_status(_handlers: &RpcHandlers, id: serde_json::Value) -> JsonRpcResponse {
    let gate_json: Vec<serde_json::Value> = gate_mesh::all_nodes()
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

    let link_json: Vec<serde_json::Value> = gate_mesh::WG_LINKS
        .iter()
        .map(|link| {
            json!({
                "from": link.from,
                "to": link.to,
                "latency_ms": link.latency_ms,
            })
        })
        .collect();

    let result = json!({
        "gates": gate_json,
        "links": link_json,
        "enrolled_count": gate_mesh::count_by_enrollment(gate_mesh::GateEnrollment::Enrolled),
        "mesh_live_count": gate_mesh::mesh_active_count(),
        "total_count": gate_mesh::all_nodes().count(),
        "source": "offline",
    });

    JsonRpcResponse::success(id, result)
}
