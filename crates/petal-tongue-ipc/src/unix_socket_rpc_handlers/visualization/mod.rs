// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for visualization.* JSON-RPC methods.
//!
//! Delegates to `VisualizationState` and `InteractionSubscriberRegistry` for
//! session lifecycle, grammar rendering, validation, export, and interaction.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::visualization_handler::{
    DashboardRenderRequest, ExportRequest, GrammarRenderRequest, InteractionApplyRequest,
    StreamUpdateRequest, UiConfig, ValidateRequest, VisualizationRenderRequest,
};
use serde_json::Value;

#[cfg(test)]
mod tests;

/// Handle visualization.introspect: return the full frame introspection snapshot
#[expect(
    clippy::significant_drop_tightening,
    reason = "content() borrows from awareness guard"
)]
pub fn handle_introspect(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let Some(awareness_arc) = &handlers.rendering_awareness else {
        return JsonRpcResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            "Rendering awareness not wired to IPC".to_string(),
        );
    };
    let awareness = awareness_arc
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let content = awareness.content();
    match content.current() {
        Some(frame) => {
            let panels: Vec<_> = frame
                .visible_panels
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "kind": p.kind,
                        "visible": p.visible,
                        "data_source": p.data_source,
                        "widget_count": p.widget_count,
                    })
                })
                .collect();
            let bound_data: Vec<_> = frame
                .bound_data
                .iter()
                .map(|b| {
                    serde_json::json!({
                        "panel_id": b.panel_id,
                        "data_object_id": b.data_object_id,
                        "binding_type": b.binding_type,
                    })
                })
                .collect();
            let interactions: Vec<_> = frame
                .possible_interactions
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "panel_id": i.panel_id,
                        "intent": i.intent,
                        "target": i.target,
                    })
                })
                .collect();
            JsonRpcResponse::success(
                id,
                serde_json::json!({
                    "frame_id": frame.frame_id,
                    "visible_panels": panels,
                    "bound_data": bound_data,
                    "possible_interactions": interactions,
                    "visible_panel_count": frame.visible_panel_count(),
                }),
            )
        }
        None => JsonRpcResponse::success(
            id,
            serde_json::json!({
                "frame_id": null,
                "visible_panels": [],
                "bound_data": [],
                "possible_interactions": [],
                "visible_panel_count": 0,
            }),
        ),
    }
}

/// Handle visualization.panels: return just the visible panel list
#[expect(
    clippy::significant_drop_tightening,
    reason = "visible_panels() borrows from awareness guard"
)]
pub fn handle_panels(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let Some(awareness_arc) = &handlers.rendering_awareness else {
        return JsonRpcResponse::success(id, serde_json::json!({ "panels": [] }));
    };
    let awareness = awareness_arc
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let panel_ids = awareness.visible_panels();
    let panels: Vec<_> = panel_ids.iter().map(|p| serde_json::json!(p)).collect();
    JsonRpcResponse::success(id, serde_json::json!({ "panels": panels }))
}

/// Handle visualization.showing: check if a data object is displayed
pub fn handle_showing(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let data_id = req
        .params
        .get("data_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    let showing = handlers.rendering_awareness.as_ref().is_some_and(|arc| {
        arc.read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_showing_data(data_id)
    });

    JsonRpcResponse::success(
        req.id,
        serde_json::json!({ "showing": showing, "data_id": data_id }),
    )
}

