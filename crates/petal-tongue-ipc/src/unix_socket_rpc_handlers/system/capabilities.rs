// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability discovery and sensory matrix JSON-RPC handlers.

use super::super::RpcHandlers;
use crate::capability_detection;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

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
/// Follows ecosystem `capability.list` standard (Wire Standard L2/L3 pattern):
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
                // System
                "health.check",
                "health.liveness",
                "health.readiness",
                "health.get",
                "identity.get",
                "lifecycle.status",
                "capabilities.list",
                "capability.announce",
                "capabilities.sensory",
                "capabilities.sensory.negotiate",
                "topology.get",
                "provider.register_capability",
                // Visualization
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
                "visualization.render.graph",
                // Interaction
                "interaction.subscribe",
                "interaction.poll",
                "interaction.unsubscribe",
                "interaction.sensor_stream.subscribe",
                "interaction.sensor_stream.unsubscribe",
                "interaction.sensor_stream.poll",
                // Audio
                "audio.synthesize",
                // UI
                "ui.render",
                "ui.display_status",
                // Motor
                "motor.set_panel",
                "motor.set_zoom",
                "motor.fit_to_view",
                "motor.set_mode",
                "motor.navigate",
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
                "capabilities.list": { "cpu_ms": 0.01, "gpu_eligible": false },
            },
        }),
    )
}

/// Handle `capabilities.sensory`: return the full sensory capability matrix.
///
/// Consumer primals and springs call this to discover
/// what input/output paths are available for the current user/session.
///
/// Optional `"agent": true` param returns an agent-only matrix.
#[must_use]
pub fn handle_capabilities_sensory(
    _handlers: &RpcHandlers,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let is_agent = request
        .params
        .get("agent")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let matrix = if is_agent {
        petal_tongue_core::SensoryCapabilityMatrix::for_agent()
    } else {
        let caps = petal_tongue_core::SensoryCapabilities::discover().unwrap_or_default();
        petal_tongue_core::SensoryCapabilityMatrix::from_sensory_capabilities(&caps)
    };

    let value = serde_json::to_value(&matrix).unwrap_or(json!(null));
    JsonRpcResponse::success(request.id, value)
}

/// Handle `capabilities.sensory.negotiate`: accept input/output overrides and
/// return a tailored matrix. Primals call this when they already know the
/// user's capabilities (e.g. from storage provider preferences).
#[must_use]
pub fn handle_capabilities_sensory_negotiate(
    _handlers: &RpcHandlers,
    mut request: JsonRpcRequest,
) -> JsonRpcResponse {
    let input: petal_tongue_core::InputCapabilitySet = request
        .params
        .as_object_mut()
        .and_then(|m| m.remove("input"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let output: petal_tongue_core::OutputCapabilitySet = request
        .params
        .as_object_mut()
        .and_then(|m| m.remove("output"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let validated_paths =
        petal_tongue_core::SensoryCapabilityMatrix::compute_validated_paths_public(&input, &output);
    let recommended =
        petal_tongue_core::SensoryCapabilityMatrix::recommend_modality_public(&input, &output);
    let patterns = petal_tongue_core::SensoryCapabilityMatrix::compute_patterns_public(&input);

    let matrix = petal_tongue_core::SensoryCapabilityMatrix {
        input,
        output,
        validated_paths,
        recommended_modality: recommended,
        interaction_patterns: patterns,
    };

    let value = serde_json::to_value(&matrix).unwrap_or(json!(null));
    JsonRpcResponse::success(request.id, value)
}
