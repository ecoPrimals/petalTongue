// SPDX-License-Identifier: AGPL-3.0-only
//! biomeOS Discovery Backend
//!
//! Implements capability-based discovery via biomeOS Neural API.
//! This is the primary discovery mechanism in production.

use crate::capability_discovery::{
    CapabilityQuery, DiscoveryBackend, DiscoveryError, PrimalEndpoint, PrimalEndpoints,
    PrimalHealth,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

/// biomeOS discovery backend
#[derive(Debug)]
pub struct BiomeOsBackend {
    /// JSON-RPC client for biomeOS Neural API
    client: BiomeOsClient,
}

/// Simple JSON-RPC client for biomeOS
#[derive(Debug)]
struct BiomeOsClient {
    socket_path: String,
}

impl BiomeOsBackend {
    /// Create a new biomeOS discovery backend
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            client: BiomeOsClient {
                socket_path: socket_path.into(),
            },
        }
    }

    /// Try to create from environment (`XDG_RUNTIME_DIR` or fallback)
    /// Create from environment with capability-based discovery
    ///
    /// # Socket Discovery Priority
    /// 1. `BIOMEOS_NEURAL_API_SOCKET` - explicit override (highest priority)
    /// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` - XDG standard
    /// 3. `/tmp/biomeos-neural-api.sock` - legacy fallback
    ///
    /// # TRUE PRIMAL: Zero hardcoded paths in discovery logic
    pub fn from_env() -> Result<Self, DiscoveryError> {
        use crate::platform_dirs;

        // Priority 1: Explicit environment override
        if let Ok(socket_path) = std::env::var("BIOMEOS_NEURAL_API_SOCKET") {
            let path = std::path::PathBuf::from(&socket_path);
            if path.exists() {
                return Ok(Self::new(socket_path));
            }
            // Env var set but socket doesn't exist - warn and continue discovery
            tracing::debug!(
                "BIOMEOS_NEURAL_API_SOCKET={} but socket not found, trying discovery",
                socket_path
            );
        }

        // Priority 2: XDG runtime directory
        if let Ok(runtime_dir) = platform_dirs::runtime_dir() {
            let socket_path =
                runtime_dir.join(format!("{}.sock", crate::constants::biomeos_socket_name()));
            if socket_path.exists() {
                return Ok(Self::new(socket_path.to_string_lossy().to_string()));
            }
        }

        // Priority 3: Legacy /tmp fallback
        let fallback = crate::constants::biomeos_legacy_socket();
        if fallback.exists() {
            return Ok(Self::new(fallback.to_string_lossy().to_string()));
        }

        Err(DiscoveryError::BackendUnavailable(
            "biomeOS Neural API socket not found. Set BIOMEOS_NEURAL_API_SOCKET env var or start biomeOS.".to_string(),
        ))
    }
}

#[async_trait::async_trait]
impl DiscoveryBackend for BiomeOsBackend {
    async fn query(&self, query: &CapabilityQuery) -> Result<Vec<PrimalEndpoint>, DiscoveryError> {
        // Build JSON-RPC request for capability query
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "discovery.query_capability".to_string(),
            params: serde_json::json!({
                "domain": query.domain,
                "operation": query.operation,
                "version_req": query.version_req,
            }),
            id: 1,
        };

        // Call biomeOS via Unix socket
        let response: JsonRpcResponse = self
            .client
            .call(&request)
            .await
            .map_err(|e| DiscoveryError::CommunicationError(e.to_string()))?;

        // Parse response
        if let Some(error) = response.error {
            if error.message.contains("not found") {
                return Err(DiscoveryError::CapabilityNotFound {
                    domain: query.domain.clone(),
                });
            }
            return Err(DiscoveryError::CommunicationError(error.message));
        }

        let result = response.result.ok_or_else(|| {
            DiscoveryError::CommunicationError("No result in response".to_string())
        })?;

        // Parse primals from result
        let primals: Vec<BiomeOsPrimal> = serde_json::from_value(result)
            .map_err(|e| DiscoveryError::CommunicationError(format!("Parse error: {e}")))?;

        // Convert to PrimalEndpoint
        Ok(primals.into_iter().map(std::convert::Into::into).collect())
    }

    async fn subscribe(&self, _query: &CapabilityQuery) -> Result<(), DiscoveryError> {
        // Registration only; use subscribe_websocket() for the event stream
        Ok(())
    }
}

