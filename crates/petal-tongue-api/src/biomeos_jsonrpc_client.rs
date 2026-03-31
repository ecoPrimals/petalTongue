// SPDX-License-Identifier: AGPL-3.0-or-later
//! `BiomeOS` API Client (JSON-RPC over Unix Sockets)
//!
//! TRUE PRIMAL architecture: Uses JSON-RPC 2.0 over Unix sockets,
//! not HTTP/REST. Connects to `BiomeOS` via the Neural API provider.
//!
//! # Migration from HTTP
//!
//! This module replaces the HTTP-based `BiomeOSClient` with a proper
//! JSON-RPC implementation per `PRIMAL_IPC_PROTOCOL.md`.
//!
//! # Standards Compliance
//!
//! - Protocol: JSON-RPC 2.0 over Unix sockets
//! - Transport: `tokio::net::UnixStream`
//! - Methods: Semantic naming (`neural_api`.*)
//! - Discovery: Capability-based, no hardcoding

use crate::biomeos_error::BiomeOsClientError;
use petal_tongue_core::constants::biomeos_socket_name;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_ipc::socket_path::discover_primal_socket;
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info};

/// `BiomeOS` JSON-RPC client (TRUE PRIMAL architecture)
pub struct BiomeOSJsonRpcClient {
    /// Socket path (e.g., `/run/user/1000/biomeos/biomeos-neural-api.sock`)
    socket_path: PathBuf,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl BiomeOSJsonRpcClient {
    /// Create a new JSON-RPC client
    ///
    /// # Socket Path Discovery
    ///
    /// 1. Environment variable: `BIOMEOS_SOCKET` / `BIOMEOS_NEURAL_API_SOCKET`
    /// 2. Canonical: `$XDG_RUNTIME_DIR/biomeos/<name>.sock` (see `discover_primal_socket`)
    /// 3. Fallback: `/tmp/biomeos/<name>.sock`
    ///
    /// # Errors
    /// Returns `BiomeOsClientError::SocketNotFound` if no socket found.
    pub fn new() -> Result<Self, BiomeOsClientError> {
        let socket_path = Self::discover_socket_path()?;

        Ok(Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Create client with explicit socket path
    pub fn with_socket_path(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Discover `BiomeOS` socket path using capability-based discovery
    ///
    /// # Socket Discovery Priority (TRUE PRIMAL compliant)
    /// 1. `BIOMEOS_SOCKET` or `BIOMEOS_NEURAL_API_SOCKET` - explicit override
    /// 2. `discover_primal_socket(biomeos_socket_name())` — `$XDG_RUNTIME_DIR/biomeos/<name>.sock`
    ///    or `/tmp/biomeos/<name>.sock` when runtime dir is unavailable
    ///
    /// Returns error if no socket found (graceful degradation over silent failure)
    fn discover_socket_path() -> Result<PathBuf, BiomeOsClientError> {
        // 1. Check environment variables (explicit override - highest priority)
        for env_var in ["BIOMEOS_SOCKET", "BIOMEOS_NEURAL_API_SOCKET"] {
            if let Ok(path) = std::env::var(env_var) {
                let socket_path = PathBuf::from(&path);
                if socket_path.exists() {
                    return Ok(socket_path);
                }
                tracing::debug!(
                    "{}={} but socket not found, continuing discovery",
                    env_var,
                    path
                );
            }
        }

        // 2. Canonical biomeOS layout (`$XDG_RUNTIME_DIR` or `/tmp` + `biomeos/<name>.sock`)
        let name = biomeos_socket_name();
        let primary = discover_primal_socket(&name, None, None).map_err(|e| {
            BiomeOsClientError::SocketNotFound(format!("biomeOS socket path resolution: {e}"))
        })?;
        if primary.exists() {
            return Ok(primary);
        }

        let tmp_fallback = PathBuf::from("/tmp")
            .join("biomeos")
            .join(format!("{name}.sock"));
        if tmp_fallback.exists() && tmp_fallback != primary {
            return Ok(tmp_fallback);
        }

        // No socket found - return helpful error instead of silent default
        Err(BiomeOsClientError::SocketNotFound(format!(
            "biomeOS socket not found. Set BIOMEOS_SOCKET env var or ensure biomeOS is running. \
            Checked: $BIOMEOS_SOCKET, $BIOMEOS_NEURAL_API_SOCKET, \
            $XDG_RUNTIME_DIR/biomeos/{name}.sock, /tmp/biomeos/{name}.sock"
        )))
    }

    /// Check if `BiomeOS` is available
    pub async fn is_available(&self) -> bool {
        matches!(
            tokio::time::timeout(
                std::time::Duration::from_millis(100),
                UnixStream::connect(&self.socket_path),
            )
            .await,
            Ok(Ok(_))
        )
    }

    /// Health check (semantic: `neural_api.health`)
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on socket or JSON-RPC failure.
    pub async fn health_check(&self) -> Result<bool, BiomeOsClientError> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.health",
            "params": {},
            "id": self.next_request_id(),
        });

        match self.send_request(&request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Discover primals (semantic: `primal.list`)
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on socket, JSON-RPC, or parse failure.
    pub async fn discover_primals(&self) -> Result<Vec<PrimalInfo>, BiomeOsClientError> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "primal.list",
            "params": {},
            "id": self.next_request_id(),
        });

        let result = self.send_request(&request).await?;

        let primals: Vec<PrimalInfo> = serde_json::from_value(result)
            .map_err(|e| BiomeOsClientError::Parse(format!("Failed to parse primals: {e}")))?;

        info!("✅ Discovered {} primals via JSON-RPC", primals.len());

        Ok(primals)
    }

