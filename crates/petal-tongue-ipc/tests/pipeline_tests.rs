// SPDX-License-Identifier: AGPL-3.0-only
//! Integration tests for the visualization pipeline and IPC handlers.

use petal_tongue_core::graph_engine::GraphEngine;
use petal_tongue_ipc::VisualizationState;
use petal_tongue_ipc::json_rpc::JsonRpcRequest;
use petal_tongue_ipc::unix_socket_rpc_handlers::RpcHandlers;
use petal_tongue_scene::primitive::{Color, Primitive};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
use serde_json::json;
use std::sync::{Arc, RwLock};

fn test_handlers() -> RpcHandlers {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
    RpcHandlers::new(graph, "test-pipeline".to_string(), viz_state)
}

#[tokio::test]
async fn test_e2e_visualization_pipeline() {
    let h = test_handlers();
    let req = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "e2e-session",
            "title": "E2E Test",
            "bindings": [
                {
                    "channel_type": "timeseries",
                    "id": "ts1",
                    "label": "Test Series",
                    "x_label": "Time",
                    "y_label": "Value",
                    "unit": "units",
                    "x_values": [1.0, 2.0, 3.0],
                    "y_values": [10.0, 20.0, 30.0]
                }
            ]
        }),
        json!(1),
    );
    let resp = h.handle_request(req).await;
    assert!(resp.error.is_none(), "expected success: {:?}", resp.error);
    let r = resp.result.expect("result");
    assert_eq!(r["status"], "rendering");
    assert!(r["bindings_accepted"].as_u64().unwrap_or(0) > 0);

    let viz_state = h.viz_state.read().unwrap();
    let has_scene = viz_state
        .grammar_scenes
        .keys()
        .any(|k| k.starts_with("e2e-session:"));
    assert!(has_scene, "expected grammar_scene for session e2e-session");
}

#[tokio::test]
async fn test_concurrent_sessions() {
    let h = test_handlers();

    let req_a = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "session-a",
            "title": "Session A",
            "bindings": [
                {
                    "channel_type": "timeseries",
                    "id": "ts-a",
                    "label": "Time Series A",
                    "x_label": "t",
                    "y_label": "v",
                    "unit": "",
                    "x_values": [0.0, 1.0],
                    "y_values": [1.0, 2.0]
                }
            ]
        }),
        json!(1),
    );
    let resp_a = h.handle_request(req_a).await;
    assert!(resp_a.error.is_none());
    assert_eq!(resp_a.result.as_ref().unwrap()["session_id"], "session-a");
    assert_eq!(resp_a.result.as_ref().unwrap()["bindings_accepted"], 1);

    let req_b = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "session-b",
            "title": "Session B",
            "bindings": [
                {
                    "channel_type": "bar",
                    "id": "bar-b",
                    "label": "Bar B",
                    "categories": ["A", "B", "C"],
                    "values": [1.0, 2.0, 3.0],
                    "unit": ""
                }
            ]
        }),
        json!(2),
    );
    let resp_b = h.handle_request(req_b).await;
    assert!(resp_b.error.is_none());
    assert_eq!(resp_b.result.as_ref().unwrap()["session_id"], "session-b");
    assert_eq!(resp_b.result.as_ref().unwrap()["bindings_accepted"], 1);

    let viz_state = h.viz_state.read().unwrap();
    assert!(viz_state.sessions.contains_key("session-a"));
    assert!(viz_state.sessions.contains_key("session-b"));

    let session_a = viz_state.sessions.get("session-a").unwrap();
    let session_b = viz_state.sessions.get("session-b").unwrap();

    assert_eq!(session_a.title, "Session A");
    assert_eq!(session_b.title, "Session B");
    assert_eq!(session_a.bindings.len(), 1);
    assert_eq!(session_b.bindings.len(), 1);

    let has_ts = matches!(
        &session_a.bindings[0],
        petal_tongue_core::DataBinding::TimeSeries { .. }
    );
    let has_bar = matches!(
        &session_b.bindings[0],
        petal_tongue_core::DataBinding::Bar { .. }
    );
    assert!(has_ts, "session-a should have TimeSeries binding");
    assert!(has_bar, "session-b should have Bar binding");
}

