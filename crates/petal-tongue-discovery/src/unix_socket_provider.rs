// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix socket discovery provider
//!
//! Discovers primals via Unix domain sockets by scanning for .sock files
//! and querying their capabilities via JSON-RPC.
//!
//! MODERN IDIOMATIC RUST:
//! - Fully async/await (no blocking `std::fs` operations)
//! - Concurrent socket probing for performance
//! - Non-blocking directory traversal
//! - Aggressive timeouts to prevent hanging on unresponsive sockets

use crate::capability_parse;
use crate::errors::{DiscoveryError, DiscoveryResult};
use futures_util::future::join_all;
use petal_tongue_core::types::{PrimalHealthStatus, PrimalInfo};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

/// Unix socket discovery provider
///
/// Discovers primals by scanning for Unix socket files and querying their capabilities.
#[derive(Debug, Clone)]
pub struct UnixSocketProvider {
    /// Directories to search for Unix sockets
    pub(crate) search_paths: Vec<PathBuf>,
}

impl UnixSocketProvider {
    /// Create a new Unix socket provider
    ///
    /// Searches for Unix sockets in:
    /// 1. $`XDG_RUNTIME_DIR` (e.g., /run/user/1000) - biomeOS convention
    /// 2. /tmp - fallback for development
    /// 3. /var/run/ecoPrimals - alternative runtime directory
    #[must_use]
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Priority 1: XDG_RUNTIME_DIR (biomeOS convention)
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            search_paths.push(PathBuf::from(xdg_runtime));
        }

        // Priority 2: /run/user/<uid> (fallback if XDG not set)
        // EVOLVED: Now using safe rustix-based function from core (was unsafe libc::getuid())
        let uid = petal_tongue_core::system_info::get_current_uid();
        search_paths.push(PathBuf::from(format!("/run/user/{uid}")));

        // Priority 3: /tmp (development fallback)
        search_paths.push(PathBuf::from("/tmp"));

        // Priority 4: /var/run/ecoPrimals (alternative)
        search_paths.push(PathBuf::from("/var/run/ecoPrimals"));

        Self { search_paths }
    }

    /// Discover all primals via Unix sockets
    ///
    /// # Errors
    /// Returns `DiscoveryError` on I/O, JSON-RPC, or connection errors.
    pub async fn discover(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        let mut primals = Vec::new();

        for search_path in &self.search_paths {
            // Non-blocking check if path exists
            if !tokio::fs::try_exists(search_path).await.unwrap_or(false) {
                debug!("Search path does not exist: {}", search_path.display());
                continue;
            }

            // Non-blocking directory read
            let mut entries = match tokio::fs::read_dir(search_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("Failed to read directory {}: {}", search_path.display(), e);
                    continue;
                }
            };

            // Process entries concurrently for better performance
            let mut probe_futures = Vec::new();
            let provider = Arc::new(self.clone());

            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                let path = entry.path();

                // Look for .sock files (Unix domain sockets)
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    let provider = Arc::clone(&provider);
                    let path = path.clone();
                    probe_futures.push(async move {
                        match provider.probe_socket(&path).await {
                            Ok(info) => {
                                info!(
                                    "🔍 Discovered primal via Unix socket: {} ({})",
                                    info.name,
                                    path.display()
                                );
                                Some(info)
                            }
                            Err(e) => {
                                debug!("Failed to probe socket {}: {}", path.display(), e);
                                None
                            }
                        }
                    });
                }
            }

            // Wait for all probes to complete concurrently
            let results = join_all(probe_futures).await;
            primals.extend(results.into_iter().flatten());
        }

        Ok(primals)
    }

    /// Probe a Unix socket for primal information
    ///
    /// Uses aggressive timeout to prevent hanging on unresponsive sockets.
    async fn probe_socket(&self, path: &Path) -> DiscoveryResult<PrimalInfo> {
        // CRITICAL: Wrap socket connect in timeout to prevent hanging
        // Unresponsive sockets are the #1 cause of test hangs
        let connect_timeout = Duration::from_millis(100);

        let stream = match tokio::time::timeout(connect_timeout, UnixStream::connect(path)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                debug!("Socket connection failed for {}: {}", path.display(), e);
                return Err(DiscoveryError::Io(e));
            }
            Err(_) => {
                debug!("Socket connection timeout for {}", path.display());
                return Err(DiscoveryError::ConnectionTimeout {
                    endpoint: path.display().to_string(),
                });
            }
        };

        let request = json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "params": {},
            "id": 1
        });

        let rpc_timeout = Duration::from_millis(200);

        let response = match tokio::time::timeout(rpc_timeout, self.send_request(stream, request))
            .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                let err_str = e.to_string();
                if err_str.contains("-32601") || err_str.contains("Method not found") {
                    let stream = tokio::time::timeout(connect_timeout, UnixStream::connect(path))
                        .await
                        .map_err(|_| DiscoveryError::ConnectionTimeout {
                            endpoint: path.display().to_string(),
                        })?
                        .map_err(DiscoveryError::Io)?;
                    let legacy_request = json!({
                        "jsonrpc": "2.0",
                        "method": "get_capabilities",
                        "params": {},
                        "id": 1
                    });
                    tokio::time::timeout(rpc_timeout, self.send_request(stream, legacy_request))
                        .await
                        .map_err(|_| DiscoveryError::RpcTimeout {
                            context: path.display().to_string(),
                        })?
                        .map_err(|e| DiscoveryError::InvalidData {
                            name: "Unix socket".to_string(),
                            reason: format!("Legacy get_capabilities failed: {e}"),
                        })?
                } else {
                    debug!("RPC request failed for {}: {}", path.display(), e);
                    return Err(e);
                }
            }
            Err(_) => {
                debug!("RPC request timeout for {}", path.display());
                return Err(DiscoveryError::RpcTimeout {
                    context: path.display().to_string(),
                });
            }
        };

        self.parse_capabilities_response(path, response)
    }

    /// Send a JSON-RPC request and receive response
    async fn send_request(&self, stream: UnixStream, request: Value) -> DiscoveryResult<Value> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request
        let request_json = serde_json::to_string(&request).map_err(DiscoveryError::Json)?;
        writer
            .write_all(request_json.as_bytes())
            .await
            .map_err(DiscoveryError::Io)?;
        writer.write_all(b"\n").await.map_err(DiscoveryError::Io)?;
        writer.flush().await.map_err(DiscoveryError::Io)?;

        // Read response
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(DiscoveryError::Io)?;

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&line).map_err(DiscoveryError::Json)?;

        if let Some(error) = response.error {
            return Err(DiscoveryError::JsonRpcError {
                code: Some(error.code),
                message: error.message,
            });
        }

        response
            .result
            .ok_or_else(|| DiscoveryError::NoResultInResponse {
                context: String::new(),
            })
    }

    /// Parse capabilities response into `PrimalInfo`
    #[expect(
        clippy::needless_pass_by_value,
        clippy::unnecessary_wraps,
        reason = "Value consumed by indexing; Ok wrapper for Result chain"
    )]
    fn parse_capabilities_response(
        &self,
        path: &Path,
        result: Value,
    ) -> DiscoveryResult<PrimalInfo> {
        let capabilities: Vec<String> = capability_parse::parse_capabilities(
            result["capabilities"].as_array().unwrap_or(&vec![]),
        );

        let node_id = result["node_id"].as_str().unwrap_or("unknown").to_string();

        let _version = result["version"].as_str().unwrap_or("unknown").to_string();

        // Derive primal type from socket name or capabilities
        let primal_type = self.infer_primal_type(path, &capabilities);

        // Derive name from socket name
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(PrimalInfo::new(
            node_id,
            name,
            primal_type,
            format!("unix://{}", path.display()),
            capabilities,
            PrimalHealthStatus::Healthy,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0)) // Robust: handles clock going backwards
                .as_secs(),
        ))
    }

    /// Infer primal type from capabilities (TRUE PRIMAL - no hardcoding)
    ///
    /// We infer the primal's role from its capabilities, not from hardcoded names.
    /// This allows ANY primal to provide ANY capability without our prior knowledge.
    #[expect(
        clippy::unused_self,
        reason = "method for consistency with parse_capabilities_response"
    )]
    fn infer_primal_type(&self, path: &Path, capabilities: &[String]) -> String {
        // CAPABILITY-BASED INFERENCE (no hardcoded primal names!)
        // We derive type from the socket filename, which the primal itself chose.
        // This is self-knowledge - the primal tells us its type via its socket name.

        let socket_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Extract the base name (before the first hyphen or dot)
        // Examples:
        //   "primal-a-instance0.sock" → "primal"
        //   "provider-b-family1.sock" → "provider"
        //   "custom-primal-123.sock" → "custom"
        let primal_type = socket_name
            .split(['-', '.'].as_ref())
            .next()
            .unwrap_or("unknown")
            .to_string();

        // If we couldn't infer from name, use capability categories as fallback
        if primal_type == "unknown" || primal_type.is_empty() {
            // Group capabilities into broad categories
            if capabilities
                .iter()
                .any(|c| c.starts_with("ui.") || c.starts_with("visualization."))
            {
                return "ui-provider".to_string();
            } else if capabilities.iter().any(|c| {
                c.starts_with("encryption.")
                    || c.starts_with("identity.")
                    || c.starts_with("security.")
            }) {
                return "security-provider".to_string();
            } else if capabilities
                .iter()
                .any(|c| c.starts_with("discovery.") || c.starts_with("registry."))
            {
                return "discovery-provider".to_string();
            } else if capabilities
                .iter()
                .any(|c| c.starts_with("storage.") || c.starts_with("persistence."))
            {
                return "storage-provider".to_string();
            } else if capabilities
                .iter()
                .any(|c| c.starts_with("compute.") || c.starts_with("execution."))
            {
                return "compute-provider".to_string();
            } else if capabilities
                .iter()
                .any(|c| c.starts_with("ai.") || c.starts_with("inference."))
            {
                return "ai-provider".to_string();
            }
        }

        primal_type
    }
}