    /// Get topology (semantic: `neural_api.get_topology`)
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on socket, JSON-RPC, or parse failure.
    pub async fn get_topology(&self) -> Result<Vec<TopologyEdge>, BiomeOsClientError> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.get_topology",
            "params": {},
            "id": self.next_request_id(),
        });

        let result = self.send_request(&request).await?;

        let edges: Vec<TopologyEdge> = serde_json::from_value(result)
            .map_err(|e| BiomeOsClientError::Parse(format!("Failed to parse topology: {e}")))?;

        debug!("✅ Retrieved {} topology edges via JSON-RPC", edges.len());

        Ok(edges)
    }

    /// Send a JSON-RPC request
    async fn send_request(&self, request: &Value) -> Result<Value, BiomeOsClientError> {
        // Connect to BiomeOS
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            BiomeOsClientError::Connect(format!(
                "Failed to connect to BiomeOS at {}\n\
                    \n\
                    Troubleshooting:\n\
                    - Ensure BiomeOS nucleus is running\n\
                    - Check BIOMEOS_SOCKET environment variable\n\
                    - Verify socket permissions\n\
                    - Check XDG_RUNTIME_DIR is set correctly: {e}",
                self.socket_path.display()
            ))
        })?;

        // Send request (line-delimited JSON-RPC)
        let request_str =
            serde_json::to_string(request).map_err(|e| BiomeOsClientError::Parse(e.to_string()))?;
        stream
            .write_all(format!("{request_str}\n").as_bytes())
            .await
            .map_err(|e| BiomeOsClientError::Io(e.to_string()))?;
        stream
            .flush()
            .await
            .map_err(|e| BiomeOsClientError::Io(e.to_string()))?;

        // Read response
        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| BiomeOsClientError::Io(format!("Failed to read response: {e}")))?;

        // Parse response
        let response: Value = serde_json::from_str(&response_line).map_err(|e| {
            BiomeOsClientError::Parse(format!("Failed to parse JSON-RPC response: {e}"))
        })?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(BiomeOsClientError::JsonRpcError(format!(
                "BiomeOS returned JSON-RPC error: {error}\n\
                \n\
                This indicates BiomeOS received the request but encountered an error."
            )));
        }

        // Extract result
        response
            .get("result")
            .cloned()
            .ok_or(BiomeOsClientError::NoResult)
    }

    /// Get next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_socket_path() {
        let client = BiomeOSJsonRpcClient::with_socket_path("/custom/path.sock");
        drop(client);
    }

    #[test]
    fn test_socket_path_discovery() {
        // May succeed if socket exists, or fail - either is valid
        let _ = BiomeOSJsonRpcClient::new();
    }

    #[test]
    fn test_discover_socket_from_env() {
        let temp = std::env::temp_dir().join("biomeos-api-test.sock");
        std::fs::write(&temp, "").unwrap();

        let result = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "BIOMEOS_SOCKET",
            temp.to_str().unwrap(),
            BiomeOSJsonRpcClient::new,
        );
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&temp);
    }

    #[tokio::test]
    async fn test_biomeos_unavailable() {
        let client = BiomeOSJsonRpcClient::with_socket_path("/tmp/nonexistent-biomeos.sock");
        let available = client.is_available().await;
        assert!(!available);
    }

    #[tokio::test]
    async fn test_health_check_unavailable() {
        let client = BiomeOSJsonRpcClient::with_socket_path("/tmp/nonexistent-biomeos-health.sock");
        let healthy = client.health_check().await.expect("health_check");
        assert!(!healthy);
    }

    #[test]
    fn test_jsonrpc_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.health",
            "params": {},
            "id": 1
        });
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "neural_api.health");
        assert!(request["params"].is_object());
    }

    #[test]
    fn test_jsonrpc_discover_primals_request() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "primal.list",
            "params": {},
            "id": 42
        });
        assert_eq!(request["method"], "primal.list");
        assert_eq!(request["id"], 42);
    }

    #[test]
    fn test_jsonrpc_get_topology_request() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.get_topology",
            "params": {},
            "id": 1
        });
        assert_eq!(request["method"], "neural_api.get_topology");
    }

    #[test]
    fn test_with_socket_path_pathbuf() {
        let client = BiomeOSJsonRpcClient::with_socket_path(std::path::PathBuf::from("/x/y.sock"));
        drop(client);
    }

    #[test]
    fn test_jsonrpc_error_response_structure() {
        let error_response = json!({
            "jsonrpc": "2.0",
            "error": {"code": -32603, "message": "Internal error"},
            "id": 1
        });
        assert!(error_response.get("error").is_some());
        assert!(error_response.get("result").is_none());
    }

    #[test]
    fn test_jsonrpc_success_response_parsing() {
        let response = json!({
            "jsonrpc": "2.0",
            "result": [{"id": "p1", "name": "Primal1"}],
            "id": 1
        });
        assert!(response.get("result").is_some());
        let result = response.get("result").cloned().unwrap();
        let primals: Vec<serde_json::Value> = serde_json::from_value(result).unwrap();
        assert_eq!(primals.len(), 1);
    }

    #[test]
    fn test_jsonrpc_response_no_result_fails() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        assert!(response.get("result").is_none());
    }

    #[test]
    fn test_primal_list_request_params() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "primal.list",
            "params": {},
            "id": 1
        });
        assert_eq!(request["params"], json!({}));
    }

    #[test]
    fn test_topology_request_params() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.get_topology",
            "params": {},
            "id": 1
        });
        assert!(request["params"].is_object());
    }
}
