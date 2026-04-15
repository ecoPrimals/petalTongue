// SPDX-License-Identifier: AGPL-3.0-or-later
//! Identity and lifecycle JSON-RPC handlers.

use super::super::RpcHandlers;
use crate::json_rpc::JsonRpcResponse;
use serde_json::json;

/// Handle `identity.get`: return primal identity for capability-based discovery.
///
/// Per `CAPABILITY_BASED_DISCOVERY_STANDARD.md` v1.1, every primal MUST implement
/// `identity.get` so sourDough and other discovery agents can identify it.
#[must_use]
pub fn handle_identity_get(handlers: &RpcHandlers, id: serde_json::Value) -> JsonRpcResponse {
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
pub fn handle_lifecycle_status(handlers: &RpcHandlers, id: serde_json::Value) -> JsonRpcResponse {
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
