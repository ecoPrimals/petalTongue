// SPDX-License-Identifier: AGPL-3.0-or-later
//! Neural API-based visualization data provider
//!
//! Connects to biomeOS Neural API for unified primal discovery and proprioception.
//! This is the PREFERRED provider as Neural API is the central coordinator.

use crate::capability_parse;
use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::capability_names::socket_roles;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info};

/// Neural API visualization provider
///
/// Connects to biomeOS Neural API for:
/// - Real-time primal discovery
/// - SAME DAVE proprioception
/// - Aggregated system metrics
/// - Unified topology view
pub struct NeuralApiProvider {
    /// Path to Neural API Unix socket
    socket_path: PathBuf,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl NeuralApiProvider {
    /// Create provider with explicit socket path (for testing)
    #[cfg(any(test, feature = "test-fixtures"))]
    #[must_use]
    pub const fn with_socket_path(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Discover Neural API socket
    ///
    /// Searches for biomeos-neural-api-{family_id}.sock in standard locations.
    ///
    /// # Errors
    /// Returns `DiscoveryError::NeuralApiNotFound` if no socket found, or health check fails.
    pub async fn discover(family_id: Option<&str>) -> DiscoveryResult<Self> {
        let family = family_id
            .map(String::from)
            .or_else(|| std::env::var("FAMILY_ID").ok())
            .unwrap_or_else(|| "nat0".to_string());

        let socket_name = format!("{}-{}.sock", socket_roles::NEURAL_API, family);

        // Search in standard locations
        let search_paths = Self::get_search_paths();

        for base_path in search_paths {
            let socket_path = base_path.join(&socket_name);
            if socket_path.exists() {
                info!("🧠 Found Neural API at: {}", socket_path.display());

                // Test connectivity
                let provider = Self {
                    socket_path: socket_path.clone(),
                    request_id: std::sync::atomic::AtomicU64::new(1),
                };

                // Verify it responds
                provider.health_check().await?;

                return Ok(provider);
            }
        }

        Err(DiscoveryError::NeuralApiNotFound {
            socket_name: socket_name.clone(),
        })
    }

    /// Get standard search paths for Unix sockets
    fn get_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Priority 1: XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            paths.push(PathBuf::from(xdg_runtime));
        }

        // Priority 2: /run/user/<uid>
        let uid = petal_tongue_core::system_info::get_current_uid();
        paths.push(PathBuf::from(format!("/run/user/{uid}")));

        // Priority 3: /tmp (development)
        paths.push(PathBuf::from("/tmp"));

        paths
    }

