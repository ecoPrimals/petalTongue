// SPDX-License-Identifier: AGPL-3.0-or-later

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
    assert_eq!(
        r["primal"], "petaltongue",
        "DEPLOYMENT_VALIDATION_STANDARD: health.check MUST include primal"
    );
    assert!(
        r["version"].as_str().is_some(),
        "DEPLOYMENT_VALIDATION_STANDARD: health.check MUST include version"
    );
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
    assert_eq!(
        r["primal"], "petaltongue",
        "DEPLOYMENT_VALIDATION_STANDARD: health.readiness MUST include primal"
    );
    assert!(
        r["version"].as_str().is_some(),
        "DEPLOYMENT_VALIDATION_STANDARD: health.readiness MUST include version"
    );
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
fn capabilities_list_includes_scene_and_interaction() {
    let h = test_handlers();
    let resp = get_capabilities(&h, serde_json::json!(1));
    let r = resp.result.expect("success");
    let methods: Vec<&str> = r["methods"]
        .as_array()
        .expect("methods array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        methods.contains(&"visualization.render.scene"),
        "visualization.render.scene must be advertised"
    );
    assert!(
        methods.contains(&"interaction.poll"),
        "interaction.poll must be advertised"
    );
    assert!(
        methods.contains(&"interaction.subscribe"),
        "interaction.subscribe must be advertised"
    );
    assert!(
        methods.contains(&"interaction.unsubscribe"),
        "interaction.unsubscribe must be advertised"
    );

    let capabilities: Vec<&str> = r["capabilities"]
        .as_array()
        .expect("capabilities array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        capabilities.contains(&"visualization.render.scene"),
        "visualization.render.scene must be in capabilities"
    );
    assert!(
        capabilities.contains(&"interaction.subscribe"),
        "interaction.subscribe must be in capabilities"
    );
    assert!(
        capabilities.contains(&"interaction.poll"),
        "interaction.poll must be in capabilities"
    );
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
