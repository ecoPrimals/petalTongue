// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code uses unwrap/expect for brevity"
)]
//! Integration tests for the visualization pipeline and IPC handlers.

use petal_tongue_core::graph_engine::GraphEngine;
use petal_tongue_ipc::VisualizationState;
use petal_tongue_ipc::json_rpc::JsonRpcRequest;
use petal_tongue_ipc::method_gate::CallerContext;
use petal_tongue_ipc::unix_socket_connection;
use petal_tongue_ipc::unix_socket_rpc_handlers::RpcHandlers;
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};
use petal_tongue_scene::primitive::{Color, Primitive};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
use serde_json::json;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

fn test_handlers() -> RpcHandlers {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
    RpcHandlers::new(graph, "test-pipeline".to_string(), viz_state)
}

const fn test_ctx() -> CallerContext {
    CallerContext::unix()
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
    let resp = h.handle_request(req, &test_ctx()).await;
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
    let resp_a = h.handle_request(req_a, &test_ctx()).await;
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
    let resp_b = h.handle_request(req_b, &test_ctx()).await;
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
    let resp = h.handle_request(req, &test_ctx()).await;
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
    h.handle_request(req1, &test_ctx()).await;

    let req2 = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "list-session-2",
            "title": "Second Session",
            "bindings": []
        }),
        json!(2),
    );
    h.handle_request(req2, &test_ctx()).await;

    let req_list = JsonRpcRequest::new("visualization.session.list", json!({}), json!(3));
    let resp = h.handle_request(req_list, &test_ctx()).await;
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
    let resp = h.handle_request(req, &test_ctx()).await;
    assert!(resp.error.is_none());
    let r = resp.result.expect("result");
    assert!(r["subscription_id"].as_str().is_some());
    assert!(!r["subscription_id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_sensor_stream_poll_empty() {
    let h = test_handlers();
    let sub_req = JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
    let sub_resp = h.handle_request(sub_req, &test_ctx()).await;
    let subscription_id = sub_resp.result.unwrap()["subscription_id"]
        .as_str()
        .unwrap()
        .to_string();

    let poll_req = JsonRpcRequest::new(
        "interaction.sensor_stream.poll",
        json!({ "subscription_id": subscription_id }),
        json!(2),
    );
    let poll_resp = h.handle_request(poll_req, &test_ctx()).await;
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
    let sub_resp = h.handle_request(sub_req, &test_ctx()).await;
    let subscription_id = sub_resp.result.unwrap()["subscription_id"]
        .as_str()
        .unwrap()
        .to_string();

    let unsub_req = JsonRpcRequest::new(
        "interaction.sensor_stream.unsubscribe",
        json!({ "subscription_id": subscription_id }),
        json!(2),
    );
    let unsub_resp = h.handle_request(unsub_req, &test_ctx()).await;
    assert!(unsub_resp.error.is_none());
    let r = unsub_resp.result.expect("result");
    assert_eq!(r["unsubscribed"], true);
}

/// Test Unix socket connection handling: spawn listener, connect client, send JSON-RPC, verify response
#[tokio::test]
async fn test_unix_socket_connection_handle_request() {
    let handlers = test_handlers();
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("test.sock");

    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    let handlers = Arc::new(handlers);

    let server_handlers = Arc::clone(&handlers);
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let ctx = CallerContext::unix();
        unix_socket_connection::handle_connection(&server_handlers, stream, &ctx)
            .await
            .expect("handle_connection");
    });

    let mut stream = UnixStream::connect(&socket_path).await.expect("connect");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "health.get",
        "params": {},
        "id": 1
    });
    let line = serde_json::to_string(&request).expect("serialize") + "\n";
    stream.write_all(line.as_bytes()).await.expect("write");

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await.expect("read");
    let response: serde_json::Value = serde_json::from_str(response_line.trim()).expect("parse");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], "healthy");

    // Send invalid JSON to exercise parse error path
    stream = reader.into_inner();
    stream.write_all(b"{invalid json}\n").await.expect("write");
    response_line.clear();
    reader = BufReader::new(stream);
    reader.read_line(&mut response_line).await.expect("read");
    let err_response: serde_json::Value =
        serde_json::from_str(response_line.trim()).expect("parse");
    assert!(err_response["error"].is_object());
    assert_eq!(err_response["error"]["code"], -32700);
}

