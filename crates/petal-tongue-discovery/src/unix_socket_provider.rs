//! Unix socket discovery provider
//!
//! Discovers primals via Unix domain sockets by scanning for .sock files
//! and querying their capabilities via JSON-RPC.
//!
//! MODERN IDIOMATIC RUST:
//! - Fully async/await (no blocking std::fs operations)
//! - Concurrent socket probing for performance
//! - Non-blocking directory traversal
//! - Aggressive timeouts to prevent hanging on unresponsive sockets

use anyhow::Result;
use futures::future::join_all;
use petal_tongue_core::types::{PrimalHealthStatus, PrimalInfo};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
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
    /// 1. $XDG_RUNTIME_DIR (e.g., /run/user/1000) - biomeOS convention
    /// 2. /tmp - fallback for development
    /// 3. /var/run/ecoPrimals - alternative runtime directory
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Priority 1: XDG_RUNTIME_DIR (biomeOS convention)
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            search_paths.push(PathBuf::from(xdg_runtime));
        }

        // Priority 2: /run/user/<uid> (fallback if XDG not set)
        // SAFETY: getuid() is a safe FFI call that returns the effective user ID
        // without preconditions that could lead to undefined behavior.
        let uid = unsafe { libc::getuid() };
        search_paths.push(PathBuf::from(format!("/run/user/{}", uid)));

        // Priority 3: /tmp (development fallback)
        search_paths.push(PathBuf::from("/tmp"));

        // Priority 4: /var/run/ecoPrimals (alternative)
        search_paths.push(PathBuf::from("/var/run/ecoPrimals"));

        Self { search_paths }
    }

    /// Discover all primals via Unix sockets
    pub async fn discover(&self) -> Result<Vec<PrimalInfo>> {
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

            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                let path = entry.path();

                // Look for .sock files (Unix domain sockets)
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    // Clone self for the async task
                    let provider = self.clone();
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
    async fn probe_socket(&self, path: &Path) -> Result<PrimalInfo> {
        // CRITICAL: Wrap socket connect in timeout to prevent hanging
        // Unresponsive sockets are the #1 cause of test hangs
        let connect_timeout = Duration::from_millis(100);

        let stream = match tokio::time::timeout(connect_timeout, UnixStream::connect(path)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                debug!("Socket connection failed for {}: {}", path.display(), e);
                return Err(e.into());
            }
            Err(_) => {
                debug!("Socket connection timeout for {}", path.display());
                return Err(anyhow::anyhow!("Connection timeout"));
            }
        };

        // Send get_capabilities JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "get_capabilities",
            "params": {},
            "id": 1
        });

        // CRITICAL: Wrap entire request/response in timeout
        let rpc_timeout = Duration::from_millis(200);

        let response =
            match tokio::time::timeout(rpc_timeout, self.send_request(stream, request)).await {
                Ok(Ok(response)) => response,
                Ok(Err(e)) => {
                    debug!("RPC request failed for {}: {}", path.display(), e);
                    return Err(e);
                }
                Err(_) => {
                    debug!("RPC request timeout for {}", path.display());
                    return Err(anyhow::anyhow!("RPC timeout"));
                }
            };

        // Parse response into PrimalInfo
        self.parse_capabilities_response(path, response)
    }

    /// Send a JSON-RPC request and receive response
    async fn send_request(&self, stream: UnixStream, request: Value) -> Result<Value> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request
        let request_json = serde_json::to_string(&request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&line)?;

        if let Some(error) = response.error {
            anyhow::bail!("JSON-RPC error: {} (code: {})", error.message, error.code);
        }

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in response"))
    }

    /// Parse capabilities response into PrimalInfo
    fn parse_capabilities_response(&self, path: &Path, result: Value) -> Result<PrimalInfo> {
        let capabilities: Vec<String> = result["capabilities"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect();

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
            node_id.clone(),
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
        //   "beardog-nat0.sock" → "beardog"
        //   "songbird-family1.sock" → "songbird"
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
}
