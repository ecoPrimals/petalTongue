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

/// Handle `provider.register_capability`: accept toadStool `ProviderRegistry` registrations.
///
/// Springs and primals call this to register their capabilities with petalTongue,
/// conforming to toadStool S145's `ProviderRegistry` protocol.
pub fn handle_provider_register(
    _handlers: &RpcHandlers,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let capability = request
        .params
        .get("capability")
        .and_then(Value::as_str)
        .unwrap_or("");
    let socket_path = request
        .params
        .get("socket_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    let provider_name = request
        .params
        .get("provider_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let version = request
        .params
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("0.0.0");
    let methods = request
        .params
        .get("methods")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    tracing::info!(
        capability,
        provider_name,
        version,
        method_count = methods.len(),
        "Provider registered capability"
    );

    JsonRpcResponse::success(
        request.id,
        json!({
            "registered": true,
            "capability": capability,
            "provider_name": provider_name,
            "socket_path": socket_path,
            "methods": methods,
            "version": version,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_rpc::JsonRpcRequest;
    use crate::unix_socket_rpc_handlers::RpcHandlers;
    use crate::visualization_handler::VisualizationState;
    use petal_tongue_core::graph_engine::GraphEngine;
    use std::sync::{Arc, RwLock};

    fn test_handlers() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
        RpcHandlers::new(graph, "test-family".to_string(), viz_state)
    }

    #[test]
    fn handle_health_check_returns_healthy() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("health.check", serde_json::json!({}), serde_json::json!(1));
        let resp = handle_health_check(&h, req);
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert_eq!(r["status"], "healthy");
        assert!(
            r["modalities_active"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("terminal"))
        );
    }

    #[test]
    fn handle_announce_capabilities_returns_capabilities() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "capability.announce",
            serde_json::json!({}),
            serde_json::json!(1),
        );
        let resp = handle_announce_capabilities(&h, req);
        let result = resp.result.expect("success response");
        let caps = result["capabilities"]
            .as_array()
            .expect("capabilities array");
        assert!(!caps.is_empty());
    }

    #[test]
    fn get_capabilities_returns_protocol_info() {
        let h = test_handlers();
        let resp = get_capabilities(&h, serde_json::json!(42));
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert_eq!(r["family_id"], "test-family");
        assert_eq!(r["protocol"], "json-rpc-2.0");
        assert!(
            r["capabilities"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("interaction.subscribe"))
        );
    }

    #[test]
    fn get_health_returns_graph_stats() {
        let h = test_handlers();
        let resp = get_health(&h, serde_json::json!(1));
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert_eq!(r["status"], "healthy");
        assert!(r["graph"]["nodes"].as_u64().unwrap() == 0);
        assert!(r["graph"]["edges"].as_u64().unwrap() == 0);
    }

    #[test]
    fn get_topology_returns_nodes_and_edges() {
        let h = test_handlers();
        let resp = get_topology(&h, serde_json::json!(1));
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert!(r["nodes"].is_array());
        assert!(r["edges"].is_array());
    }
}
