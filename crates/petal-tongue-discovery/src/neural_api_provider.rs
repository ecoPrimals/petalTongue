// SPDX-License-Identifier: AGPL-3.0-only
//! Neural API-based visualization data provider
//!
//! Connects to biomeOS Neural API for unified primal discovery and proprioception.
//! This is the PREFERRED provider as Neural API is the central coordinator.

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
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
    #[cfg(test)]
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
    pub async fn discover(family_id: Option<&str>) -> Result<Self> {
        let family = family_id
            .map(String::from)
            .or_else(|| std::env::var("FAMILY_ID").ok())
            .unwrap_or_else(|| "nat0".to_string());

        let socket_name = format!("biomeos-neural-api-{family}.sock");

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

        Err(anyhow::anyhow!(
            "Neural API not found. Is biomeOS nucleus serve running? (looking for {socket_name})"
        ))
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
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> Result<Value> {
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
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to Neural API socket")?;

        // Send request
        let request_str = serde_json::to_string(&request)?;
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        // Read response
        let (reader, _writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;

        // Parse response
        let response: Value =
            serde_json::from_str(&response_line).context("Failed to parse Neural API response")?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!(
                "Neural API error: {}",
                error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
            ));
        }

        // Extract result
        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in Neural API response"))
    }

    /// Parse primal from Neural API format to `PrimalInfo`
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Ok wrapper for struct literal in Result chain"
    )]
    fn parse_primal(primal: &Value) -> Result<PrimalInfo> {
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
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
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
    pub async fn get_proprioception(&self) -> Result<petal_tongue_core::ProprioceptionData> {
        let result = self
            .call_method("neural_api.get_proprioception", None)
            .await?;
        serde_json::from_value(result).context("Failed to parse proprioception data")
    }

    /// Get system metrics from Neural API
    pub async fn get_metrics(&self) -> Result<Value> {
        self.call_method("neural_api.get_metrics", None).await
    }
}

#[async_trait]
impl VisualizationDataProvider for NeuralApiProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        debug!("Querying Neural API for all primals");

        let result = self.call_method("neural_api.get_primals", None).await?;

        let primals_array = result["primals"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected primals array"))?;

        let mut primals = Vec::new();
        for primal_value in primals_array {
            if let Ok(primal) = Self::parse_primal(primal_value) {
                primals.push(primal);
            }
        }

        info!("🧠 Neural API reports {} primals", primals.len());
        Ok(primals)
    }

    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        debug!("Querying Neural API for topology");

        let result = self.call_method("neural_api.get_topology", None).await?;

        let connections = result["connections"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected connections array"))?;

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

    async fn health_check(&self) -> Result<String> {
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
            "method": "neural_api.get_primals",
            "params": {},
            "id": 1
        });
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "neural_api.get_primals");
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
}
