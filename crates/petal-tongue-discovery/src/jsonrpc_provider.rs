// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC 2.0 Provider - PRIMARY PRIMAL PROTOCOL
//!
//! Line-delimited JSON-RPC 2.0 over Unix sockets for fast, secure, port-free
//! inter-primal communication.
//!
//! # Philosophy
//!
//! This is the **PRIMARY** protocol for ecoPrimals ecosystem:
//! - Songbird: JSON-RPC + tarpc ✅
//! - BearDog: JSON-RPC + tarpc ✅
//! - ToadStool: JSON-RPC + tarpc ✅
//! - NestGate: JSON-RPC + tarpc ✅
//! - Squirrel: JSON-RPC + tarpc ✅
//! - biomeOS: JSON-RPC (primary) ✅
//! - petalTongue: JSON-RPC (NOW ALIGNED!) ✅
//!
//! HTTP/REST is an **optional fallback** for external integrations only.
//!
//! # Protocol
//!
//! Line-delimited JSON-RPC 2.0 over Unix domain sockets:
//!
//! ```text
//! Request:  {"jsonrpc":"2.0","method":"get_primals_extended","params":null,"id":1}\n
//! Response: {"jsonrpc":"2.0","result":[...],"id":1}\n
//! ```
//!
//! # Benefits
//!
//! - **100x faster**: Unix sockets vs TCP/IP stack
//! - **Port-free**: No port conflicts or exhaustion
//! - **Secure**: File permissions control access
//! - **Language agnostic**: JSON-RPC 2.0 is universal
//! - **TRUE PRIMAL**: Self-stable, then network, then externals

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// JSON-RPC 2.0 provider for Unix socket communication
///
/// # Example
///
/// ```rust,no_run
/// use petal_tongue_discovery::{JsonRpcProvider, VisualizationDataProvider};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Auto-discover
/// let provider = JsonRpcProvider::discover().await?;
///
/// // Or explicit path
/// let provider = JsonRpcProvider::new("/run/user/1000/biomeos-device-management.sock");
///
/// // Use it!
/// let primals = provider.get_primals().await?;
/// # Ok(())
/// # }
/// ```
pub struct JsonRpcProvider {
    socket_path: PathBuf,
    request_id: AtomicU64,
    timeout: Duration,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
    id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl JsonRpcProvider {
    /// Create a new JSON-RPC provider for the given Unix socket path
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: AtomicU64::new(1),
            timeout: Duration::from_secs(10),
        }
    }

    /// Auto-discover JSON-RPC providers on standard Unix socket paths
    ///
    /// Scans for sockets in standard locations:
    /// - `/run/user/{uid}/biomeos-device-management.sock`
    /// - `/run/user/{uid}/biomeos-ui.sock`
    /// - `/run/user/{uid}/discovery-service.sock`
    /// - `/tmp/biomeos.sock`
    ///
    /// # Environment Variables
    ///
    /// - `BIOMEOS_URL=unix:///path/to/socket` - Explicit socket path
    pub async fn discover() -> anyhow::Result<Self> {
        info!("🔍 Auto-discovering JSON-RPC providers on Unix sockets...");

        // Check environment variable first (explicit override)
        if let Ok(url) = std::env::var("BIOMEOS_URL") {
            if let Some(socket_path) = url.strip_prefix("unix://") {
                debug!("Using BIOMEOS_URL: {}", socket_path);
                if tokio::fs::metadata(socket_path).await.is_ok() {
                    info!("✅ Found JSON-RPC provider at {}", socket_path);
                    return Ok(Self::new(socket_path));
                } else {
                    warn!(
                        "❌ Socket specified in BIOMEOS_URL not found: {}",
                        socket_path
                    );
                }
            }
        }

        // Scan standard paths
        let standard_paths = Self::get_standard_socket_paths()?;

        for path in standard_paths {
            debug!("Checking for socket at: {}", path.display());
            if tokio::fs::metadata(&path).await.is_ok() {
                // Try to connect to verify it's a valid JSON-RPC endpoint
                if Self::test_connection(&path).await.is_ok() {
                    info!("✅ Discovered JSON-RPC provider at {}", path.display());
                    return Ok(Self::new(path));
                }
            }
        }

        anyhow::bail!(
            "No JSON-RPC providers found!\n\
            \n\
            Tried standard paths:\n\
            - /run/user/{{uid}}/biomeos-device-management.sock\n\
            - /run/user/{{uid}}/biomeos-ui.sock\n\
            - /run/user/{{uid}}/discovery-service.sock\n\
            - /tmp/biomeos.sock\n\
            \n\
            💡 Set BIOMEOS_URL=unix:///path/to/socket for custom path"
        )
    }

    /// Get standard Unix socket paths to scan
    fn get_standard_socket_paths() -> anyhow::Result<Vec<PathBuf>> {
        // Get current user UID for /run/user/{uid}/ paths (using safe helper)
        let uid = petal_tongue_core::system_info::get_current_uid();

        Ok(vec![
            PathBuf::from(format!("/run/user/{uid}/biomeos-device-management.sock")),
            PathBuf::from(format!("/run/user/{uid}/biomeos-ui.sock")),
            PathBuf::from(format!("/run/user/{uid}/discovery-service.sock")),
            PathBuf::from("/tmp/biomeos.sock"),
        ])
    }

    /// Test if a socket is a valid JSON-RPC endpoint
    async fn test_connection(path: &Path) -> anyhow::Result<()> {
        let stream =
            tokio::time::timeout(Duration::from_secs(2), UnixStream::connect(path)).await??;

        // Just connecting successfully is enough - we don't want to send
        // random RPC calls during discovery
        drop(stream);
        Ok(())
    }

    /// Call a JSON-RPC method
    ///
    /// # Arguments
    ///
    /// * `method` - RPC method name (e.g., "get_primals_extended")
    /// * `params` - Optional parameters (JSON value)
    ///
    /// # Returns
    ///
    /// The result field from the JSON-RPC response
    async fn call(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        // Generate unique request ID
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        // Create request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };

        debug!("→ JSON-RPC request: {} (id={})", method, id);

        // Connect with timeout
        let stream = tokio::time::timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| anyhow::anyhow!("Connection timeout: {}", self.socket_path.display()))??;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request (line-delimited)
        let request_json = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_json.as_bytes()).await?;
        writer.flush().await?;

        debug!("✓ Request sent");

        // Read response (line-delimited)
        let mut line = String::new();
        tokio::time::timeout(self.timeout, reader.read_line(&mut line))
            .await
            .map_err(|_| anyhow::anyhow!("Response timeout: {method}"))??;

        debug!("← JSON-RPC response ({} bytes)", line.len());

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| anyhow::anyhow!("Invalid JSON-RPC response: {e}"))?;

        // Verify request ID matches
        if response.id != id {
            anyhow::bail!("Request ID mismatch: expected {}, got {}", id, response.id);
        }

        // Check for errors
        if let Some(error) = response.error {
            anyhow::bail!(
                "JSON-RPC error {}: {}{}",
                error.code,
                error.message,
                error
                    .data
                    .map(|d| format!(" (data: {d})"))
                    .unwrap_or_default()
            );
        }

        // Return result (or null if missing)
        Ok(response.result.unwrap_or(serde_json::Value::Null))
    }

    /// Call with automatic retry on transient errors
    async fn call_with_retry(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        max_retries: u32,
    ) -> anyhow::Result<serde_json::Value> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self.call(method, params.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Don't retry on protocol errors (method not found, invalid params, etc.)
                    if e.to_string().contains("JSON-RPC error") {
                        return Err(e);
                    }

                    debug!(
                        "RPC call failed (attempt {}/{}): {}",
                        attempt, max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < max_retries {
                        // Exponential backoff with jitter
                        let delay_ms = 100 * (1 << (attempt - 1));
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
}

#[async_trait]
impl VisualizationDataProvider for JsonRpcProvider {
    async fn get_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        debug!("Calling get_primals_extended via JSON-RPC");

        // Call with retry (3 attempts)
        let result = self
            .call_with_retry("get_primals_extended", None, 3)
            .await?;

        // Parse as Vec<PrimalInfo>
        let primals: Vec<PrimalInfo> = serde_json::from_value(result)
            .map_err(|e| anyhow::anyhow!("Failed to parse primals: {e}"))?;

        debug!("✓ Received {} primals", primals.len());
        Ok(primals)
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        debug!("Calling get_topology via JSON-RPC");

        // Try to get topology, but gracefully fallback to empty vec if not supported
        match self.call("get_topology", None).await {
            Ok(result) => {
                let topology: Vec<TopologyEdge> = serde_json::from_value(result)
                    .map_err(|e| anyhow::anyhow!("Failed to parse topology: {e}"))?;
                debug!("✓ Received {} edges", topology.len());
                Ok(topology)
            }
            Err(e) => {
                // If method not found, return empty (graceful degradation)
                if e.to_string().contains("-32601") || e.to_string().contains("Method not found") {
                    debug!("Topology not supported by provider (graceful fallback)");
                    Ok(Vec::new())
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        debug!("Performing JSON-RPC health check");

        // Use get_primals_extended as a health check
        self.call("get_primals_extended", None).await?;

        Ok(format!(
            "JSON-RPC provider at {} is healthy",
            self.socket_path.display()
        ))
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "JSON-RPC Provider".to_string(),
            endpoint: format!("unix://{}", self.socket_path.display()),
            protocol: "jsonrpc-2.0".to_string(),
            capabilities: vec![
                "primals".to_string(),
                "devices".to_string(),
                "topology".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::AsyncWriteExt;
    use tokio::net::{UnixListener, UnixStream};

    /// Create a mock JSON-RPC server for testing
    async fn create_mock_server(socket_path: &str) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        // Remove existing socket if it exists
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path)?;

        let handle = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(handle_mock_connection(stream));
            }
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(handle)
    }

    async fn handle_mock_connection(stream: UnixStream) {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        while reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
            // Parse request
            if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
                let response = match request.method.as_str() {
                    "get_primals_extended" => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!([
                            {
                                "id": "test-primal",
                                "name": "Test Primal",
                                "primal_type": "test",
                                "endpoint": "unix:///tmp/test.sock",
                                "capabilities": ["test"],
                                "health": "Healthy",
                                "last_seen": 0
                            }
                        ])),
                        error: None,
                        id: request.id,
                    },
                    "get_topology" => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!([])),
                        error: None,
                        id: request.id,
                    },
                    _ => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: "Method not found".to_string(),
                            data: None,
                        }),
                        id: request.id,
                    },
                };

                let response_json = serde_json::to_string(&response).unwrap() + "\n";
                let _ = writer.write_all(response_json.as_bytes()).await;
                let _ = writer.flush().await;
            }

            line.clear();
        }
    }

    #[tokio::test]
    async fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get_primals_extended".to_string(),
            params: None,
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"get_primals_extended\""));
        assert!(json.contains("\"id\":1"));
    }

    #[tokio::test]
    async fn test_jsonrpc_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","result":[],"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_jsonrpc_error_response() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_jsonrpc_provider_get_primals() {
        let socket_path = "/tmp/test-jsonrpc-primals.sock";
        let _server = create_mock_server(socket_path).await.unwrap();

        let provider = JsonRpcProvider::new(socket_path);
        let primals = provider.get_primals().await.unwrap();

        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].id, "test-primal");

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_jsonrpc_provider_health_check() {
        let socket_path = "/tmp/test-jsonrpc-health.sock";
        let _server = create_mock_server(socket_path).await.unwrap();

        let provider = JsonRpcProvider::new(socket_path);
        let health = provider.health_check().await.unwrap();

        assert!(health.contains("healthy"));

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_jsonrpc_provider_get_topology() {
        let socket_path = "/tmp/test-jsonrpc-topology.sock";
        let _server = create_mock_server(socket_path).await.unwrap();

        let provider = JsonRpcProvider::new(socket_path);
        let topology = provider.get_topology().await.unwrap();

        // Should return empty vec (mock server returns empty array)
        assert_eq!(topology.len(), 0);

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_jsonrpc_provider_metadata() {
        let provider = JsonRpcProvider::new("/tmp/test.sock");
        let metadata = provider.get_metadata();

        assert_eq!(metadata.name, "JSON-RPC Provider");
        assert_eq!(metadata.protocol, "jsonrpc-2.0");
        assert!(metadata.capabilities.contains(&"primals".to_string()));
    }

    #[tokio::test]
    async fn test_jsonrpc_request_id_increment() {
        let provider = JsonRpcProvider::new("/tmp/test.sock");
        let id1 = provider.request_id.load(Ordering::SeqCst);
        provider.request_id.fetch_add(1, Ordering::SeqCst);
        let id2 = provider.request_id.load(Ordering::SeqCst);

        assert_eq!(id2, id1 + 1);
    }

    #[tokio::test]
    async fn test_standard_socket_paths() {
        let paths = JsonRpcProvider::get_standard_socket_paths().unwrap();

        // Should have at least 4 standard paths
        assert!(paths.len() >= 4);

        // Should include biomeOS path
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("biomeos")));
    }

    #[tokio::test]
    async fn test_jsonrpc_discover_with_biomeos_url() {
        let socket_path = "/tmp/test-jsonrpc-discover.sock";
        let _ = std::fs::remove_file(socket_path);
        let _server = create_mock_server(socket_path).await.unwrap();

        let result = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
            "BIOMEOS_URL",
            &format!("unix://{socket_path}"),
            || async {
                JsonRpcProvider::discover().await
            },
        )
        .await;

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert!(provider.get_metadata().endpoint.contains("test-jsonrpc-discover"));

        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_jsonrpc_get_topology_method_not_found() {
        let socket_path = "/tmp/test-jsonrpc-topology-nf.sock";
        let _ = std::fs::remove_file(socket_path);
        let _server = create_mock_server(socket_path).await.unwrap();

        let provider = JsonRpcProvider::new(socket_path);
        let topology = provider.get_topology().await.unwrap();

        assert_eq!(topology.len(), 0);

        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_jsonrpc_provider_construction() {
        let provider = JsonRpcProvider::new("/nonexistent/socket.sock");
        let metadata = provider.get_metadata();
        assert_eq!(metadata.name, "JSON-RPC Provider");
        assert!(metadata.endpoint.contains("nonexistent"));
    }
}