    /// Send JSON-RPC request to Neural API
    /// Call a Neural API method (public for graph client)
    ///
    /// # Errors
    /// Returns `DiscoveryError` on connection, I/O, or JSON-RPC errors.
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> DiscoveryResult<Value> {
        let id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or_else(|| json!({})),
            "id": id
        });

        debug!("🧠 Calling Neural API: {}", method);

        // Connect to socket
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            DiscoveryError::HealthCheckFailed {
                name: "Neural API".to_string(),
                endpoint: self.socket_path.display().to_string(),
                source: e.into(),
            }
        })?;

        // Send request
        let request_str = serde_json::to_string(&request).map_err(DiscoveryError::Json)?;
        stream
            .write_all(request_str.as_bytes())
            .await
            .map_err(DiscoveryError::Io)?;
        stream.write_all(b"\n").await.map_err(DiscoveryError::Io)?;

        // Read response
        let (reader, _writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(DiscoveryError::Io)?;

        // Parse response
        let response: Value =
            serde_json::from_str(&response_line).map_err(|e| DiscoveryError::ParseError {
                data_type: "Neural API response".to_string(),
                message: e.to_string(),
            })?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            #[expect(
                clippy::cast_possible_truncation,
                reason = "JSON-RPC error codes are in -32k range; i64 fits in i32"
            )]
            return Err(DiscoveryError::JsonRpcError {
                code: error
                    .get("code")
                    .and_then(serde_json::Value::as_i64)
                    .map(|c| c as i32),
                message: msg.to_string(),
            });
        }

        // Extract result
        response
            .get("result")
            .cloned()
            .ok_or_else(|| DiscoveryError::NoResultInResponse {
                context: " (Neural API)".to_string(),
            })
    }

    /// Parse primal from Neural API format to `PrimalInfo`
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Ok wrapper for struct literal in Result chain"
    )]
    fn parse_primal(primal: &Value) -> DiscoveryResult<PrimalInfo> {
        Ok(PrimalInfo {
            id: primal["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string()
                .into(),
            name: primal["primal_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            primal_type: primal["primal_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            endpoint: primal["socket_path"].as_str().unwrap_or("").to_string(),
            capabilities: primal["capabilities"]
                .as_array()
                .map(|v| capability_parse::parse_capabilities(v))
                .unwrap_or_default(),
            health: match primal["health"].as_str() {
                Some("healthy") => PrimalHealthStatus::Healthy,
                _ => PrimalHealthStatus::Unknown,
            },
            last_seen: 0, // Neural API doesn't provide this yet
            endpoints: None,
            metadata: None,
            properties: std::collections::HashMap::default(),
            #[expect(deprecated)]
            trust_level: None,
            #[expect(deprecated)]
            family_id: None,
        })
    }

    /// Get proprioception data from Neural API
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails or response is invalid.
    pub async fn get_proprioception(
        &self,
    ) -> DiscoveryResult<petal_tongue_core::ProprioceptionData> {
        let result = self
            .call_method("neural_api.get_proprioception", None)
            .await?;
        serde_json::from_value(result).map_err(|e| DiscoveryError::ParseError {
            data_type: "proprioception data".to_string(),
            message: e.to_string(),
        })
    }

    /// Get system metrics from Neural API
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails.
    pub async fn get_metrics(&self) -> DiscoveryResult<Value> {
        self.call_method("neural_api.get_metrics", None).await
    }
}

#[async_trait]
impl VisualizationDataProvider for NeuralApiProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!("Querying Neural API for all primals");

        let result = self.call_method("primal.list", None).await?;

        // Support both formats: direct array or { primals: [...] }
        let primals_array = result["primals"]
            .as_array()
            .or_else(|| result.as_array())
            .ok_or_else(|| DiscoveryError::ExpectedArray {
                context: " of primals".to_string(),
            })?;

        let mut primals = Vec::new();
        for primal_value in primals_array {
            if let Ok(primal) = Self::parse_primal(primal_value) {
                primals.push(primal);
            }
        }

        info!("🧠 Neural API reports {} primals", primals.len());
        Ok(primals)
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        debug!("Querying Neural API for topology");

        let result = self.call_method("neural_api.get_topology", None).await?;

        let connections =
            result["connections"]
                .as_array()
                .ok_or_else(|| DiscoveryError::ExpectedArray {
                    context: " of connections".to_string(),
                })?;

        let mut edges = Vec::new();
        for conn in connections {
            edges.push(TopologyEdge {
                from: conn["from"].as_str().unwrap_or("").to_string().into(),
                to: conn["to"].as_str().unwrap_or("").to_string().into(),
                edge_type: conn["connection_type"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                capability: None,
                label: None,
                metrics: None,
            });
        }

        Ok(edges)
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        let result = self
            .call_method("neural_api.get_proprioception", None)
            .await?;

        let health_status = result["health"]["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(format!("Neural API: {health_status}"))
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Neural API (Central Coordinator)".to_string(),
            endpoint: self.socket_path.display().to_string(),
            protocol: "unix+jsonrpc".to_string(),
            capabilities: vec![
                "primal-discovery".to_string(),
                "proprioception".to_string(),
                "metrics".to_string(),
                "topology".to_string(),
                "coordination".to_string(),
            ],
        }
    }
}

/// Create a mock Neural API Unix socket server for testing
#[cfg(test)]
async fn create_mock_neural_api_server(
    socket_path: &std::path::Path,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let _ = std::fs::remove_file(socket_path);
    let listener = tokio::net::UnixListener::bind(socket_path)?;
    let handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_neural_api_connection(stream));
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(handle)
}

