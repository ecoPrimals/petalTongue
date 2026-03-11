// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for visualization.* JSON-RPC methods.
//!
//! Delegates to `VisualizationState` and `InteractionSubscriberRegistry` for
//! session lifecycle, grammar rendering, validation, export, and interaction.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::visualization_handler::{
    DashboardRenderRequest, DismissRequest, ExportRequest, GrammarRenderRequest,
    InteractionApplyRequest, StreamUpdateRequest, ValidateRequest, VisualizationRenderRequest,
};
use serde_json::Value;

/// Handle visualization.introspect: return the full frame introspection snapshot
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

/// Handle visualization.render: create or replace a visualization session
pub fn handle_render(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<VisualizationRenderRequest>(req.params) {
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

/// Handle visualization.dismiss: remove a session
pub fn handle_dismiss(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<DismissRequest>(req.params) {
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
        .handle_dismiss(params);
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

/// Handle visualization.interact.apply: apply interaction intent and broadcast
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
    let response = handlers
        .interaction_subscribers
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .apply_interaction(&params);
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

/// Handle visualization.capabilities: return supported DataBinding variant names
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
    let output_modalities = ["svg", "audio", "description"];
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_rpc::{JsonRpcRequest, error_codes};
    use crate::unix_socket_rpc_handlers::RpcHandlers;
    use petal_tongue_core::graph_engine::GraphEngine;
    use petal_tongue_core::{
        BindingType, InteractionKind, PanelId, RenderingAwareness,
        frame_introspection::{
            BoundDataObject, FrameIntrospection, InteractionCapability, PanelKind, PanelSnapshot,
        },
    };
    use serde_json::json;
    use std::sync::{Arc, RwLock};

    fn test_handlers() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(
            crate::visualization_handler::VisualizationState::new(),
        ));
        let mut h = RpcHandlers::new(graph, String::new(), viz_state);
        h.interaction_subscribers = Arc::new(RwLock::new(
            crate::visualization_handler::InteractionSubscriberRegistry::new(),
        ));
        h
    }

    #[test]
    fn handle_introspect_no_awareness_returns_error() {
        let mut h = test_handlers();
        h.rendering_awareness = None;
        let resp = handle_introspect(&h, json!(1));
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("error").code,
            error_codes::INTERNAL_ERROR
        );
        assert!(resp.result.is_none());
    }

    #[test]
    fn handle_introspect_empty_content_returns_null_frame() {
        let mut h = test_handlers();
        let awareness = Arc::new(RwLock::new(RenderingAwareness::new()));
        h.rendering_awareness = Some(awareness);

        let resp = handle_introspect(&h, json!(42));
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["frame_id"], json!(null));
        assert!(r["visible_panels"].as_array().expect("array").is_empty());
        assert_eq!(r["visible_panel_count"], 0);
    }

    #[test]
    fn handle_introspect_with_frame_returns_panels_and_data() {
        let mut h = test_handlers();
        let mut awareness = RenderingAwareness::new();
        let mut frame = FrameIntrospection::empty(100);
        frame.visible_panels.push(PanelSnapshot {
            id: PanelId::LeftSidebar,
            kind: PanelKind::Controls,
            visible: true,
            data_source: Some("graph".into()),
            widget_count: 3,
        });
        frame.bound_data.push(BoundDataObject {
            panel_id: PanelId::Custom("canvas".into()),
            data_object_id: "obj-1".into(),
            binding_type: BindingType::GraphNode,
        });
        frame.possible_interactions.push(InteractionCapability {
            panel_id: PanelId::Custom("canvas".into()),
            intent: InteractionKind::Select,
            target: Some("node-a".into()),
        });
        awareness.record_frame_content(frame);
        h.rendering_awareness = Some(Arc::new(RwLock::new(awareness)));

        let resp = handle_introspect(&h, json!(1));
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["frame_id"], 100);
        assert_eq!(r["visible_panel_count"], 1);
        let panels = r["visible_panels"].as_array().expect("panels");
        assert_eq!(panels.len(), 1);
        assert_eq!(panels[0]["widget_count"], 3);
    }

    #[test]
    fn handle_panels_no_awareness_returns_empty() {
        let mut h = test_handlers();
        h.rendering_awareness = None;
        let resp = handle_panels(&h, json!(1));
        assert!(resp.error.is_none());
        assert!(
            resp.result.expect("result")["panels"]
                .as_array()
                .expect("arr")
                .is_empty()
        );
    }

    #[test]
    fn handle_panels_with_awareness_returns_visible_panels() {
        let mut h = test_handlers();
        let mut awareness = RenderingAwareness::new();
        let mut frame = FrameIntrospection::empty(1);
        frame
            .visible_panels
            .push(PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu));
        awareness.record_frame_content(frame);
        h.rendering_awareness = Some(Arc::new(RwLock::new(awareness)));

        let resp = handle_panels(&h, json!(1));
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        let panels = r["panels"].as_array().expect("arr");
        assert_eq!(panels.len(), 1);
    }

    #[test]
    fn handle_showing_missing_data_id_uses_empty_string() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("viz.showing", json!({}), json!(1));
        let resp = handle_showing(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["data_id"], "");
        assert_eq!(r["showing"], false);
    }

    #[test]
    fn handle_showing_with_data_id_no_awareness_returns_false() {
        let mut h = test_handlers();
        h.rendering_awareness = None;
        let req = JsonRpcRequest::new("viz.showing", json!({"data_id": "my-data"}), json!(1));
        let resp = handle_showing(&h, req);
        let r = resp.result.expect("result");
        assert_eq!(r["showing"], false);
        assert_eq!(r["data_id"], "my-data");
    }

    #[test]
    fn handle_showing_with_data_id_and_awareness_showing() {
        let mut h = test_handlers();
        let mut awareness = RenderingAwareness::new();
        let mut frame = FrameIntrospection::empty(1);
        frame.bound_data.push(BoundDataObject {
            panel_id: PanelId::Custom("canvas".into()),
            data_object_id: "bound-123".into(),
            binding_type: BindingType::GraphNode,
        });
        awareness.record_frame_content(frame);
        h.rendering_awareness = Some(Arc::new(RwLock::new(awareness)));

        let req = JsonRpcRequest::new("viz.showing", json!({"data_id": "bound-123"}), json!(1));
        let resp = handle_showing(&h, req);
        assert_eq!(resp.result.expect("result")["showing"], true);
    }

    #[test]
    fn handle_capabilities_returns_all_variants() {
        let h = test_handlers();
        let resp = handle_capabilities(&h, json!(1));
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert!(
            r["data_binding_variants"]
                .as_array()
                .expect("arr")
                .contains(&json!("TimeSeries"))
        );
        assert!(
            r["grammar_geometry_types"]
                .as_array()
                .expect("arr")
                .contains(&json!("Line"))
        );
        assert!(
            r["output_modalities"]
                .as_array()
                .expect("arr")
                .contains(&json!("svg"))
        );
        assert_eq!(r["scene_engine"], true);
    }

    #[test]
    fn handle_render_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.render",
            json!({
                "session_id": "s1",
                "title": "Test",
                "bindings": []
            }),
            json!(1),
        );
        let resp = handle_render(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["session_id"], "s1");
        assert_eq!(r["bindings_accepted"], 0);
    }

    #[test]
    fn handle_render_invalid_params_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.render",
            json!({"session_id": 123, "title": "x"}), // bindings missing, session_id wrong type
            json!(1),
        );
        let resp = handle_render(&h, req);
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[test]
    fn handle_stream_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.render.stream",
            json!({
                "session_id": "s1",
                "binding_id": "b1",
                "operation": {"type": "set_value", "value": 42.0}
            }),
            json!(1),
        );
        let resp = handle_stream(&h, req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn handle_stream_invalid_params_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.render.stream",
            json!({"session_id": "s1"}), // missing binding_id, operation
            json!(1),
        );
        let resp = handle_stream(&h, req);
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[test]
    fn handle_validate_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.validate",
            json!({
                "grammar": {
                    "data_source": "tbl",
                    "geometry": "Line",
                    "variables": [
                        {"name": "x", "field": "a", "role": "X"},
                        {"name": "y", "field": "b", "role": "Y"}
                    ],
                    "scales": [],
                    "coordinate": "Cartesian",
                    "facets": null,
                    "aesthetics": []
                },
                "data": [{"a": 1, "b": 2}]
            }),
            json!(1),
        );
        let resp = handle_validate(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert!(r["passed"].as_bool().is_some());
    }

    #[test]
    fn handle_export_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.export",
            json!({"session_id": "s1", "format": "svg"}),
            json!(1),
        );
        let resp = handle_export(&h, req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn handle_dismiss_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.dismiss",
            json!({"session_id": "s1"}),
            json!(1),
        );
        let resp = handle_dismiss(&h, req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn handle_interact_apply_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.interact.apply",
            json!({"intent": "select", "targets": ["node-1"]}),
            json!(1),
        );
        let resp = handle_interact_apply(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert!(r["accepted"].as_bool().is_some());
    }

    #[test]
    fn handle_interact_perspectives_returns_list() {
        let h = test_handlers();
        let resp = handle_interact_perspectives(&h, json!(1));
        assert!(resp.error.is_none());
        assert!(
            resp.result.expect("result")["perspectives"]
                .as_array()
                .is_some()
        );
    }
}
