// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`NeuralApiProvider`] and [`VisualizationDataProvider`] implementation.

use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use petal_tongue_core::capability_names::socket_roles;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info};

use super::parse;

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
    pub(super) fn get_search_paths() -> Vec<PathBuf> {
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

    /// Parse primal from Neural API format to `PrimalInfo` (exposed for unit tests)
    #[cfg(test)]
    pub(super) fn parse_primal(primal: &Value) -> DiscoveryResult<PrimalInfo> {
        parse::parse_primal(primal)
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
            if let Ok(primal) = parse::parse_primal(primal_value) {
                primals.push(primal);
            }
        }

        info!("🧠 Neural API reports {} primals", primals.len());
        Ok(primals)
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        debug!("Querying Neural API for topology");

        let result = self.call_method("neural_api.get_topology", None).await?;
        parse::parse_topology_edges(&result)
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
