// SPDX-License-Identifier: AGPL-3.0-only

use crate::json_rpc::JsonRpcRequest;
use crate::socket_path;
use crate::unix_socket_connection;
use crate::unix_socket_rpc_handlers::RpcHandlers;
use crate::visualization_handler::VisualizationState;
use anyhow::Result;
use petal_tongue_core::graph_engine::GraphEngine;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tracing::{debug, error, info};

/// Unix socket server for petalTongue JSON-RPC IPC
pub struct UnixSocketServer {
    socket_path: PathBuf,
    family_id: String,
    handlers: RpcHandlers,
    motor_tx: Option<std::sync::mpsc::Sender<petal_tongue_core::MotorCommand>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl UnixSocketServer {
    /// Create a new Unix socket server with graph and visualization state
    pub fn new(graph: Arc<std::sync::RwLock<GraphEngine>>) -> Result<Self> {
        let family_id = socket_path::get_family_id();
        let socket_path = socket_path::get_petaltongue_socket_path()?;
        let viz_state = Arc::new(std::sync::RwLock::new(VisualizationState::new()));

        Ok(Self {
            socket_path,
            family_id: family_id.clone(),
            handlers: RpcHandlers::new(graph, family_id, viz_state),
            motor_tx: None,
        })
    }

    /// Attach a motor command sender so IPC motor commands are forwarded
    /// to the UI's efferent channel.
    #[must_use]
    pub fn with_motor_sender(
        mut self,
        tx: std::sync::mpsc::Sender<petal_tongue_core::MotorCommand>,
    ) -> Self {
        self.motor_tx = Some(tx.clone());
        self.handlers.motor_tx = Some(tx);
        self
    }

    /// Inject a shared `VisualizationState` so the UI can poll IPC sessions.
    #[must_use]
    pub fn with_visualization_state(
        mut self,
        viz_state: Arc<std::sync::RwLock<VisualizationState>>,
    ) -> Self {
        self.handlers.viz_state = viz_state;
        self
    }

    /// Return a handle to the shared visualization state.
    #[must_use]
    pub fn visualization_state_handle(&self) -> Arc<std::sync::RwLock<VisualizationState>> {
        Arc::clone(&self.handlers.viz_state)
    }

    /// Return a handle to the sensor stream subscriber registry.
    #[must_use]
    pub fn sensor_stream_handle(
        &self,
    ) -> Arc<std::sync::RwLock<crate::visualization_handler::SensorStreamRegistry>> {
        Arc::clone(&self.handlers.sensor_stream_subscribers)
    }

    /// Return a handle to the interaction subscriber registry.
    #[must_use]
    pub fn interaction_subscribers_handle(
        &self,
    ) -> Arc<std::sync::RwLock<crate::visualization_handler::InteractionSubscriberRegistry>> {
        Arc::clone(&self.handlers.interaction_subscribers)
    }

    /// Attach rendering awareness so IPC can serve introspection queries.
    #[must_use]
    pub fn with_rendering_awareness(
        mut self,
        awareness: Arc<std::sync::RwLock<petal_tongue_core::RenderingAwareness>>,
    ) -> Self {
        self.handlers.rendering_awareness = Some(awareness);
        self
    }

    /// Start the server: bind socket and accept connections
    pub async fn start(self: Arc<Self>) -> Result<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
            debug!("Removed old socket: {}", self.socket_path.display());
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!(
            "🔌 Unix socket server listening: {}",
            self.socket_path.display()
        );
        info!("   Family ID: {}", self.family_id);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) =
                            unix_socket_connection::handle_connection(&server.handlers, stream)
                                .await
                        {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    fn get_capabilities(&self, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.get_capabilities(id)
    }

    fn get_health(&self, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.get_health(id)
    }

    fn get_topology(&self, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.get_topology(id)
    }

    fn handle_health_check(&self, request: JsonRpcRequest) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_health_check(request)
    }

    fn handle_announce_capabilities(
        &self,
        request: JsonRpcRequest,
    ) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_announce_capabilities(request)
    }

    fn handle_ui_display_status(
        &self,
        request: JsonRpcRequest,
    ) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_ui_display_status(request)
    }

    async fn render_graph(&self, params: Value, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.render_graph(params, id).await
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                error!("Failed to remove socket: {}", e);
            } else {
                info!("Cleaned up socket: {}", self.socket_path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_rpc::JsonRpcRequest;
    use crate::json_rpc::error_codes;
    use petal_tongue_core::test_fixtures::env_test_helpers;
    use serde_json::json;
    use std::sync::RwLock;

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
                assert_eq!(server.family_id, "test-family");
                let socket_str = server.socket_path.to_str().unwrap();
                assert!(
                    socket_str.ends_with("petaltongue-test-family-default.sock"),
                    "Socket path should end with family and node ID, got: {socket_str}"
                );
                assert!(
                    socket_str.contains("/tmp") || socket_str.contains("/run/user"),
                    "Socket path should use XDG runtime directory, got: {socket_str}"
                );
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
}