impl Default for UnixSocketProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_socket_provider_creation() {
        let provider = UnixSocketProvider::new();
        // Should have 4 search paths: XDG_RUNTIME_DIR (if set), /run/user/<uid>, /tmp, /var/run/ecoPrimals
        assert!(provider.search_paths.len() >= 3); // At least 3 fallback paths
        assert!(provider.search_paths.iter().any(|p| p.ends_with("tmp")));
    }

    #[test]
    fn test_infer_primal_type_from_socket_name() {
        let provider = UnixSocketProvider::new();

        // Self-knowledge via socket name (preferred)
        let path = PathBuf::from("/tmp/beardog-nat0.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "beardog");

        let path = PathBuf::from("/tmp/songbird-family1.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "songbird");

        let path = PathBuf::from("/tmp/custom-primal.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "custom");

        // Socket file with only extension - should return "unknown" from fallback
        let path = PathBuf::from("/tmp/.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        // Empty stem becomes empty string after split, falls through to "unknown"
        assert!(primal_type == "unknown" || primal_type.is_empty());
    }

    #[test]
    fn test_infer_primal_type_from_capabilities() {
        let provider = UnixSocketProvider::new();

        // When socket name is unknown, infer from capabilities
        let path = PathBuf::from("/tmp/unknown.sock");

        let capabilities = vec!["ui.render".to_string(), "ui.graph".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "ui-provider");

        let capabilities = vec!["encryption.aes".to_string(), "identity.keys".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "security-provider");

        let capabilities = vec!["discovery.mdns".to_string(), "registry.primals".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "discovery-provider");

        let capabilities = vec!["storage.kv".to_string(), "persistence.files".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "storage-provider");

        let capabilities = vec!["compute.wasm".to_string(), "execution.native".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "compute-provider");

        let capabilities = vec!["ai.llm".to_string(), "inference.local".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "ai-provider");
    }

    #[test]
    fn test_socket_name_takes_precedence() {
        let provider = UnixSocketProvider::new();

        // Socket name is self-knowledge (preferred over capability inference)
        let path = PathBuf::from("/tmp/my-custom-primal-nat0.sock");
        let capabilities = vec!["ui.render".to_string()]; // UI capability

        // Should use socket name, not capability
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "my"); // First part of hyphenated name
    }

    #[test]
    fn test_unix_socket_provider_default() {
        let provider = UnixSocketProvider::default();
        assert!(provider.search_paths.len() >= 3);
        assert!(provider.search_paths.iter().any(|p| p.ends_with("tmp")));
    }

    #[test]
    fn test_infer_primal_type_visualization_capability() {
        let provider = UnixSocketProvider::new();
        let path = PathBuf::from("/tmp/unknown.sock");
        let capabilities = vec!["visualization.graph".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "ui-provider");
    }

    #[test]
    fn test_infer_primal_type_dot_in_socket_name() {
        let provider = UnixSocketProvider::new();
        let path = PathBuf::from("/tmp/foo.bar-baz.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "foo");
    }

    #[test]
    fn test_infer_primal_type_unknown_no_capabilities() {
        let provider = UnixSocketProvider::new();
        let path = PathBuf::from("/tmp/unknown.sock");
        let capabilities: Vec<String> = vec![];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "unknown");
    }

    #[test]
    fn test_search_paths_include_var_run_ecoprimals() {
        let provider = UnixSocketProvider::new();
        assert!(
            provider
                .search_paths
                .iter()
                .any(|p| p.to_string_lossy().contains("ecoPrimals"))
        );
    }

    #[test]
    fn test_search_paths_include_run_user() {
        let provider = UnixSocketProvider::new();
        assert!(
            provider
                .search_paths
                .iter()
                .any(|p| p.to_string_lossy().contains("/run/user/"))
        );
    }

    #[test]
    fn test_infer_primal_type_single_segment_name() {
        let provider = UnixSocketProvider::new();
        let path = PathBuf::from("/tmp/beardog.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "beardog");
    }

    #[test]
    fn test_parse_capabilities_response() {
        let provider = UnixSocketProvider::new();
        let path = std::path::Path::new("/tmp/test-primal-nat0.sock");
        let result = json!({
            "capabilities": ["visualization.graph", "ui.render"],
            "node_id": "test-node-123",
            "version": "1.0.0"
        });
        let info = provider.parse_capabilities_response(path, result).unwrap();
        assert_eq!(info.id.as_str(), "test-node-123");
        assert_eq!(info.name, "test-primal-nat0");
        assert_eq!(info.capabilities.len(), 2);
        assert!(info.endpoint.contains("test-primal"));
    }

    #[test]
    fn test_parse_capabilities_response_minimal() {
        let provider = UnixSocketProvider::new();
        let path = std::path::Path::new("/tmp/foo.sock");
        let result = json!({});
        let info = provider.parse_capabilities_response(path, result).unwrap();
        assert_eq!(info.id.as_str(), "unknown");
        assert_eq!(info.name, "foo");
        assert!(info.capabilities.is_empty());
    }

    #[test]
    fn test_capability_list_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "params": {},
            "id": 1
        });
        assert_eq!(request["method"], "capability.list");
        assert!(request["params"].is_object());
    }

    #[test]
    fn test_legacy_get_capabilities_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "get_capabilities",
            "params": {},
            "id": 1
        });
        assert_eq!(request["method"], "get_capabilities");
    }

    #[test]
    fn test_jsonrpc_response_no_result() {
        let response = json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "Invalid"},
            "id": 1
        });
        let result = response.get("result");
        assert!(result.is_none() || result == Some(&serde_json::Value::Null));
    }
}
