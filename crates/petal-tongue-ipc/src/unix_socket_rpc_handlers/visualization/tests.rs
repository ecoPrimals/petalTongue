// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization handler unit tests.

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
        json!({"session_id": 123, "title": "x"}),
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
        json!({"session_id": "s1"}),
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

#[test]
fn handle_render_scene_missing_scene_returns_error() {
    let h = test_handlers();
    let req = JsonRpcRequest::new(
        "visualization.render.scene",
        json!({"session_id": "s1"}),
        json!(1),
    );
    let resp = handle_render_scene(&h, req);
    assert!(resp.error.is_some());
    assert_eq!(
        resp.error.as_ref().expect("err").code,
        error_codes::INVALID_PARAMS
    );
}

#[test]
fn handle_render_scene_invalid_scene_returns_error() {
    let h = test_handlers();
    let req = JsonRpcRequest::new(
        "visualization.render.scene",
        json!({"session_id": "s1", "scene": "not a valid scene"}),
        json!(1),
    );
    let resp = handle_render_scene(&h, req);
    assert!(resp.error.is_some());
}

#[test]
fn handle_render_scene_valid_succeeds() {
    let h = test_handlers();
    let scene = petal_tongue_scene::scene_graph::SceneGraph::new();
    let scene_value = serde_json::to_value(&scene).expect("serialize");
    let req = JsonRpcRequest::new(
        "visualization.render.scene",
        json!({"session_id": "scene-session", "scene": scene_value}),
        json!(1),
    );
    let resp = handle_render_scene(&h, req);
    assert!(resp.error.is_none());
    let r = resp.result.expect("result");
    assert_eq!(r["session_id"], "scene-session");
    assert_eq!(r["status"], "scene_stored");
}

#[test]
fn handle_grammar_render_invalid_params_returns_error() {
    let h = test_handlers();
    let req = JsonRpcRequest::new(
        "visualization.render.grammar",
        json!({"invalid": "params"}),
        json!(1),
    );
    let resp = handle_grammar_render(&h, req);
    assert!(resp.error.is_some());
}

#[test]
fn handle_grammar_render_valid_params_succeeds() {
    let h = test_handlers();
    let req = JsonRpcRequest::new(
        "visualization.render.grammar",
        json!({
            "session_id": "gram-s1",
            "grammar": {
                "data_source": "tbl",
                "geometry": "Point",
                "variables": [
                    {"name": "x", "field": "a", "role": "X"},
                    {"name": "y", "field": "b", "role": "Y"}
                ],
                "scales": [],
                "coordinate": "Cartesian",
                "facets": null,
                "aesthetics": []
            },
            "data": [{"a": 1.0, "b": 2.0}, {"a": 3.0, "b": 4.0}],
            "modality": "svg"
        }),
        json!(1),
    );
    let resp = handle_grammar_render(&h, req);
    assert!(resp.error.is_none());
    let r = resp.result.expect("result");
    assert_eq!(r["session_id"], "gram-s1");
    assert_eq!(r["modality"], "svg");
}

#[test]
fn handle_dashboard_render_invalid_params_returns_error() {
    let h = test_handlers();
    let req = JsonRpcRequest::new("visualization.render.dashboard", json!({}), json!(1));
    let resp = handle_dashboard_render(&h, req);
    assert!(resp.error.is_some());
}

#[test]
fn handle_dashboard_render_valid_params_succeeds() {
    let h = test_handlers();
    let req = JsonRpcRequest::new(
        "visualization.render.dashboard",
        json!({
            "session_id": "dash1",
            "title": "Test Dashboard",
            "bindings": []
        }),
        json!(1),
    );
    let resp = handle_dashboard_render(&h, req);
    assert!(resp.error.is_none());
    let r = resp.result.expect("result");
    assert_eq!(r["session_id"], "dash1");
    assert_eq!(r["panel_count"], 0);
}

#[test]
fn handle_export_invalid_params_returns_error() {
    let h = test_handlers();
    let req = JsonRpcRequest::new("visualization.export", json!({"session_id": 123}), json!(1));
    let resp = handle_export(&h, req);
    assert!(resp.error.is_some());
}

#[test]
fn handle_render_scene_default_session_id() {
    let h = test_handlers();
    let scene = petal_tongue_scene::scene_graph::SceneGraph::new();
    let scene_value = serde_json::to_value(&scene).expect("serialize");
    let req = JsonRpcRequest::new(
        "visualization.render.scene",
        json!({"scene": scene_value}),
        json!(1),
    );
    let resp = handle_render_scene(&h, req);
    assert!(resp.error.is_none());
    assert_eq!(resp.result.expect("result")["session_id"], "scene-session");
}
