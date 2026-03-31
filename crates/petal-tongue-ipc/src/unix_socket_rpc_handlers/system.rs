// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for health, capability, and topology JSON-RPC methods.

use super::RpcHandlers;
use crate::capability_detection;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

/// Handle health.liveness: lightweight probe for Kubernetes/biomeOS — is the process alive?
///
/// Per `SEMANTIC_METHOD_NAMING_STANDARD.md` v2.2, the canonical response is
/// `{"status": "alive"}`. The `alive` boolean is kept for backward compat.
#[must_use]
pub fn handle_health_liveness(_handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        json!({
            "status": "alive",
            "alive": true,
        }),
    )
}

/// Handle health.readiness: is the primal ready to serve requests?
///
/// Per `SEMANTIC_METHOD_NAMING_STANDARD.md` v2.2, the canonical response is
/// `{"status": "ready", ...}`. Checks graph engine and viz state accessibility.
#[must_use]
pub fn handle_health_readiness(handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    let graph_ok = handlers.graph.read().is_ok();
    let viz_ok = handlers.viz_state.read().is_ok();
    let ready = graph_ok && viz_ok;

    let status = if ready { "ready" } else { "not_ready" };

    JsonRpcResponse::success(
        request.id,
        json!({
            "status": status,
            "ready": ready,
            "checks": {
                "graph_engine": graph_ok,
                "visualization_state": viz_ok,
            },
        }),
    )
}

/// Handle health.check: return status, version, uptime, and modalities
#[must_use]
pub fn handle_health_check(handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    use petal_tongue_core::capability_taxonomy::CapabilityTaxonomy;

    let modalities = capability_detection::detect_active_modalities();
    let modality_strs: Vec<&str> = modalities.iter().map(CapabilityTaxonomy::as_str).collect();
    let display_available = modalities.contains(&CapabilityTaxonomy::UIVisualization);

    JsonRpcResponse::success(
        request.id,
        json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": handlers.uptime_seconds(),
            "display_available": display_available,
            "modalities_active": modality_strs,
        }),
    )
}

/// Handle capability.announce: return detected capabilities
#[must_use]
pub fn handle_announce_capabilities(
    _handlers: &RpcHandlers,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let capability_strs = capability_detection::detect_capability_strings();

    JsonRpcResponse::success(
        request.id,
        json!({
            "capabilities": capability_strs,
        }),
    )
}

/// Handle capability.list: return supported capabilities with enriched metadata.
///
/// Follows ecosystem `capability.list` standard (loamSpine/sweetGrass pattern):
/// returns version, protocol, transport, methods, and dependency info.
#[must_use]
pub fn get_capabilities(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    use petal_tongue_core::capability_names::{
        discovery_capabilities, methods, primal_names, self_capabilities,
    };

    let mut transport = vec!["unix-socket"];
    if handlers.tcp_enabled {
        transport.push("tcp");
    }

    JsonRpcResponse::success(
        id,
        json!({
            "primal": primal_names::PETALTONGUE,
            "version": env!("CARGO_PKG_VERSION"),
            "family_id": &handlers.family_id,
            "protocol": "json-rpc-2.0",
            "transport": transport,
            "capabilities": self_capabilities::ALL,
            "methods": [
                "health.check",
                "health.liveness",
                "health.readiness",
                "health.get",
                "topology.get",
                methods::VISUALIZATION_RENDER,
                methods::VISUALIZATION_RENDER_STREAM,
                methods::VISUALIZATION_RENDER_GRAMMAR,
                methods::VISUALIZATION_RENDER_DASHBOARD,
                methods::VISUALIZATION_RENDER_SCENE,
                methods::VISUALIZATION_VALIDATE,
                methods::VISUALIZATION_EXPORT,
                methods::VISUALIZATION_CAPABILITIES,
                methods::VISUALIZATION_DISMISS,
                methods::VISUALIZATION_INTERACT_APPLY,
                methods::VISUALIZATION_INTERACT_PERSPECTIVES,
                methods::VISUALIZATION_INTROSPECT,
                methods::VISUALIZATION_PANELS,
                methods::VISUALIZATION_SHOWING,
                methods::VISUALIZATION_SESSION_LIST,
                methods::VISUALIZATION_SESSION_STATUS,
            ],
            "depends_on": [
                { "capability": discovery_capabilities::DISPLAY_BACKEND, "required": false },
                { "capability": discovery_capabilities::GPU_DISPATCH, "required": false },
                { "capability": discovery_capabilities::SHADER_COMPILE, "required": false },
            ],
            "data_bindings": 11,
            "geometry_types": 10,
            "operation_dependencies": {
                "visualization.render.dashboard": ["visualization.render"],
                "visualization.render.grammar": ["visualization.render"],
                "visualization.render.scene": ["visualization.render"],
                "visualization.export": ["visualization.render"],
                "visualization.interact.apply": ["interaction.subscribe"],
            },
            "cost_estimates": {
                "visualization.render": { "cpu_ms": 1.0, "gpu_eligible": true },
                "visualization.validate": { "cpu_ms": 0.5, "gpu_eligible": false },
                "visualization.export": { "cpu_ms": 5.0, "gpu_eligible": true },
                "health.check": { "cpu_ms": 0.01, "gpu_eligible": false },
                "capability.list": { "cpu_ms": 0.01, "gpu_eligible": false },
            },
        }),
    )
}

