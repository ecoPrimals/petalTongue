// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provider registration JSON-RPC handler.

use super::super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

/// Handle `provider.register_capability`: accept `ProviderRegistry` registrations.
///
/// Springs and primals call this to register their capabilities with petalTongue,
/// conforming to the compute provider's `ProviderRegistry` protocol.
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