/// Wave 86 P2: health.liveness round-trip over UDS — must return {"status":"alive"}
/// without BTSP auth, matching ecosystem standard for 13/13 health parity.
#[tokio::test]
async fn test_uds_health_liveness_returns_alive() {
    let handlers = test_handlers();
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("health-liveness.sock");

    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    let handlers = Arc::new(handlers);

    let server_handlers = Arc::clone(&handlers);
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let ctx = CallerContext::unix();
        unix_socket_connection::handle_connection(&server_handlers, stream, &ctx)
            .await
            .expect("handle_connection");
    });

    let mut stream = UnixStream::connect(&socket_path).await.expect("connect");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {},
        "id": 1
    });
    let line = serde_json::to_string(&request).expect("serialize") + "\n";
    stream.write_all(line.as_bytes()).await.expect("write");

    let reader = BufReader::new(stream);
    let mut response_line = String::new();
    let mut reader = reader;
    reader.read_line(&mut response_line).await.expect("read");
    let response: serde_json::Value =
        serde_json::from_str(response_line.trim()).expect("parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["error"].is_null(),
        "health.liveness should not return an error: {response}"
    );
    assert_eq!(
        response["result"]["status"], "alive",
        "ecosystem standard: health.liveness must return {{\"status\":\"alive\"}}"
    );
}

