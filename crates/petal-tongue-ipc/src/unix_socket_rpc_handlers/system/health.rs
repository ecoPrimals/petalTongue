// SPDX-License-Identifier: AGPL-3.0-or-later
//! Health-related JSON-RPC handlers.

use super::super::RpcHandlers;
use crate::capability_detection;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

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
/// Per `DEPLOYMENT_VALIDATION_STANDARD.md` v1.0 the response MUST include
/// `version` and `primal` fields alongside `status`.
#[must_use]
pub fn handle_health_readiness(handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    use petal_tongue_core::capability_names::primal_names;

    let graph_ok = handlers.graph.read().is_ok();
    let viz_ok = handlers.viz_state.read().is_ok();
    let ready = graph_ok && viz_ok;

    let status = if ready { "ready" } else { "not_ready" };

    JsonRpcResponse::success(
        request.id,
        json!({
            "status": status,
            "ready": ready,
            "version": env!("CARGO_PKG_VERSION"),
            "primal": primal_names::PETALTONGUE,
            "checks": {
                "graph_engine": graph_ok,
                "visualization_state": viz_ok,
            },
        }),
    )
}

/// Handle health.check: return status, version, primal, uptime, and modalities.
///
/// Per `DEPLOYMENT_VALIDATION_STANDARD.md` v1.0 the response MUST include
/// `version` and `primal` fields.
#[must_use]
pub fn handle_health_check(handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    use petal_tongue_core::capability_names::primal_names;
    use petal_tongue_core::capability_taxonomy::CapabilityTaxonomy;

    let modalities = capability_detection::detect_active_modalities();
    let modality_strs: Vec<&str> = modalities.iter().map(CapabilityTaxonomy::as_str).collect();
    let display_available = modalities.contains(&CapabilityTaxonomy::UIVisualization);

    JsonRpcResponse::success(
        request.id,
        json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "primal": primal_names::PETALTONGUE,
            "uptime_seconds": handlers.uptime_seconds(),
            "display_available": display_available,
            "modalities_active": modality_strs,
        }),
    )
}

/// Handle health.get: return health status and graph stats
pub fn get_health(handlers: &RpcHandlers, id: serde_json::Value) -> JsonRpcResponse {
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