/// Handle visualization.render: create or replace a visualization session.
///
/// Accepts either the canonical `VisualizationRenderRequest` format or any
/// spring-native format recognized by `SpringDataAdapter` (ludoSpring game
/// channels, ecoPrimals/time-series/v1, bare `DataBinding` arrays).
pub fn handle_render(handlers: &RpcHandlers, mut req: JsonRpcRequest) -> JsonRpcResponse {
    let has_bindings = req.params.get("bindings").is_some();
    let params = if has_bindings {
        match serde_json::from_value::<VisualizationRenderRequest>(req.params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid params: {e}"),
                );
            }
        }
    } else {
        let session_id = req
            .params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("spring-session")
            .to_string();
        let title = req
            .params
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Spring Data")
            .to_string();
        let domain = req
            .params
            .get("domain")
            .and_then(|v| v.as_str())
            .map(String::from);
        let ui_config = req
            .params
            .as_object_mut()
            .and_then(|m| m.remove("ui_config"))
            .and_then(|v| serde_json::from_value::<UiConfig>(v).ok());
        let thresholds = req
            .params
            .as_object_mut()
            .and_then(|m| m.remove("thresholds"))
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        match petal_tongue_core::spring_adapter::SpringDataAdapter::adapt(req.params) {
            Ok(bindings) if !bindings.is_empty() => VisualizationRenderRequest {
                session_id,
                title,
                bindings,
                thresholds,
                domain,
                ui_config,
            },
            _ => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    "Invalid params: unrecognized spring format".to_string(),
                );
            }
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_render(params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.render.stream: incremental update to a binding
pub fn handle_stream(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<StreamUpdateRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_stream_update(params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.render.grammar: compile grammar expression through scene engine
pub fn handle_grammar_render(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<GrammarRenderRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid grammar params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_grammar_render(params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.render.dashboard: compile bindings into a multi-panel dashboard
pub fn handle_dashboard_render(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<DashboardRenderRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid dashboard params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_dashboard_render(params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.validate: validate grammar + data against Tufte constraints
pub fn handle_validate(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<ValidateRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_validate(&params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.export: export a session to the requested format
pub fn handle_export(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<ExportRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_export(params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.interact.apply: apply interaction intent and broadcast (PT-06).
///
/// Produces callback dispatches for subscribers with `callback_method` set.
/// Dispatches with a `callback_socket` are sent to the push delivery background
/// task as JSON-RPC notifications. Subscribers without a socket still use poll.
pub fn handle_interact_apply(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<InteractionApplyRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {e}"),
            );
        }
    };
    let (response, callbacks) = handlers
        .interaction_subscribers
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .apply_interaction(&params);

    if !callbacks.is_empty() {
        if let Some(tx) = &handlers.callback_tx {
            for cb in callbacks {
                if let Err(e) = tx.send(cb) {
                    tracing::warn!("push delivery channel closed: {e}");
                    break;
                }
            }
        } else {
            tracing::debug!(
                count = callbacks.len(),
                "callback dispatches produced (no push delivery channel — poll only)"
            );
        }
    }

    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.interact.perspectives: return available perspectives
pub fn handle_interact_perspectives(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let perspectives = handlers
        .interaction_subscribers
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .perspectives();
    JsonRpcResponse::success(id, serde_json::json!({ "perspectives": perspectives }))
}

/// Handle visualization.capabilities: return supported `DataBinding` variant names
pub fn handle_capabilities(_handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let variants = [
        "TimeSeries",
        "Distribution",
        "Bar",
        "Gauge",
        "Heatmap",
        "Scatter3D",
        "Scatter",
        "FieldMap",
        "Spectrum",
    ];
    let grammar_geometry = [
        "Point", "Line", "Bar", "Area", "Ribbon", "Tile", "Arc", "ErrorBar", "Mesh3D", "Sphere",
        "Cylinder", "Text",
    ];
    let output_modalities = ["svg", "html", "audio", "description"];
    let tufte_constraints = ["DataInkRatio", "ChartjunkDetection"];
    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "data_binding_variants": variants,
            "grammar_geometry_types": grammar_geometry,
            "output_modalities": output_modalities,
            "tufte_constraints": tufte_constraints,
            "scene_engine": true,
        }),
    )
}

/// Handle visualization.render.scene: directly submit a serialized `SceneGraph`.
///
/// Bypasses the grammar/data-binding pipeline; the `SceneGraph` is stored directly
/// so springs can submit arbitrary visual scenes.
pub fn handle_render_scene(handlers: &RpcHandlers, mut req: JsonRpcRequest) -> JsonRpcResponse {
    let session_id = req
        .params
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or("scene-session")
        .to_string();

    let Some(scene_value) = req.params.as_object_mut().and_then(|m| m.remove("scene")) else {
        return JsonRpcResponse::error(
            req.id,
            error_codes::INVALID_PARAMS,
            "Missing 'scene' field",
        );
    };

    let scene: petal_tongue_scene::scene_graph::SceneGraph =
        match serde_json::from_value(scene_value) {
            Ok(s) => s,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid SceneGraph: {e}"),
                );
            }
        };

    let node_count = scene.node_count();
    {
        let mut state = handlers
            .viz_state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.grammar_scenes.insert(session_id.clone(), scene);
    }

    JsonRpcResponse::success(
        req.id,
        serde_json::json!({
            "session_id": session_id,
            "nodes_accepted": node_count,
            "status": "scene_stored",
        }),
    )
}