/// Verify multiple health methods work on the same UDS connection (persistent
/// connection with sequential requests — ecosystem health sweep pattern).
#[tokio::test]
async fn test_uds_health_multi_method_sequence() {
    let handlers = test_handlers();
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("health-multi.sock");

    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    let handlers = Arc::new(handlers);

    let server_handlers = Arc::clone(&handlers);
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let ctx = CallerContext::unix();
        unix_socket_connection::handle_connection(&server_handlers, stream, &ctx)
            .await
            .expect("handle_connection");
    });

    let stream = UnixStream::connect(&socket_path).await.expect("connect");
    let (read_half, mut write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);

    let methods = [
        ("health.liveness", "alive"),
        ("health.check", "healthy"),
        ("health.readiness", "ready"),
    ];

    for (i, (method, expected_status)) in methods.iter().enumerate() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {},
            "id": i + 1
        });
        let line = serde_json::to_string(&request).expect("serialize") + "\n";
        write_half.write_all(line.as_bytes()).await.expect("write");

        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .expect("read response");
        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).expect("parse");
        assert_eq!(
            response["result"]["status"], *expected_status,
            "{method} should return status={expected_status}"
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Barrick Lab baselines — full pipeline test
// ──────────────────────────────────────────────────────────────────────────

fn barrick_baselines_bindings() -> Vec<serde_json::Value> {
    vec![
        json!({
            "channel_type": "genome_track",
            "id": "bl_breseq_genome", "label": "breseq Genome Track",
            "sequence_length": 4_629_812.0,
            "tracks": ["SNP", "IS Element"],
            "segments": [
                {"track": "SNP", "start": 70867.0, "end": 70868.0, "strand": ".", "label": "SNP"},
                {"track": "IS Element", "start": 776_697.0, "end": 778_028.0, "strand": "+", "label": "IS1"}
            ],
            "unit": "bp"
        }),
        json!({
            "channel_type": "bar",
            "id": "bl_breseq_evidence", "label": "Evidence Types",
            "categories": ["RA", "MC", "JC"], "values": [42.0, 12.0, 23.0], "unit": "count"
        }),
        json!({
            "channel_type": "gauge",
            "id": "bl_breseq_mutations", "label": "Total Mutations",
            "value": 94.0, "min": 0.0, "max": 200.0, "unit": "mutations",
            "normal_range": [0.0, 50.0], "warning_range": [50.0, 150.0]
        }),
        json!({
            "channel_type": "circular_map",
            "id": "bl_plannotate_map", "label": "pUC19",
            "sequence_length": 2686.0,
            "rings": ["features"],
            "arcs": [
                {"start_angle": 0.0, "end_angle": 90.0, "ring": 0, "label": "ori"},
                {"start_angle": 120.0, "end_angle": 200.0, "ring": 0, "label": "AmpR"}
            ],
            "unit": "bp"
        }),
        json!({
            "channel_type": "bar",
            "id": "bl_plannotate_features", "label": "Feature Lengths",
            "categories": ["ori", "AmpR", "lacZ"], "values": [600.0, 860.0, 510.0], "unit": "bp"
        }),
        json!({
            "channel_type": "scatter",
            "id": "bl_ostir_tir", "label": "OSTIR TIR",
            "x": [42.0, 156.0, 891.0], "y": [1200.0, 45000.0, 8900.0],
            "point_labels": ["RBS1", "RBS2", "RBS3"],
            "x_label": "Position", "y_label": "TIR", "unit": "au"
        }),
        json!({
            "channel_type": "distribution",
            "id": "bl_ostir_rate_dist", "label": "TIR Distribution",
            "values": [42.8, 127.5, 85.2], "mean": 85.15, "std": 59.9,
            "comparison_value": 100.0, "unit": "au"
        }),
        json!({
            "channel_type": "genome_track",
            "id": "bl_cryptkeeper_track", "label": "CryptKeeper",
            "sequence_length": 4_629_812.0,
            "tracks": ["ORFs", "Cryptic Promoters"],
            "segments": [
                {"track": "ORFs", "start": 100_000.0, "end": 102_000.0, "strand": "+", "label": "lacZ"},
                {"track": "Cryptic Promoters", "start": 101_800.0, "end": 102_200.0, "strand": "+", "label": "P_crypto"}
            ],
            "unit": "bp"
        }),
        json!({
            "channel_type": "genome_track",
            "id": "bl_efm_track", "label": "EFM Features",
            "sequence_length": 4_629_812.0,
            "tracks": ["IS Target", "Repeat Indel"],
            "segments": [
                {"track": "IS Target", "start": 776_697.0, "end": 778_028.0, "strand": "+", "label": "IS1"},
                {"track": "Repeat Indel", "start": 1_200_000.0, "end": 1_200_500.0, "strand": ".", "label": "repeat1"}
            ],
            "unit": "bp"
        }),
        json!({
            "channel_type": "scatter",
            "id": "bl_md_divergence", "label": "Marker Divergence",
            "x": [0.0, 100.0, 500.0, 1000.0], "y": [0.0, 0.15, 0.52, 0.78],
            "point_labels": ["t0", "t100", "t500", "t1000"],
            "x_label": "Generations", "y_label": "Divergence", "unit": "rel"
        }),
        json!({
            "channel_type": "heatmap",
            "id": "bl_rna_mi_cov", "label": "RNA MI Covariance",
            "x_labels": ["p1", "p2", "p3"], "y_labels": ["p1", "p2", "p3"],
            "values": [1.0, 0.8, 0.2, 0.8, 1.0, 0.3, 0.2, 0.3, 1.0], "unit": "bits"
        }),
        json!({
            "channel_type": "spectrum",
            "id": "bl_rna_mi_entropy", "label": "Positional Entropy",
            "frequencies": [1.0, 2.0, 3.0, 4.0], "amplitudes": [0.3, 0.7, 0.5, 0.9], "unit": "bits"
        }),
        json!({
            "channel_type": "timeseries",
            "id": "bl_growth_curve", "label": "Growth Curve",
            "x_label": "Time", "y_label": "OD600", "unit": "OD",
            "x_values": [0.0, 1.0, 2.0, 3.0], "y_values": [0.05, 0.12, 0.35, 0.78]
        }),
    ]
}

#[tokio::test]
async fn test_barrick_baselines_full_pipeline() {
    let h = test_handlers();
    let bindings = barrick_baselines_bindings();
    let binding_count = bindings.len();

    let req = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "barrick-baselines",
            "title": "Barrick Lab Baselines",
            "domain": "genomics",
            "bindings": bindings
        }),
        json!(1),
    );
    let resp = h.handle_request(req, &test_ctx()).await;
    assert!(resp.error.is_none(), "expected success: {:?}", resp.error);
    let r = resp.result.expect("result");
    assert_eq!(r["status"], "rendering");
    assert_eq!(
        r["bindings_accepted"].as_u64().unwrap_or(0),
        binding_count as u64,
        "all bindings should be accepted"
    );

    let viz_state = h.viz_state.read().unwrap();

    // Verify grammar_scenes has entries for each binding
    let scene_keys: Vec<&String> = viz_state
        .grammar_scenes
        .keys()
        .filter(|k| k.starts_with("barrick-baselines:"))
        .collect();
    assert_eq!(
        scene_keys.len(),
        binding_count,
        "expected {binding_count} grammar_scenes, got {}: {scene_keys:?}",
        scene_keys.len()
    );

    // Verify each scene has at least some primitives and exports to valid SVG
    let svg_compiler = SvgCompiler::new();
    for key in &scene_keys {
        let compiled = viz_state
            .grammar_scenes
            .get(*key)
            .unwrap_or_else(|| panic!("missing scene for {key}"));
        assert!(
            compiled.scene.total_primitives() > 0,
            "{key}: scene should have >0 primitives, got {}",
            compiled.scene.total_primitives()
        );

        let output = svg_compiler.compile(&compiled.scene);
        let svg = match output {
            ModalityOutput::Svg(bytes) => String::from_utf8(bytes.to_vec()).expect("valid UTF-8"),
            other => panic!("{key}: expected SVG, got {other:?}"),
        };
        assert!(svg.starts_with("<svg"), "{key}: SVG should start with <svg");
        assert!(svg.ends_with("</svg>"), "{key}: SVG should end with </svg>");
        assert!(!svg.contains("NaN"), "{key}: SVG should not contain NaN");
        assert!(
            !svg.contains("Infinity"),
            "{key}: SVG should not contain Infinity"
        );
    }
}