#[tokio::test]
async fn test_render_scene_direct() {
    let h = test_handlers();

    let mut scene = SceneGraph::new();
    scene.add_to_root(
        SceneNode::new("point-node").with_primitive(Primitive::Point {
            x: 10.0,
            y: 20.0,
            radius: 5.0,
            fill: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            stroke: None,
            data_id: Some("test-data".to_string()),
        }),
    );

    let scene_json = serde_json::to_value(&scene).expect("serialize scene");
    let req = JsonRpcRequest::new(
        "visualization.render.scene",
        json!({
            "session_id": "scene-session",
            "scene": scene_json
        }),
        json!(1),
    );
    let resp = h.handle_request(req).await;
    assert!(resp.error.is_none(), "expected success: {:?}", resp.error);
    let r = resp.result.expect("result");
    assert_eq!(r["status"], "scene_stored");
    assert!(r["nodes_accepted"].as_u64().unwrap_or(0) > 0);

    let viz_state = h.viz_state.read().unwrap();
    assert!(
        viz_state.grammar_scenes.contains_key("scene-session"),
        "expected grammar_scene for scene-session"
    );
}

#[tokio::test]
async fn test_session_list() {
    let h = test_handlers();

    let req1 = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "list-session-1",
            "title": "First Session",
            "bindings": []
        }),
        json!(1),
    );
    h.handle_request(req1).await;

    let req2 = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "list-session-2",
            "title": "Second Session",
            "bindings": []
        }),
        json!(2),
    );
    h.handle_request(req2).await;

    let req_list = JsonRpcRequest::new("visualization.session.list", json!({}), json!(3));
    let resp = h.handle_request(req_list).await;
    assert!(resp.error.is_none());
    let r = resp.result.expect("result");
    let sessions = r["sessions"].as_array().expect("sessions array");

    let titles: Vec<&str> = sessions
        .iter()
        .map(|s| s["title"].as_str().unwrap_or(""))
        .collect();
    assert!(
        titles.contains(&"First Session"),
        "expected First Session in list: {titles:?}"
    );
    assert!(
        titles.contains(&"Second Session"),
        "expected Second Session in list: {titles:?}"
    );
}

#[tokio::test]
async fn test_sensor_stream_subscribe() {
    let h = test_handlers();
    let req = JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
    let resp = h.handle_request(req).await;
    assert!(resp.error.is_none());
    let r = resp.result.expect("result");
    assert!(r["subscription_id"].as_str().is_some());
    assert!(!r["subscription_id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_sensor_stream_poll_empty() {
    let h = test_handlers();
    let sub_req = JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
    let sub_resp = h.handle_request(sub_req).await;
    let subscription_id = sub_resp.result.unwrap()["subscription_id"]
        .as_str()
        .unwrap()
        .to_string();

    let poll_req = JsonRpcRequest::new(
        "interaction.sensor_stream.poll",
        json!({ "subscription_id": subscription_id }),
        json!(2),
    );
    let poll_resp = h.handle_request(poll_req).await;
    assert!(poll_resp.error.is_none());
    let r = poll_resp.result.expect("result");
    let events = r["events"].as_array().expect("events array");
    assert!(
        events.is_empty(),
        "expected empty batch immediately after subscribe"
    );
}

#[tokio::test]
async fn test_sensor_stream_unsubscribe() {
    let h = test_handlers();
    let sub_req = JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
    let sub_resp = h.handle_request(sub_req).await;
    let subscription_id = sub_resp.result.unwrap()["subscription_id"]
        .as_str()
        .unwrap()
        .to_string();

    let unsub_req = JsonRpcRequest::new(
        "interaction.sensor_stream.unsubscribe",
        json!({ "subscription_id": subscription_id }),
        json!(2),
    );
    let unsub_resp = h.handle_request(unsub_req).await;
    assert!(unsub_resp.error.is_none());
    let r = unsub_resp.result.expect("result");
    assert_eq!(r["unsubscribed"], true);
}
