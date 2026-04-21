// SPDX-License-Identifier: AGPL-3.0-or-later

use super::UnixSocketServer;
use crate::json_rpc::JsonRpcRequest;
use crate::json_rpc::error_codes;
use crate::visualization_handler::VisualizationState;
use petal_tongue_core::graph_engine::GraphEngine;
use petal_tongue_core::test_fixtures::env_test_helpers;
use serde_json::json;
use std::sync::{Arc, RwLock};

#[test]
fn test_unix_socket_server_creation() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("test-family")),
            ("XDG_RUNTIME_DIR", Some("/tmp")),
            ("PETALTONGUE_NODE_ID", Some("default")),
        ],
        || {
            let server = UnixSocketServer::new(graph).unwrap();
            assert!(
                server.push_delivery_wired_for_tests(),
                "PT-06: push delivery should be wired for callback notifications"
            );
            assert_eq!(server.family_id, "test-family");
            let socket_str = server.socket_path.to_str().unwrap();
            assert!(
                socket_str.ends_with("biomeos/petaltongue-test-family.sock"),
                "Socket path should end with family-scoped petaltongue-test-family.sock, got: {socket_str}"
            );
            assert!(
                socket_str.contains("/tmp") || socket_str.contains("/run/user"),
                "Socket path should use XDG runtime directory, got: {socket_str}"
            );
        },
    );
}

#[test]
fn test_unix_socket_server_drop_removes_socket() {
    let tmp = tempfile::tempdir().expect("tempdir");
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("drop-test")),
            ("XDG_RUNTIME_DIR", tmp.path().to_str()),
            ("PETALTONGUE_NODE_ID", Some("node")),
        ],
        || {
            let graph = Arc::new(RwLock::new(GraphEngine::new()));
            let server = UnixSocketServer::new(graph).unwrap();
            let socket_path = server.socket_path.clone();
            std::fs::write(&socket_path, "stale").expect("create file");
            assert!(socket_path.exists());
            drop(server);
            assert!(!socket_path.exists(), "Drop should remove socket file");
        },
    );
}

#[test]
fn test_unix_socket_server_drop_removes_socket_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("dir-drop")),
            ("XDG_RUNTIME_DIR", tmp.path().to_str()),
            ("PETALTONGUE_NODE_ID", Some("x")),
        ],
        || {
            let graph = Arc::new(RwLock::new(GraphEngine::new()));
            let server = UnixSocketServer::new(graph).unwrap();
            let socket_path = server.socket_path.clone();
            std::fs::create_dir(&socket_path).expect("create dir");
            assert!(socket_path.is_dir());
            drop(server);
            assert!(!socket_path.exists(), "Drop should remove socket dir");
        },
    );
}

#[test]
fn test_get_capabilities_response() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let response = server.get_capabilities(json!(1));
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["capabilities"].is_array());
        assert_eq!(result["family_id"], server.family_id);
    });
}

#[test]
fn test_get_health_response() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let response = server.get_health(json!(1));
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "healthy");
        assert_eq!(result["family_id"], server.family_id);
    });
}

#[test]
fn test_biomeos_health_check() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let request = JsonRpcRequest::new("health.check", json!({}), json!(1));
        let response = server.handle_health_check(request);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "healthy");
        assert_eq!(result["version"], env!("CARGO_PKG_VERSION"));
        assert!(result["modalities_active"].is_array());
    });
}

#[test]
fn test_biomeos_announce_capabilities() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let request = JsonRpcRequest::new("capability.announce", json!({}), json!(1));
        let response = server.handle_announce_capabilities(request);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["capabilities"].is_array());
        let caps = result["capabilities"].as_array().unwrap();
        assert!(!caps.is_empty());
    });
}

#[test]
fn test_handle_ui_display_status_valid_params() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let request = JsonRpcRequest::new(
            "ui.display_status",
            json!({
                "primal_name": "beardog",
                "status": {
                    "health": "healthy",
                    "tunnels_active": 3
                }
            }),
            json!(42),
        );
        let response = server.handle_ui_display_status(request);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["updated"], true);
        assert_eq!(result["primal"], "beardog");
    });
}

#[test]
fn test_handle_ui_display_status_missing_params() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let request = JsonRpcRequest::new("ui.display_status", json!(null), json!(1));
        let response = server.handle_ui_display_status(request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, error_codes::INVALID_PARAMS);
    });
}

#[test]
fn test_get_topology_with_nodes() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_core::test_fixtures::primals;

    let mut graph = GraphEngine::new();
    graph.add_node(primals::test_primal("node1"));
    graph.add_node(primals::test_primal("node2"));
    graph.add_edge(petal_tongue_core::TopologyEdge {
        from: "node1".into(),
        to: "node2".into(),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);

    let graph = Arc::new(RwLock::new(graph));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let response = server.get_topology(json!(1));
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["nodes"].is_array());
        assert!(result["edges"].is_array());
        assert_eq!(result["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(result["edges"].as_array().unwrap().len(), 1);
    });
}

#[test]
fn test_render_graph_svg_format() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(server.render_graph(json!({"format": "svg"}), json!(1)));
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["format"], "svg");
        assert!(result["data"].as_str().unwrap().contains("svg"));
    });
}