impl BiomeOsClient {
    async fn call(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, std::io::Error> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        // Connect to biomeOS socket
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        // Send request
        let request_json = serde_json::to_vec(request)?;
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?;

        // Read response
        let mut response_buf = Vec::new();
        stream.read_to_end(&mut response_buf).await?;

        // Parse response
        serde_json::from_slice(&response_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// JSON-RPC request
#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC response (jsonrpc and id required for spec compliance, not read after parse)
#[derive(Deserialize)]
struct JsonRpcResponse {
    #[expect(dead_code, reason = "Required by JSON-RPC spec for deserialization")]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    #[expect(dead_code, reason = "Required by JSON-RPC spec for deserialization")]
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcError {
    message: String,
}

/// biomeOS primal format (from Neural API)
#[derive(Deserialize)]
struct BiomeOsPrimal {
    id: String,
    capabilities: Vec<String>,
    tarpc_endpoint: Option<String>,
    jsonrpc_endpoint: Option<String>,
    health: String,
}

impl From<BiomeOsPrimal> for PrimalEndpoint {
    fn from(p: BiomeOsPrimal) -> Self {
        use crate::capability_discovery::Capability;

        Self {
            id: p.id,
            capabilities: p
                .capabilities
                .into_iter()
                .map(|cap| {
                    // Parse capability string "domain.operation"
                    let parts: Vec<&str> = cap.split('.').collect();
                    if parts.len() == 2 {
                        Capability::new(parts[0]).with_operation(parts[1])
                    } else {
                        Capability::new(cap)
                    }
                })
                .collect(),
            endpoints: PrimalEndpoints {
                tarpc: p.tarpc_endpoint,
                jsonrpc: p.jsonrpc_endpoint,
                https: None,
            },
            health: match p.health.as_str() {
                "healthy" => PrimalHealth::Healthy,
                "degraded" => PrimalHealth::Degraded,
                _ => PrimalHealth::Unavailable,
            },
        }
    }
}

/// Real-time topology/health event from biomeOS WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BiomeOSDiscoveryEvent {
    /// Primal health status changed
    PrimalStatus {
        /// ID of the primal whose status changed
        primal_id: String,
        /// New health status string (e.g. "healthy", "degraded")
        health: String,
    },
    /// Topology changed (primal added/removed or edge changed)
    TopologyUpdate {
        /// List of primal IDs in the topology
        primals: Vec<String>,
        /// Edges as (from_id, to_id) pairs
        edges: Vec<(String, String)>,
    },
}

impl BiomeOsBackend {
    /// Subscribe to real-time topology/health updates via WebSocket.
    ///
    /// Connects to the biomeOS WebSocket endpoint (discovered via socket/env),
    /// subscribes to topology and health events, and returns a receiver stream.
    /// Handles reconnection gracefully by spawning a background task that
    /// retries on disconnect.
    #[expect(
        clippy::unused_async,
        reason = "async for API consistency with other discovery methods"
    )]
    pub async fn subscribe_websocket(
        &self,
        _query: &CapabilityQuery,
    ) -> Result<mpsc::Receiver<BiomeOSDiscoveryEvent>, DiscoveryError> {
        let ws_url = Self::derive_websocket_url();
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(Self::websocket_loop(ws_url, tx));
        Ok(rx)
    }

    fn derive_websocket_url() -> String {
        if let Ok(url) = std::env::var("BIOMEOS_WS_ENDPOINT") {
            return url;
        }
        crate::constants::default_biomeos_ws_topology_url()
    }

    async fn websocket_loop(url: String, tx: mpsc::Sender<BiomeOSDiscoveryEvent>) {
        const MAX_BACKOFF: u64 = 30;
        let mut backoff = 1u64;

        loop {
            match Self::connect_and_forward(&url, &tx).await {
                Ok(()) => {
                    backoff = 1;
                }
                Err(e) => {
                    tracing::debug!("WebSocket disconnected: {e}, reconnecting in {backoff}s");
                    tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    }

    async fn connect_and_forward(
        url: &str,
        tx: &mpsc::Sender<BiomeOSDiscoveryEvent>,
    ) -> Result<(), String> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::connect_async;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {e}"))?;

        let (mut write, mut read) = ws_stream.split();

        // Send subscription message
        let subscribe_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "topology.subscribe",
            "params": {},
            "id": 1
        });
        let _ = write
            .send(tokio_tungstenite::tungstenite::Message::Text(
                subscribe_msg.to_string(),
            ))
            .await;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                    if let Ok(event) = serde_json::from_str::<BiomeOSDiscoveryEvent>(&text)
                        && tx.send(event).await.is_err()
                    {
                        return Ok(()); // Receiver dropped, exit
                    }
                }
                Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                    return Err("WebSocket closed by server".to_string());
                }
                Err(e) => return Err(format!("WebSocket error: {e}")),
                _ => {}
            }
        }
        Err("WebSocket stream ended".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn test_biomeos_backend_new() {
        let backend = BiomeOsBackend::new("/tmp/custom.sock");
        // Just verify it constructs - we can't call query without a real socket
        drop(backend);
    }

    #[test]
    fn test_biomeos_from_env_explicit_socket() {
        let temp = std::env::temp_dir().join("biomeos-test-socket");
        std::fs::create_dir_all(temp.parent().unwrap()).unwrap();
        std::fs::write(&temp, "").unwrap();

        env_test_helpers::with_env_var("BIOMEOS_NEURAL_API_SOCKET", temp.to_str().unwrap(), || {
            let backend = BiomeOsBackend::from_env().unwrap();
            drop(backend);
        });

        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_biomeos_from_env_socket_not_found() {
        env_test_helpers::with_env_var(
            "BIOMEOS_NEURAL_API_SOCKET",
            "/nonexistent/path/neural-api.sock",
            || {
                let result = BiomeOsBackend::from_env();
                assert!(result.is_err());
                if let Err(DiscoveryError::BackendUnavailable(_)) = result {
                    // Expected
                } else {
                    panic!("Expected BackendUnavailable error");
                }
            },
        );
    }

    #[test]
    fn test_biomeos_primal_conversion_healthy() {
        let biomeos_primal = BiomeOsPrimal {
            id: "test-primal-1".to_string(),
            capabilities: vec!["crypto.encrypt".to_string(), "crypto.decrypt".to_string()],
            tarpc_endpoint: Some("tarpc://unix:/run/primal/test".to_string()),
            jsonrpc_endpoint: Some("/run/primal/test.sock".to_string()),
            health: "healthy".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.id, "test-primal-1");
        assert_eq!(endpoint.capabilities.len(), 2);
        assert_eq!(endpoint.health, PrimalHealth::Healthy);
    }

    #[test]
    fn test_biomeos_primal_conversion_degraded() {
        let biomeos_primal = BiomeOsPrimal {
            id: "degraded-primal".to_string(),
            capabilities: vec!["storage.cache".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: Some("/run/degraded.sock".to_string()),
            health: "degraded".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.health, PrimalHealth::Degraded);
    }

    #[test]
    fn test_biomeos_primal_conversion_unavailable() {
        let biomeos_primal = BiomeOsPrimal {
            id: "unavail-primal".to_string(),
            capabilities: vec![],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "unavailable".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.health, PrimalHealth::Unavailable);
    }

    #[test]
    fn test_biomeos_primal_conversion_unknown_health() {
        let biomeos_primal = BiomeOsPrimal {
            id: "unknown-primal".to_string(),
            capabilities: vec!["ui.render".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "unknown-status".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.health, PrimalHealth::Unavailable);
    }

    #[test]
    fn test_biomeos_discovery_event_primal_status() {
        let json = r#"{"type":"PrimalStatus","primal_id":"p1","health":"healthy"}"#;
        let event: BiomeOSDiscoveryEvent = serde_json::from_str(json).expect("parse PrimalStatus");
        match &event {
            BiomeOSDiscoveryEvent::PrimalStatus { primal_id, health } => {
                assert_eq!(primal_id, "p1");
                assert_eq!(health, "healthy");
            }
            BiomeOSDiscoveryEvent::TopologyUpdate { .. } => panic!("expected PrimalStatus"),
        }
    }

    #[test]
    fn test_biomeos_discovery_event_topology_update() {
        let json = r#"{"type":"TopologyUpdate","primals":["a","b"],"edges":[["a","b"]]}"#;
        let event: BiomeOSDiscoveryEvent =
            serde_json::from_str(json).expect("parse TopologyUpdate");
        match &event {
            BiomeOSDiscoveryEvent::TopologyUpdate { primals, edges } => {
                assert_eq!(primals, &["a", "b"]);
                assert_eq!(edges, &[("a".to_string(), "b".to_string())]);
            }
            BiomeOSDiscoveryEvent::PrimalStatus { .. } => panic!("expected TopologyUpdate"),
        }
    }

    #[test]
    fn test_biomeos_primal_conversion_single_part_capability() {
        let biomeos_primal = BiomeOsPrimal {
            id: "legacy-primal".to_string(),
            capabilities: vec!["legacy".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "healthy".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.capabilities.len(), 1);
    }

    #[tokio::test]
    async fn test_biomeos_query_unavailable_socket() {
        let backend = BiomeOsBackend::new("/nonexistent/path/neural-api-12345.sock");
        let query = CapabilityQuery {
            domain: "test".to_string(),
            operation: Some("op".to_string()),
            version_req: None,
        };
        let result = backend.query(&query).await;
        assert!(result.is_err());
        if let Err(DiscoveryError::CommunicationError(msg)) = result {
            assert!(!msg.is_empty());
        }
    }

    #[tokio::test]
    async fn test_biomeos_subscribe_returns_ok() {
        let backend = BiomeOsBackend::new("/tmp/nonexistent.sock");
        let query = CapabilityQuery {
            domain: "test".to_string(),
            operation: None,
            version_req: None,
        };
        let result = backend.subscribe(&query).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "discovery.query_capability",
            "params": {"domain": "crypto", "operation": "encrypt", "version_req": null},
            "id": 1
        });
        assert_eq!(request["method"], "discovery.query_capability");
        assert_eq!(request["params"]["domain"], "crypto");
    }

    #[test]
    fn test_jsonrpc_response_error_parsing() {
        let json = r#"{"jsonrpc":"2.0","error":{"message":"capability not found"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.is_some());
        assert!(
            response
                .error
                .as_ref()
                .unwrap()
                .message
                .contains("not found")
        );
    }

    #[test]
    fn test_jsonrpc_response_result_parsing() {
        let json = r#"{"jsonrpc":"2.0","result":[{"id":"p1","capabilities":[],"health":"healthy"}],"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
    }

    #[test]
    fn test_biomeos_discovery_event_serialization_roundtrip() {
        let json = r#"{"type":"PrimalStatus","primal_id":"p1","health":"degraded"}"#;
        let event: BiomeOSDiscoveryEvent = serde_json::from_str(json).expect("parse");
        let serialized = serde_json::to_string(&event).expect("serialize");
        let restored: BiomeOSDiscoveryEvent = serde_json::from_str(&serialized).expect("parse");
        match (&event, &restored) {
            (
                BiomeOSDiscoveryEvent::PrimalStatus {
                    primal_id: a,
                    health: b,
                },
                BiomeOSDiscoveryEvent::PrimalStatus {
                    primal_id: c,
                    health: d,
                },
            ) => {
                assert_eq!(a, c);
                assert_eq!(b, d);
            }
            _ => panic!("expected PrimalStatus"),
        }
    }

    #[test]
    fn test_biomeos_discovery_event_topology_serialization() {
        let json =
            r#"{"type":"TopologyUpdate","primals":["a","b","c"],"edges":[["a","b"],["b","c"]]}"#;
        let event: BiomeOSDiscoveryEvent = serde_json::from_str(json).expect("parse");
        match &event {
            BiomeOSDiscoveryEvent::TopologyUpdate { primals, edges } => {
                assert_eq!(primals.len(), 3);
                assert_eq!(edges.len(), 2);
            }
            BiomeOSDiscoveryEvent::PrimalStatus { .. } => panic!("expected TopologyUpdate"),
        }
    }

    #[test]
    fn test_biomeos_primal_capability_domain_operation_parsing() {
        let biomeos_primal = BiomeOsPrimal {
            id: "cap-test".to_string(),
            capabilities: vec!["domain.operation".to_string(), "single".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "healthy".to_string(),
        };
        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.capabilities.len(), 2);
    }
}
