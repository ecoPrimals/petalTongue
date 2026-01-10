//! Unix socket JSON-RPC server for petalTongue
//!
//! Provides a port-free, file-system-based IPC mechanism for local inter-primal
//! communication following the ecoPrimals standard.

use crate::json_rpc::{error_codes, JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use petal_tongue_core::graph_engine::GraphEngine;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

/// Unix socket JSON-RPC server for petalTongue
pub struct UnixSocketServer {
    /// Path to the Unix socket
    socket_path: PathBuf,
    
    /// Node ID for this petalTongue instance
    node_id: String,
    
    /// Shared graph engine
    graph: Arc<RwLock<GraphEngine>>,
}

impl UnixSocketServer {
    /// Create a new Unix socket server
    ///
    /// Socket path will be: `/tmp/petaltongue-{node_id}.sock`
    pub fn new(node_id: String, graph: Arc<RwLock<GraphEngine>>) -> Self {
        let socket_path = PathBuf::from(format!("/tmp/petaltongue-{}.sock", node_id));
        
        Self {
            socket_path,
            node_id,
            graph,
        }
    }
    
    /// Start the Unix socket server
    ///
    /// This will bind to the Unix socket and accept connections in a loop.
    /// Each connection is handled in a separate task.
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Clean up old socket if exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
            debug!("Removed old socket: {}", self.socket_path.display());
        }
        
        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("🔌 Unix socket server listening: {}", self.socket_path.display());
        info!("   Node ID: {}", self.node_id);
        
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
                    debug!("Received request: method={}, id={}", request.method, request.id);
                    self.handle_request(request).await
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {}", e);
                    JsonRpcResponse::error(
                        json!(null),
                        error_codes::PARSE_ERROR,
                        format!("Parse error: {}", e)
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
    async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "get_capabilities" => self.get_capabilities(req.id),
            "render_graph" => self.render_graph(req.params, req.id).await,
            "get_health" => self.get_health(req.id),
            "get_topology" => self.get_topology(req.id),
            _ => {
                warn!("Unknown method: {}", req.method);
                JsonRpcResponse::error(
                    req.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method not found: {}", req.method)
                )
            }
        }
    }
    
    /// API: get_capabilities
    ///
    /// Returns the capabilities of this petalTongue instance
    fn get_capabilities(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::success(id, json!({
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
            "node_id": &self.node_id,
            "protocol": "json-rpc-2.0",
            "transport": "unix-socket"
        }))
    }
    
    /// API: get_health
    ///
    /// Returns health status of this petalTongue instance
    fn get_health(&self, id: Value) -> JsonRpcResponse {
        // Get graph stats
        let graph = self.graph.read()
            .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");
        
        let node_count = graph.nodes().len();
        let edge_count = graph.edges().len();
        
        JsonRpcResponse::success(id, json!({
            "status": "healthy",
            "node_id": &self.node_id,
            "graph": {
                "nodes": node_count,
                "edges": edge_count
            },
            "protocol": "json-rpc-2.0"
        }))
    }
    
    /// API: get_topology
    ///
    /// Returns the current topology view from this petalTongue instance
    fn get_topology(&self, id: Value) -> JsonRpcResponse {
        let graph = self.graph.read()
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
                JsonRpcResponse::success(id, json!({
                    "format": "svg",
                    "data": "<svg><!-- TODO: Implement SVG rendering --></svg>",
                    "metadata": {
                        "nodes": 0,
                        "edges": 0
                    }
                }))
            }
            "png" => {
                // TODO: Implement PNG rendering
                JsonRpcResponse::success(id, json!({
                    "format": "png",
                    "data": "",  // Base64-encoded PNG
                    "metadata": {
                        "nodes": 0,
                        "edges": 0
                    }
                }))
            }
            "terminal" => {
                // TODO: Implement terminal rendering
                JsonRpcResponse::success(id, json!({
                    "format": "terminal",
                    "data": "TODO: Terminal rendering"
                }))
            }
            _ => {
                JsonRpcResponse::error(
                    id,
                    error_codes::INVALID_PARAMS,
                    format!("Unsupported format: {}", format)
                )
            }
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

    #[test]
    fn test_unix_socket_server_creation() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let server = UnixSocketServer::new("test-node".to_string(), graph);
        
        assert_eq!(server.node_id, "test-node");
        assert_eq!(server.socket_path.to_str().unwrap(), "/tmp/petaltongue-test-node.sock");
    }

    #[test]
    fn test_get_capabilities_response() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let server = UnixSocketServer::new("test-node".to_string(), graph);
        
        let response = server.get_capabilities(json!(1));
        
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["capabilities"].is_array());
        assert_eq!(result["node_id"], "test-node");
    }

    #[test]
    fn test_get_health_response() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let server = UnixSocketServer::new("test-node".to_string(), graph);
        
        let response = server.get_health(json!(1));
        
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "healthy");
        assert_eq!(result["node_id"], "test-node");
    }
}

