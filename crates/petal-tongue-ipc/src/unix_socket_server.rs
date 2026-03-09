// SPDX-License-Identifier: AGPL-3.0-only
//! Unix socket JSON-RPC server for petalTongue
//!
//! Provides a port-free, file-system-based IPC mechanism for local inter-primal
//! communication following the ecoPrimals standard.
//!
//! # biomeOS Integration
//!
//! Socket path follows biomeOS convention: `/run/user/<uid>/petaltongue-<family>.sock`
//! This enables zero-config discovery and capability-based inter-primal communication.

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::socket_path;
use anyhow::Result;
use petal_tongue_core::graph_engine::GraphEngine;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

/// Unix socket JSON-RPC server for petalTongue
pub struct UnixSocketServer {
    /// Path to the Unix socket
    socket_path: PathBuf,

    /// Family ID for this petalTongue instance
    family_id: String,

    /// Shared graph engine
    graph: Arc<RwLock<GraphEngine>>,

    /// Server start time (for uptime calculation)
    start_time: SystemTime,
}

impl UnixSocketServer {
    /// Create a new Unix socket server
    ///
    /// # biomeOS Convention
    ///
    /// Socket path will be: `/run/user/<uid>/petaltongue-<family>.sock`
    ///
    /// Uses `FAMILY_ID` environment variable (default: "nat0")
    ///
    /// # TRUE PRIMAL Principles
    ///
    /// - **Zero Hardcoding**: Path determined at runtime from environment
    /// - **Capability-Based**: Uses standard Unix runtime directory
    /// - **Self-Knowledge**: Only knows own identity (petaltongue)
    pub fn new(graph: Arc<RwLock<GraphEngine>>) -> Result<Self> {
        let family_id = socket_path::get_family_id();
        let socket_path = socket_path::get_petaltongue_socket_path()?;

        Ok(Self {
            socket_path,
            family_id,
            graph,
            start_time: SystemTime::now(),
        })
    }