#[test]
fn test_render_graph_unsupported_format() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(server.render_graph(json!({"format": "pdf"}), json!(1)));
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, error_codes::INVALID_PARAMS);
    });
}

#[test]
fn test_with_motor_sender() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let (tx, _rx) = std::sync::mpsc::channel();
        let server = UnixSocketServer::new(graph).unwrap().with_motor_sender(tx);
        let response = server.get_health(json!(1));
        assert!(response.result.is_some());
    });
}

#[test]
fn test_get_topology_empty_graph() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let response = server.get_topology(json!(1));
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["nodes"].as_array().unwrap().is_empty());
        assert!(result["edges"].as_array().unwrap().is_empty());
    });
}

#[test]
fn test_handle_ui_display_status_empty_status() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let request = JsonRpcRequest::new(
            "ui.display_status",
            json!({"primal_name": "test", "status": {}}),
            json!(1),
        );
        let response = server.handle_ui_display_status(request);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["primal"], "test");
    });
}

#[test]
fn test_visualization_state_handle() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let handle = server.visualization_state_handle();
        assert!(Arc::strong_count(&handle) >= 2);
    });
}

#[test]
fn test_sensor_stream_handle() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let handle = server.sensor_stream_handle();
        assert!(Arc::strong_count(&handle) >= 2);
    });
}

#[test]
fn test_interaction_subscribers_handle() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let handle = server.interaction_subscribers_handle();
        assert!(Arc::strong_count(&handle) >= 2);
    });
}

#[test]
fn test_with_rendering_awareness() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let awareness = Arc::new(RwLock::new(petal_tongue_core::RenderingAwareness::default()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph)
            .unwrap()
            .with_rendering_awareness(awareness);
        let response = server.get_health(json!(1));
        assert!(response.result.is_some());
    });
}

#[test]
fn test_with_visualization_state() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph)
            .unwrap()
            .with_visualization_state(viz_state);
        let response = server.get_capabilities(json!(1));
        assert!(response.result.is_some());
    });
}

#[test]
fn test_get_capabilities_returns_family_id() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("cap-test-family")),
            ("XDG_RUNTIME_DIR", Some("/tmp")),
        ],
        || {
            let server = UnixSocketServer::new(graph).unwrap();
            let response = server.get_capabilities(json!(99));
            assert!(response.result.is_some());
            assert_eq!(response.result.unwrap()["family_id"], "cap-test-family");
        },
    );
}

#[test]
fn test_get_health_returns_graph_stats() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let response = server.get_health(json!(1));
        let result = response.result.unwrap();
        assert_eq!(result["status"], "healthy");
        assert!(result["graph"].is_object());
        assert!(result["graph"]["nodes"].is_number());
        assert!(result["graph"]["edges"].is_number());
    });
}

#[test]
fn test_handle_announce_capabilities_returns_array() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        let server = UnixSocketServer::new(graph).unwrap();
        let request = JsonRpcRequest::new("capability.announce", json!({}), json!(1));
        let response = server.handle_announce_capabilities(request);
        let result = response.result.unwrap();
        let caps = result["capabilities"].as_array().unwrap();
        assert!(!caps.is_empty());
    });
}

#[tokio::test]
async fn test_default_rendering_awareness_initialized() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let server = env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        UnixSocketServer::new(graph).unwrap()
    });
    let request = JsonRpcRequest::new(
        "visualization.showing",
        json!({"data_id": "test-data"}),
        json!(1),
    );
    let response = server.handle_request(request).await;
    assert!(
        response.error.is_none(),
        "should not error when awareness is default-initialized"
    );
    let result = response.result.unwrap();
    assert_eq!(result["data_id"], "test-data");
}

#[tokio::test]
async fn test_introspect_works_with_default_awareness() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let server = env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp", || {
        UnixSocketServer::new(graph).unwrap()
    });
    let request = JsonRpcRequest::new("visualization.introspect", json!({}), json!(1));
    let response = server.handle_request(request).await;
    assert!(
        response.error.is_none(),
        "introspect should succeed with default awareness (PT-05)"
    );
}

#[test]
fn btsp_json_announcement_detected() {
    let announcement = br#"{"protocol":"btsp","version":"1.0"}"#;
    assert!(
        UnixSocketServer::is_btsp_json_announcement(announcement),
        "should detect BTSP JSON-line protocol announcement"
    );
}

#[test]
fn plain_jsonrpc_not_classified_as_btsp() {
    let jsonrpc = br#"{"jsonrpc":"2.0","method":"health.check","id":1}"#;
    assert!(
        !UnixSocketServer::is_btsp_json_announcement(jsonrpc),
        "plain JSON-RPC should not be classified as BTSP announcement"
    );
}

#[test]
fn non_json_not_classified_as_btsp_announcement() {
    assert!(
        !UnixSocketServer::is_btsp_json_announcement(b"\x00\x00\x00\x10"),
        "length-prefixed binary should not match JSON-line check"
    );
}

#[test]
fn empty_buffer_not_classified_as_btsp() {
    assert!(!UnixSocketServer::is_btsp_json_announcement(b""));
}

#[test]
fn partial_protocol_key_not_classified() {
    let partial = br#"{"proto":"btsp"}"#;
    assert!(
        !UnixSocketServer::is_btsp_json_announcement(partial),
        "partial key 'proto' should not match"
    );
}