#[tokio::test]
async fn test_barrick_baselines_stream_recompile() {
    let h = test_handlers();

    let req = JsonRpcRequest::new(
        "visualization.render",
        json!({
            "session_id": "stream-test",
            "title": "Stream Test",
            "bindings": [{
                "channel_type": "timeseries",
                "id": "ts-stream",
                "label": "Live TS",
                "x_label": "t", "y_label": "v", "unit": "u",
                "x_values": [0.0, 1.0], "y_values": [10.0, 20.0]
            }]
        }),
        json!(1),
    );
    h.handle_request(req, &test_ctx()).await;

    // Verify initial scene exists
    {
        let viz_state = h.viz_state.read().unwrap();
        assert!(
            viz_state
                .grammar_scenes
                .contains_key("stream-test:ts-stream"),
            "initial scene should exist"
        );
        let initial_prims = viz_state.grammar_scenes["stream-test:ts-stream"]
            .scene
            .total_primitives();
        assert!(initial_prims > 0);
    }

    // Send a stream update (append_point)
    let stream_req = JsonRpcRequest::new(
        "visualization.render.stream",
        json!({
            "session_id": "stream-test",
            "binding_id": "ts-stream",
            "operation": {
                "type": "append",
                "x_values": [2.0],
                "y_values": [30.0]
            }
        }),
        json!(2),
    );
    let stream_resp = h.handle_request(stream_req, &test_ctx()).await;
    assert!(
        stream_resp.error.is_none(),
        "stream update should succeed: {:?}",
        stream_resp.error
    );
    let r = stream_resp.result.expect("result");
    assert_eq!(r["accepted"], true);

    // Verify scene was recompiled (should now have updated data)
    let viz_state = h.viz_state.read().unwrap();
    assert!(
        viz_state
            .grammar_scenes
            .contains_key("stream-test:ts-stream"),
        "scene should still exist after stream update"
    );
    let scene = &viz_state.grammar_scenes["stream-test:ts-stream"];
    assert!(
        scene.scene.total_primitives() > 0,
        "recompiled scene should have primitives"
    );
}
