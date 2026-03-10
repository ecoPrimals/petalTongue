// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for health, capability, and topology JSON-RPC methods.

use super::RpcHandlers;
use crate::capability_detection;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

/// Handle health.check: return status, version, uptime, and modalities
#[must_use]
pub fn handle_health_check(handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    let modalities = capability_detection::detect_active_modalities();

    JsonRpcResponse::success(
        request.id,
        json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": handlers.uptime_seconds(),
            "display_available": modalities.contains(&"visual"),
            "modalities_active": modalities,
        }),
    )
}

/// Handle capability.announce: return detected capabilities
#[must_use]
pub fn handle_announce_capabilities(
    _handlers: &RpcHandlers,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let capabilities = capability_detection::detect_capabilities();

    JsonRpcResponse::success(
        request.id,
        json!({
            "capabilities": capabilities,
        }),
    )
}

/// Handle capability.list: return supported capabilities and protocol info
#[must_use]
pub fn get_capabilities(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        json!({
            "capabilities": [
                "interaction.subscribe",
                "ui.desktop-interface",
                "ui.primal-interaction",
                "visualization.graph-rendering",
                "visualization.real-time-topology",
                "visualization.flow-animation",
                "ui.multi-modal",
                "ui.awakening-experience",
                "visualization.terminal",
                "visualization.svg",
                "visualization.png",
                "visualization.egui"
            ],
            "version": env!("CARGO_PKG_VERSION"),
            "family_id": &handlers.family_id,
            "protocol": "json-rpc-2.0",
            "transport": "unix-socket"
        }),
    )
}

/// Handle health.get: return health status and graph stats
pub fn get_health(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let graph = handlers
        .graph
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let node_count = graph.nodes().len();
    let edge_count = graph.edges().len();

    JsonRpcResponse::success(
        id,
        json!({
            "status": "healthy",
            "family_id": &handlers.family_id,
            "graph": {
                "nodes": node_count,
                "edges": edge_count
            },
            "protocol": "json-rpc-2.0"
        }),
    )
}

/// Handle topology.get: return graph nodes and edges
pub fn get_topology(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
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