    /// Get uptime in seconds
    fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().unwrap_or_default().as_secs()
    }

    /// Start the Unix socket server
    ///
    /// This will bind to the Unix socket and accept connections in a loop.
    /// Each connection is handled in a separate task.
    ///
    /// # biomeOS Integration
    ///
    /// Socket will be at: `/run/user/<uid>/petaltongue-<family>.sock`
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Clean up old socket if exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
            debug!("Removed old socket: {}", self.socket_path.display());
        }

        // Bind to Unix socket
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
                        if let Err(e) = server.handle_connection(stream).await {
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

    /// Handle a single connection
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                // Connection closed
                break;
            }

            // Parse JSON-RPC request
            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => {
                    debug!(
                        "Received request: method={}, id={}",
                        request.method, request.id
                    );
                    self.handle_request(request).await
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {}", e);
                    JsonRpcResponse::error(
                        json!(null),
                        error_codes::PARSE_ERROR,
                        format!("Parse error: {e}"),
                    )
                }
            };

            // Send response
            let response_json = serde_json::to_string(&response)?;
            writer.write_all(response_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    ///
    /// # biomeOS Integration
    ///
    /// Supports both legacy methods and new biomeOS methods:
    /// - Legacy: get_capabilities, get_health, get_topology, render_graph
    /// - biomeOS: health_check, announce_capabilities, ui.render, ui.display_status
    async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            // biomeOS standard methods
            "health_check" => self.handle_health_check(&req),
            "announce_capabilities" => self.handle_announce_capabilities(&req),
            "ui.render" => self.handle_ui_render(&req).await,
            "ui.display_status" => self.handle_ui_display_status(&req),

            // Legacy methods (for backward compatibility)
            "get_capabilities" => self.get_capabilities(req.id),
            "render_graph" => self.render_graph(req.params, req.id).await,
            "get_health" => self.get_health(req.id),
            "get_topology" => self.get_topology(req.id),

            _ => {
                warn!("Unknown method: {}", req.method);
                JsonRpcResponse::error(
                    req.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method not found: {}", req.method),
                )
            }
        }
    }

    // ===== biomeOS Integration Methods (Semantic Naming) =====

    /// health.check - Returns health status following biomeOS convention
    ///
    /// # Response Format
    ///
    /// ```json
    /// {
    ///   "status": "healthy",
    ///   "version": "1.3.0",
    ///   "uptime_seconds": 123,
    ///   "display_available": true,
    ///   "modalities_active": ["visual", "audio"]
    /// }
    /// ```
    fn handle_health_check(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        // Detect available modalities (capability-based, not hardcoded)
        let modalities = self.detect_active_modalities();

        JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": self.uptime_seconds(),
                "display_available": modalities.contains(&"visual"),
                "modalities_active": modalities,
            }),
        )
    }

    /// biomeOS API: announce_capabilities
    ///
    /// Returns available capabilities following biomeOS taxonomy.
    ///
    /// # TRUE PRIMAL: Capability-Based
    ///
    /// Capabilities are discovered at runtime, not hardcoded.
    ///
    /// # Response Format
    ///
    /// ```json
    /// {
    ///   "capabilities": [
    ///     "ui.render",
    ///     "ui.visualization",
    ///     "ui.graph",
    ///     "ui.terminal",
    ///     "ui.audio",
    ///     "ui.framebuffer"
    ///   ]
    /// }
    /// ```
    fn handle_announce_capabilities(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        // Detect available capabilities (runtime, not hardcoded)
        let capabilities = self.detect_capabilities();

        JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "capabilities": capabilities,
            }),
        )
    }

    /// biomeOS API: ui.render
    ///
    /// Renders content following biomeOS convention.
    ///
    /// # Request Format
    ///
    /// ```json
    /// {
    ///   "content_type": "graph",
    ///   "data": {
    ///     "nodes": [...],
    ///     "edges": [...]
    ///   },
    ///   "options": {
    ///     "title": "Primal Network",
    ///     "layout": "force-directed"
    ///   }
    /// }
    /// ```
    ///
    /// # Response Format
    ///
    /// ```json
    /// {
    ///   "rendered": true,
    ///   "modality": "visual",
    ///   "window_id": "main"
    /// }
    /// ```
    async fn handle_ui_render(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "params must be an object",
                );
            }
        };

        let content_type = params
            .get("content_type")
            .and_then(|v| v.as_str())
            .unwrap_or("graph");

        let data = params.get("data").cloned().unwrap_or(json!({}));

        // Route to appropriate rendering engine based on content_type
        match content_type {
            "graph" => {
                // Update graph engine with new data
                let result = self.render_graph_data(data).await;

                match result {
                    Ok(_) => JsonRpcResponse::success(
                        request.id.clone(),
                        json!({
                            "rendered": true,
                            "modality": "visual",
                            "window_id": "main"
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        request.id.clone(),
                        error_codes::INTERNAL_ERROR,
                        format!("Render error: {e}"),
                    ),
                }
            }
            _ => JsonRpcResponse::error(
                request.id.clone(),
                error_codes::INVALID_PARAMS,
                format!("Unsupported content_type: {content_type}"),
            ),
        }
    }

    /// biomeOS API: ui.display_status
    ///
    /// Updates primal status display in UI.
    ///
    /// # Request Format
    ///
    /// ```json
    /// {
    ///   "primal_name": "beardog",
    ///   "status": {
    ///     "health": "healthy",
    ///     "tunnels_active": 3,
    ///     "encryption_rate": "1.2 GB/s"
    ///   }
    /// }
    /// ```
    fn handle_ui_display_status(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "params must be an object",
                );
            }
        };

        let primal_name = params
            .get("primal_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = params.get("status").cloned().unwrap_or(json!({}));

        // TODO: Integrate with SystemDashboard to update primal status
        // For now, just acknowledge receipt
        debug!("Status update for {}: {:?}", primal_name, status);

        JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "updated": true,
                "primal": primal_name,
            }),
        )
    }

    // ===== Helper Methods =====

    /// Detect active modalities (capability-based)
    ///
    /// # TRUE PRIMAL: Runtime Detection
    ///
    /// Modalities are detected at runtime, not hardcoded.
    fn detect_active_modalities(&self) -> Vec<&'static str> {
        let mut modalities = Vec::new();

        // Always available (terminal fallback)
        modalities.push("terminal");

        // Check for display (visual mode)
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            modalities.push("visual");
        }

        // Check for audio (if audio devices available)
        // TODO: More sophisticated audio detection
        modalities.push("audio");

        // Check for framebuffer
        if std::path::Path::new("/dev/fb0").exists() {
            modalities.push("framebuffer");
        }

        modalities
    }

    /// Detect available capabilities (runtime, not hardcoded)
    ///
    /// # TRUE PRIMAL: Capability-Based
    ///
    /// Returns capabilities following biomeOS taxonomy.
    fn detect_capabilities(&self) -> Vec<&'static str> {
        let mut capabilities = Vec::new();

        // Core UI capabilities (always available)
        capabilities.push("ui.render");
        capabilities.push("ui.visualization");
        capabilities.push("ui.graph");

        // Modality-specific capabilities (runtime detected)
        let modalities = self.detect_active_modalities();

        if modalities.contains(&"visual") {
            // Visual capabilities available
        }

        if modalities.contains(&"terminal") {
            capabilities.push("ui.terminal");
        }

        if modalities.contains(&"audio") {
            capabilities.push("ui.audio");
        }

        if modalities.contains(&"framebuffer") {
            capabilities.push("ui.framebuffer");
        }

        capabilities
    }

    /// Render graph data from biomeOS format
    async fn render_graph_data(&self, data: Value) -> Result<()> {
        // TODO: Parse biomeOS graph format and update graph engine
        // For now, just acknowledge
        debug!("Rendering graph data: {:?}", data);
        Ok(())
    }

    // ===== Legacy Methods (Backward Compatibility) =====

    /// API: get_capabilities (legacy)
    ///
    /// Returns the capabilities of this petalTongue instance
    fn get_capabilities(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "capabilities": [
                    "ui.desktop-interface",
                    "ui.primal-interaction",
                    "visualization.graph-rendering",
                    "visualization.real-time-topology",
                    "visualization.flow-animation",
                    "ui.multi-modal",
                    "ui.awakening-experience",
                    "visualization.terminal",
                    "visualization.svg",
                    "visualization.png",
                    "visualization.egui"
                ],
                "version": env!("CARGO_PKG_VERSION"),
                "family_id": &self.family_id,
                "protocol": "json-rpc-2.0",
                "transport": "unix-socket"
            }),
        )
    }

    /// API: get_health (legacy)
    ///
    /// Returns health status of this petalTongue instance
    fn get_health(&self, id: Value) -> JsonRpcResponse {
        // Get graph stats
        let graph = self
            .graph
            .read()
            .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

        let node_count = graph.nodes().len();
        let edge_count = graph.edges().len();

        JsonRpcResponse::success(
            id,
            json!({
                "status": "healthy",
                "family_id": &self.family_id,
                "graph": {
                    "nodes": node_count,
                    "edges": edge_count
                },
                "protocol": "json-rpc-2.0"
            }),
        )
    }

    /// API: get_topology
    ///
    /// Returns the current topology view from this petalTongue instance
    fn get_topology(&self, id: Value) -> JsonRpcResponse {
        let graph = self
            .graph
            .read()
            .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

        // Build topology from graph
        let topology = json!({
            "nodes": graph.nodes().iter().map(|node| {
                json!({
                    "id": node.info.id,
                    "name": node.info.name,
                    "type": node.info.primal_type,
                    "capabilities": node.info.capabilities,
                    "health": format!("{:?}", node.info.health),
                    "position": {
                        "x": node.position.x,
                        "y": node.position.y,
                        "z": node.position.z
                    }
                })
            }).collect::<Vec<_>>(),
            "edges": graph.edges().iter().map(|edge| {
                json!({
                    "from": edge.from,
                    "to": edge.to,
                    "type": edge.edge_type
                })
            }).collect::<Vec<_>>()
        });

        JsonRpcResponse::success(id, topology)
    }

    /// API: render_graph
    ///
    /// Renders the topology to a specified format
    async fn render_graph(&self, params: Value, id: Value) -> JsonRpcResponse {
        // Parse parameters
        let format = params["format"].as_str().unwrap_or("svg");

        match format {
            "svg" => {
                // TODO: Implement SVG rendering
                // For now, return a simple response
                JsonRpcResponse::success(
                    id,
                    json!({
                        "format": "svg",
                        "data": "<svg><!-- TODO: Implement SVG rendering --></svg>",
                        "metadata": {
                            "nodes": 0,
                            "edges": 0
                        }
                    }),
                )
            }
            "png" => {
                // TODO: Implement PNG rendering
                JsonRpcResponse::success(
                    id,
                    json!({
                        "format": "png",
                        "data": "",  // Base64-encoded PNG
                        "metadata": {
                            "nodes": 0,
                            "edges": 0
                        }
                    }),
                )
            }
            "terminal" => {
                // TODO: Implement terminal rendering
                JsonRpcResponse::success(
                    id,
                    json!({
                        "format": "terminal",
                        "data": "TODO: Terminal rendering"
                    }),
                )
            }
            _ => JsonRpcResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                format!("Unsupported format: {format}"),
            ),
        }
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        // Clean up socket file on shutdown
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
    use petal_tongue_core::test_fixtures::env_test_helpers;

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
            let request = JsonRpcRequest::new("health_check", json!({}), json!(1));
            let response = server.handle_health_check(&request);
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
            let request = JsonRpcRequest::new("announce_capabilities", json!({}), json!(1));
            let response = server.handle_announce_capabilities(&request);
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
            let response = server.handle_ui_display_status(&request);
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
            let response = server.handle_ui_display_status(&request);
            assert!(response.error.is_some());
            assert_eq!(response.error.unwrap().code, error_codes::INVALID_PARAMS);
        });
    }

    #[test]
    fn test_get_topology_with_nodes() {
        use petal_tongue_core::test_fixtures::primals;
        use petal_tongue_core::LayoutAlgorithm;

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

}