#[cfg(test)]
async fn handle_neural_api_connection(mut stream: tokio::net::UnixStream) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    while reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
            let method = request["method"].as_str().unwrap_or("");
            let id = request["id"].clone();
            let result = match method {
                "primal.list" => serde_json::json!({
                    "primals": [{"id": "p1", "primal_type": "test", "socket_path": "/tmp/p1.sock",
                        "capabilities": ["viz"], "health": "healthy"}]
                }),
                "neural_api.get_proprioception" => {
                    let now = chrono::Utc::now();
                    serde_json::json!({
                        "timestamp": now.to_rfc3339(),
                        "family_id": "test",
                        "health": {"percentage": 100.0, "status": "healthy"},
                        "confidence": 90.0,
                        "sensory": {"active_sockets": 2, "last_scan": now.to_rfc3339()},
                        "self_awareness": {"knows_about": 1, "can_coordinate": true,
                            "has_security": false, "has_discovery": true, "has_compute": false},
                        "motor": {"can_deploy": false, "can_execute_graphs": true,
                            "can_coordinate_primals": true},
                        "afferent_channels": [],
                        "efferent_channels": []
                    })
                }
                "neural_api.get_metrics" => {
                    serde_json::json!({"cpu_percent": 10, "memory_mb": 128})
                }
                "neural_api.get_topology" => serde_json::json!({
                    "connections": [{"from": "p1", "to": "p2", "connection_type": "trust"}]
                }),
                "neural_api.save_graph" => serde_json::json!({"graph_id": "g-saved-123"}),
                "neural_api.load_graph" => serde_json::json!({"graph": {"nodes": [], "edges": []}}),
                "neural_api.list_graphs" => serde_json::json!({
                    "graphs": [{"id": "g1", "name": "Graph 1", "description": null,
                        "created_at": "2026-01-01", "modified_at": "2026-01-02",
                        "node_count": 2, "edge_count": 1}]
                }),
                "neural_api.execute_graph" => serde_json::json!({"execution_id": "exec-456"}),
                "neural_api.get_execution_status" => serde_json::json!({
                    "execution_id": "exec-456", "graph_id": "g1", "status": "completed",
                    "started_at": "2026-01-01T00:00:00Z", "completed_at": "2026-01-01T00:01:00Z",
                    "error": null, "output": {"result": "ok"}
                }),
                "neural_api.cancel_execution"
                | "neural_api.delete_graph"
                | "neural_api.update_graph_metadata" => serde_json::json!({}),
                _ => serde_json::json!({"error": "Method not found"}),
            };
            let response = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": id});
            let response_str = serde_json::to_string(&response).unwrap() + "\n";
            let _ = writer.write_all(response_str.as_bytes()).await;
            let _ = writer.flush().await;
        }
        line.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_paths() {
        let paths = NeuralApiProvider::get_search_paths();
        assert!(!paths.is_empty());
        // Should always have /tmp as fallback
        assert!(paths.iter().any(|p| p.to_str() == Some("/tmp")));
    }

    #[test]
    fn test_search_paths_with_xdg_runtime() {
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_RUNTIME_DIR",
            "/custom/runtime",
            || {
                let paths = NeuralApiProvider::get_search_paths();
                assert_eq!(
                    paths.first().and_then(|p| p.to_str()),
                    Some("/custom/runtime")
                );
            },
        );
    }

    #[test]
    fn test_get_metadata() {
        let provider = NeuralApiProvider::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let metadata = provider.get_metadata();

        assert_eq!(metadata.name, "Neural API (Central Coordinator)");
        assert!(metadata.endpoint.contains("test.sock"));
        assert_eq!(metadata.protocol, "unix+jsonrpc");
        assert!(
            metadata
                .capabilities
                .contains(&"primal-discovery".to_string())
        );
        assert!(
            metadata
                .capabilities
                .contains(&"proprioception".to_string())
        );
    }

    #[test]
    fn test_jsonrpc_request_format() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "primal.list",
            "params": {},
            "id": 1
        });
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "primal.list");
        assert!(request["params"].is_object());
    }

    #[test]
    fn test_jsonrpc_request_with_params() {
        let params = json!({"graph_id": "g-1"});
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.load_graph",
            "params": params,
            "id": 2
        });
        assert_eq!(request["params"]["graph_id"], "g-1");
    }

    #[test]
    fn test_search_paths_contains_uid() {
        let paths = NeuralApiProvider::get_search_paths();
        let uid = petal_tongue_core::system_info::get_current_uid();
        let run_user = format!("/run/user/{uid}");
        assert!(
            paths.iter().any(|p| p.to_str() == Some(&run_user)),
            "paths should include /run/user/<uid>"
        );
    }

    #[test]
    fn test_parse_primal_full() {
        let primal = serde_json::json!({
            "id": "p1",
            "primal_type": "airSpring",
            "socket_path": "/run/user/1000/p1.sock",
            "capabilities": ["science.et0", "visualization"],
            "health": "healthy"
        });
        let info = NeuralApiProvider::parse_primal(&primal).unwrap();
        assert_eq!(info.id.as_str(), "p1");
        assert_eq!(info.name, "airSpring");
        assert_eq!(info.endpoint, "/run/user/1000/p1.sock");
        assert_eq!(info.capabilities.len(), 2);
        assert_eq!(info.health, PrimalHealthStatus::Healthy);
    }

    #[test]
    fn test_parse_primal_minimal() {
        let primal = serde_json::json!({});
        let info = NeuralApiProvider::parse_primal(&primal).unwrap();
        assert_eq!(info.id.as_str(), "unknown");
        assert_eq!(info.name, "unknown");
        assert_eq!(info.endpoint, "");
        assert!(info.capabilities.is_empty());
        assert_eq!(info.health, PrimalHealthStatus::Unknown);
    }

    #[test]
    fn test_parse_primal_health_unknown() {
        let primal = serde_json::json!({
            "id": "p2",
            "primal_type": "test",
            "health": "degraded"
        });
        let info = NeuralApiProvider::parse_primal(&primal).unwrap();
        assert_eq!(info.health, PrimalHealthStatus::Unknown);
    }

    #[test]
    fn test_parse_primal_capabilities_empty_array() {
        let primal = serde_json::json!({
            "id": "p3",
            "primal_type": "test",
            "capabilities": []
        });
        let info = NeuralApiProvider::parse_primal(&primal).unwrap();
        assert!(info.capabilities.is_empty());
    }

    #[test]
    fn test_jsonrpc_error_extraction() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "Invalid request"},
            "id": 1
        });
        let msg = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        assert_eq!(msg, "Invalid request");
    }

    #[test]
    fn test_parse_primal_socket_path_empty() {
        let primal = serde_json::json!({
            "id": "p4",
            "primal_type": "test",
            "socket_path": ""
        });
        let info = NeuralApiProvider::parse_primal(&primal).unwrap();
        assert_eq!(info.endpoint, "");
    }

    #[test]
    fn test_parse_primal_capabilities_non_string_filtered() {
        let primal = serde_json::json!({
            "id": "p5",
            "primal_type": "test",
            "socket_path": "/tmp/p5.sock",
            "capabilities": ["valid", 123, null, "also-valid"]
        });
        let info = NeuralApiProvider::parse_primal(&primal).unwrap();
        assert_eq!(info.capabilities.len(), 2);
        assert!(info.capabilities.contains(&"valid".to_string()));
        assert!(info.capabilities.contains(&"also-valid".to_string()));
    }

    #[test]
    fn test_topology_connection_parsing_structure() {
        let conn = serde_json::json!({
            "from": "primal-a",
            "to": "primal-b",
            "connection_type": "trust"
        });
        let from = conn["from"].as_str().unwrap_or("").to_string();
        let to = conn["to"].as_str().unwrap_or("").to_string();
        let edge_type = conn["connection_type"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        assert_eq!(from, "primal-a");
        assert_eq!(to, "primal-b");
        assert_eq!(edge_type, "trust");
    }

    #[test]
    fn test_topology_connection_missing_fields() {
        let conn = serde_json::json!({});
        let from = conn["from"].as_str().unwrap_or("").to_string();
        let to = conn["to"].as_str().unwrap_or("").to_string();
        assert_eq!(from, "");
        assert_eq!(to, "");
    }

    #[test]
    fn test_socket_name_format() {
        let family = "nat0";
        let socket_name = format!("biomeos-neural-api-{family}.sock");
        assert_eq!(socket_name, "biomeos-neural-api-nat0.sock");
    }

    #[test]
    fn test_jsonrpc_result_extraction() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"health": {"status": "ok"}},
            "id": 1
        });
        let result = response.get("result").cloned().unwrap();
        let status = result["health"]["status"].as_str().unwrap();
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_primals_array_format() {
        let result = serde_json::json!({"primals": [{"id": "p1", "primal_type": "t"}]});
        let primals = result["primals"].as_array().unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0]["id"], "p1");
    }

    #[test]
    fn test_primals_direct_array_format() {
        let result = serde_json::json!([{"id": "p1", "primal_type": "t"}]);
        let arr = result.as_array();
        assert!(arr.is_some());
        assert_eq!(arr.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_neural_api_call_method_get_primals() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("neural.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = NeuralApiProvider::with_socket_path(sock_path.clone());
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].id.as_str(), "p1");
    }

    #[tokio::test]
    async fn test_neural_api_call_method_get_topology() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("neural-topology.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = NeuralApiProvider::with_socket_path(sock_path);
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology.len(), 1);
        assert_eq!(topology[0].from.as_str(), "p1");
        assert_eq!(topology[0].to.as_str(), "p2");
    }

    #[tokio::test]
    async fn test_neural_api_health_check() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("neural-health.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = NeuralApiProvider::with_socket_path(sock_path);
        let health = provider.health_check().await.unwrap();
        assert!(health.contains("healthy"));
    }

    #[tokio::test]
    async fn test_neural_api_get_proprioception() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("neural-proprio.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = NeuralApiProvider::with_socket_path(sock_path);
        let proprio = provider.get_proprioception().await.unwrap();
        assert_eq!(proprio.family_id, "test");
        assert_eq!(
            proprio.health.status,
            petal_tongue_core::ProprioceptionHealthStatus::Healthy
        );
    }

    #[tokio::test]
    async fn test_neural_api_get_metrics() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("neural-metrics.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = NeuralApiProvider::with_socket_path(sock_path);
        let metrics = provider.get_metrics().await.unwrap();
        assert_eq!(metrics["cpu_percent"], 10);
        assert_eq!(metrics["memory_mb"], 128);
    }

    #[tokio::test]
    async fn test_neural_api_connection_failure() {
        let provider = NeuralApiProvider::with_socket_path(PathBuf::from(
            "/tmp/nonexistent-neural-api-xyz-99999.sock",
        ));
        let result = provider.call_method("primal.list", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_neural_graph_client_save_load_list_execute() {
        use crate::NeuralGraphClient;

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("neural-graph.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = NeuralApiProvider::with_socket_path(sock_path.clone());
        let client = NeuralGraphClient::new(&provider);

        let graph_id = client
            .save_graph(serde_json::json!({"nodes": [], "edges": []}))
            .await
            .unwrap();
        assert_eq!(graph_id, "g-saved-123");

        let graph = client.load_graph("g1").await.unwrap();
        assert!(graph.get("nodes").is_some());

        let graphs = client.list_graphs().await.unwrap();
        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0].id, "g1");

        let exec_id = client
            .execute_graph("g1", Some(serde_json::json!({"param": 1})))
            .await
            .unwrap();
        assert_eq!(exec_id, "exec-456");

        let status = client.get_execution_status("exec-456").await.unwrap();
        assert_eq!(status.status, crate::ExecutionStatus::Completed);

        client.cancel_execution("exec-456").await.unwrap();
        client.delete_graph("g1").await.unwrap();
        client
            .update_graph_metadata("g1", Some("New".to_string()), Some("Desc".to_string()))
            .await
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "test-fixtures")]
    async fn test_neural_api_discover_with_mock_socket() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("biomeos-neural-api-testfam.sock");
        let _server = create_mock_neural_api_server(&sock_path).await.unwrap();

        let provider = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
            "XDG_RUNTIME_DIR",
            dir.path().to_str().unwrap(),
            || async { NeuralApiProvider::discover(Some("testfam")).await },
        )
        .await
        .unwrap();

        assert!(
            provider
                .get_metadata()
                .endpoint
                .contains("biomeos-neural-api-testfam")
        );
    }

    #[tokio::test]
    async fn test_neural_graph_client_connection_failure() {
        use crate::NeuralGraphClient;

        let provider = NeuralApiProvider::with_socket_path(PathBuf::from(
            "/tmp/nonexistent-neural-xyz-88888.sock",
        ));
        let client = NeuralGraphClient::new(&provider);

        let result = client.save_graph(serde_json::json!({})).await;
        assert!(result.is_err());

        let result = client.load_graph("g1").await;
        assert!(result.is_err());

        let result = client.list_graphs().await;
        assert!(result.is_err());
    }
}