/// Handle health.get: return health status and graph stats
pub fn get_health(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let (node_count, edge_count) = {
        let graph = handlers
            .graph
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (graph.nodes().len(), graph.edges().len())
    };

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
#[expect(
    clippy::significant_drop_tightening,
    reason = "graph used in json! macro for nodes/edges"
)]
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

/// Handle `identity.get`: return primal identity for capability-based discovery.
///
/// Per `CAPABILITY_BASED_DISCOVERY_STANDARD.md` v1.1, every primal MUST implement
/// `identity.get` so sourDough and other discovery agents can identify it.
#[must_use]
pub fn handle_identity_get(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    use petal_tongue_core::capability_names::primal_names;

    JsonRpcResponse::success(
        id,
        json!({
            "primal": primal_names::PETALTONGUE,
            "version": env!("CARGO_PKG_VERSION"),
            "family_id": &handlers.family_id,
            "protocol": "json-rpc-2.0",
            "uptime_seconds": handlers.uptime_seconds(),
        }),
    )
}

/// Handle `lifecycle.status`: return current lifecycle state.
///
/// Per `PRIMALSPRING_COMPOSITION_GUIDANCE.md`, primals must expose lifecycle
/// status so primalSpring can validate composition health during gate probes.
#[must_use]
pub fn handle_lifecycle_status(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let graph_ok = handlers.graph.read().is_ok();

    JsonRpcResponse::success(
        id,
        json!({
            "state": "running",
            "uptime_seconds": handlers.uptime_seconds(),
            "healthy": graph_ok,
        }),
    )
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
                .contains(&serde_json::json!("ui.terminal"))
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

    #[test]
    fn handle_provider_register_full_params() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "provider.register_capability",
            serde_json::json!({
                "capability": "gpu.dispatch",
                "socket_path": "/tmp/gpu.sock",
                "provider_name": "barracuda",
                "version": "0.3.3",
                "methods": ["compute.dispatch", "health.check"]
            }),
            serde_json::json!(1),
        );
        let resp = handle_provider_register(&h, req);
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert_eq!(r["registered"], true);
        assert_eq!(r["capability"], "gpu.dispatch");
        assert_eq!(r["provider_name"], "barracuda");
        assert_eq!(r["socket_path"], "/tmp/gpu.sock");
        assert_eq!(r["version"], "0.3.3");
        let methods = r["methods"].as_array().expect("methods");
        assert_eq!(methods.len(), 2);
    }

    #[test]
    fn handle_provider_register_missing_params_uses_defaults() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "provider.register_capability",
            serde_json::json!({}),
            serde_json::json!(1),
        );
        let resp = handle_provider_register(&h, req);
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert_eq!(r["capability"], "");
        assert_eq!(r["provider_name"], "unknown");
        assert_eq!(r["version"], "0.0.0");
        assert!(r["methods"].as_array().unwrap().is_empty());
    }

    #[test]
    fn handle_health_liveness_returns_alive() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "health.liveness",
            serde_json::json!({}),
            serde_json::json!(1),
        );
        let resp = handle_health_liveness(&h, req);
        let r = resp.result.expect("success");
        assert_eq!(r["status"], "alive");
        assert_eq!(r["alive"], true);
    }

    #[test]
    fn handle_health_readiness_returns_ready() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "health.readiness",
            serde_json::json!({}),
            serde_json::json!(1),
        );
        let resp = handle_health_readiness(&h, req);
        let r = resp.result.expect("success");
        assert_eq!(r["status"], "ready");
        assert_eq!(r["ready"], true);
        assert_eq!(r["checks"]["graph_engine"], true);
        assert_eq!(r["checks"]["visualization_state"], true);
    }

    #[test]
    fn handle_identity_get_returns_primal_info() {
        let h = test_handlers();
        let resp = handle_identity_get(&h, serde_json::json!(1));
        let r = resp.result.expect("success");
        assert_eq!(r["primal"], "petaltongue");
        assert!(r["version"].as_str().is_some());
        assert_eq!(r["family_id"], "test-family");
        assert_eq!(r["protocol"], "json-rpc-2.0");
    }

    #[test]
    fn handle_lifecycle_status_returns_running() {
        let h = test_handlers();
        let resp = handle_lifecycle_status(&h, serde_json::json!(1));
        let r = resp.result.expect("success");
        assert_eq!(r["state"], "running");
        assert_eq!(r["healthy"], true);
        assert!(r["uptime_seconds"].as_u64().is_some());
    }
}
