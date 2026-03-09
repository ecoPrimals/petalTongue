// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC 2.0 client for petalTongue IPC
//!
//! Per wateringHole UNIVERSAL_IPC_STANDARD_V3.md, JSON-RPC 2.0 is the PRIMARY protocol
//! for local IPC. This client connects to Unix domain sockets and communicates
//! using the JSON-RPC 2.0 specification.
//!
//! # Features
//!
//! - Connects to Unix domain sockets (primary transport)
//! - JSON-RPC 2.0 request/response protocol
//! - Semantic method naming: `{domain}.{operation}`
//! - Timeout configuration
//! - Zero-copy where possible (bytes::Bytes for payloads)
//! - Async (tokio-based)
//! - Proper error handling (no panics)
//!
//! # Example
//!
//! ```no_run
//! use petal_tongue_ipc::JsonRpcClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = JsonRpcClient::new("/tmp/petaltongue-nat0-default.sock")?;
//! let health = client.call("health_check", serde_json::json!({})).await?;
//! println!("Health: {:?}", health);
//! # Ok(())
//! # }
//! ```

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use bytes::Bytes;
use petal_tongue_core::PrimalInfo;
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;
use tracing::trace;

/// Error type for JSON-RPC client operations
#[derive(Debug, Error)]
pub enum JsonRpcClientError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    Connection(String),

    /// JSON-RPC protocol error
    #[error("JSON-RPC error (code {code}): {message}")]
    RpcError {
        /// JSON-RPC error code
        code: i32,
        /// Human-readable error message
        message: String,
        /// Optional additional error data
        data: Option<Value>,
    },

    /// Serialization/deserialization failed
    #[error("Serialization failed: {0}")]
    Serialization(String),

    /// Timeout exceeded
    #[error("Timeout: {0}")]
    Timeout(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Result type for JSON-RPC client operations
pub type JsonRpcResult<T> = Result<T, JsonRpcClientError>;

/// Topology data returned by get_topology
///
/// Matches the format returned by the Unix socket server's get_topology method.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopologyData {
    /// Graph nodes with position info
    pub nodes: Vec<Value>,
    /// Graph edges
    pub edges: Vec<Value>,
}

/// JSON-RPC 2.0 client for Unix domain socket IPC
///
/// Connects to petalTongue or other primals via Unix sockets using the
/// JSON-RPC 2.0 protocol. Supports semantic method naming per ecosystem standards.
#[derive(Clone)]
pub struct JsonRpcClient {
    /// Socket path
    socket_path: std::path::PathBuf,
    /// Request timeout
    timeout: Duration,
    /// Request ID counter
    request_id: std::sync::Arc<AtomicU64>,
}

impl JsonRpcClient {
    /// Create a new JSON-RPC client
    ///
    /// # Arguments
    /// * `socket_path` - Path to the Unix domain socket (e.g., `/tmp/petaltongue-nat0-default.sock`)
    ///
    /// # Errors
    /// Returns error if socket path is invalid
    pub fn new(socket_path: impl AsRef<Path>) -> JsonRpcResult<Self> {
        let path = socket_path.as_ref().to_path_buf();
        if path.as_os_str().is_empty() {
            return Err(JsonRpcClientError::Connection(
                "Socket path cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            socket_path: path,
            timeout: Duration::from_secs(5),
            request_id: std::sync::Arc::new(AtomicU64::new(1)),
        })
    }

    /// Create client with custom timeout
    pub fn with_timeout(
        socket_path: impl AsRef<Path>,
        timeout_duration: Duration,
    ) -> JsonRpcResult<Self> {
        let mut client = Self::new(socket_path)?;
        client.timeout = timeout_duration;
        Ok(client)
    }

    /// Get the socket path
    #[must_use]
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }

    /// Get the configured timeout
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Send a JSON-RPC call and receive response
    ///
    /// # Arguments
    /// * `method` - Method name (e.g., "health_check", "get_topology")
    /// * `params` - Method parameters as JSON value
    ///
    /// # Returns
    /// The result field from the JSON-RPC response
    pub async fn call(&self, method: &str, params: Value) -> JsonRpcResult<Value> {
        let id = self.next_id();
        let request = JsonRpcRequest::new(method, params, Value::Number(id.into()));

        let response = self.send_request(&request).await?;
        self.extract_result(response, id)
    }

    /// Send a JSON-RPC notification (no response expected)
    ///
    /// # Arguments
    /// * `method` - Method name
    /// * `params` - Method parameters as JSON value
    pub async fn notify(&self, method: &str, params: Value) -> JsonRpcResult<()> {
        let request = JsonRpcRequest::new(method, params, Value::Null);
        self.send_request_no_response(&request).await
    }

    /// Send a batch of requests
    ///
    /// # Arguments
    /// * `requests` - Vector of (method, params) pairs
    ///
    /// # Returns
    /// Vector of responses (one per request, in order)
    pub async fn batch(&self, requests: Vec<(&str, Value)>) -> JsonRpcResult<Vec<JsonRpcResponse>> {
        let mut responses = Vec::with_capacity(requests.len());
        for (method, params) in requests {
            let id = self.next_id();
            let request = JsonRpcRequest::new(method, params, Value::Number(id.into()));
            let response = self.send_request(&request).await?;
            responses.push(response);
        }
        Ok(responses)
    }

    /// Discover primals (semantic: discovery.primals)
    ///
    /// Calls the discovery service to get a list of registered primals.
    /// Returns METHOD_NOT_FOUND if the server doesn't support this method
    /// (e.g., when connecting to petalTongue's own socket).
    pub async fn discover_primals(&self) -> JsonRpcResult<Vec<PrimalInfo>> {
        let result = match self.call("discovery.primals", serde_json::json!({})).await {
            Ok(r) => r,
            Err(_) => {
                self.call("neural_api.get_primals", serde_json::json!({}))
                    .await?
            }
        };

        let primals: Vec<PrimalInfo> = serde_json::from_value(result).map_err(|e| {
            JsonRpcClientError::Serialization(format!("Failed to parse primals: {e}"))
        })?;
        Ok(primals)
    }

    /// Get topology (semantic: graph.get_topology)
    ///
    /// Returns the current graph topology from the server.
    pub async fn get_topology(&self) -> JsonRpcResult<TopologyData> {
        let result = match self.call("get_topology", serde_json::json!({})).await {
            Ok(r) => r,
            Err(_) => {
                self.call("neural_api.get_topology", serde_json::json!({}))
                    .await?
            }
        };

        // Parse as TopologyData (flexible format)
        let data: TopologyData = serde_json::from_value(result).map_err(|e| {
            JsonRpcClientError::Serialization(format!("Failed to parse topology: {e}"))
        })?;
        Ok(data)
    }

    /// Health check (semantic: health.check)
    pub async fn health_check(&self) -> JsonRpcResult<Value> {
        match self.call("health_check", serde_json::json!({})).await {
            Ok(r) => Ok(r),
            Err(_) => self.call("get_health", serde_json::json!({})).await,
        }
    }

    /// Get capabilities (semantic: capabilities.announce)
    pub async fn get_capabilities(&self) -> JsonRpcResult<Value> {
        match self
            .call("announce_capabilities", serde_json::json!({}))
            .await
        {
            Ok(r) => Ok(r),
            Err(_) => self.call("get_capabilities", serde_json::json!({})).await,
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn send_request(&self, request: &JsonRpcRequest) -> JsonRpcResult<JsonRpcResponse> {
        let stream = timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| {
                JsonRpcClientError::Timeout(format!(
                    "Connection timeout to {}",
                    self.socket_path.display()
                ))
            })?
            .map_err(|e| {
                JsonRpcClientError::Connection(format!(
                    "Failed to connect to {}: {e}",
                    self.socket_path.display()
                ))
            })?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request_json = serde_json::to_string(request)
            .map_err(|e| JsonRpcClientError::Serialization(format!("Serialize request: {e}")))?;

        let request_bytes: Bytes = Bytes::from(format!("{request_json}\n"));
        timeout(self.timeout, writer.write_all(&request_bytes))
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Write timeout".to_string()))??;
        timeout(self.timeout, writer.flush())
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Flush timeout".to_string()))??;

        let mut line = String::new();
        timeout(self.timeout, reader.read_line(&mut line))
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Read timeout".to_string()))??;

        if line.is_empty() {
            return Err(JsonRpcClientError::InvalidResponse(
                "Empty response from server".to_string(),
            ));
        }

        let response: JsonRpcResponse = serde_json::from_str(&line).map_err(|e| {
            JsonRpcClientError::InvalidResponse(format!("Invalid JSON response: {e}"))
        })?;

        trace!("JSON-RPC response: {:?}", response);

        if let Some(ref err) = response.error {
            return Err(JsonRpcClientError::RpcError {
                code: err.code,
                message: err.message.clone(),
                data: err.data.clone(),
            });
        }

        Ok(response)
    }

    async fn send_request_no_response(&self, request: &JsonRpcRequest) -> JsonRpcResult<()> {
        let stream = timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| {
                JsonRpcClientError::Timeout(format!(
                    "Connection timeout to {}",
                    self.socket_path.display()
                ))
            })?
            .map_err(|e| {
                JsonRpcClientError::Connection(format!(
                    "Failed to connect to {}: {e}",
                    self.socket_path.display()
                ))
            })?;

        let (_reader, mut writer) = stream.into_split();
        let request_json = serde_json::to_string(request)
            .map_err(|e| JsonRpcClientError::Serialization(format!("Serialize request: {e}")))?;
        let request_bytes: Bytes = Bytes::from(format!("{request_json}\n"));
        timeout(self.timeout, writer.write_all(&request_bytes))
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Write timeout".to_string()))??;
        timeout(self.timeout, writer.flush())
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Flush timeout".to_string()))??;
        Ok(())
    }

    fn extract_result(&self, response: JsonRpcResponse, _expected_id: u64) -> JsonRpcResult<Value> {
        response.result.ok_or_else(|| {
            JsonRpcClientError::InvalidResponse("Response has no result field".to_string())
        })
    }
}

impl std::fmt::Debug for JsonRpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonRpcClient")
            .field("socket_path", &self.socket_path)
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = JsonRpcClient::new("/tmp/test.sock").unwrap();
        assert_eq!(client.socket_path(), std::path::Path::new("/tmp/test.sock"));
        assert_eq!(client.timeout(), Duration::from_secs(5));
    }

    #[test]
    fn test_client_creation_empty_path() {
        let result = JsonRpcClient::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_with_timeout() {
        let client =
            JsonRpcClient::with_timeout("/tmp/test.sock", Duration::from_secs(10)).unwrap();
        assert_eq!(client.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_debug_impl() {
        let client = JsonRpcClient::new("/tmp/test.sock").unwrap();
        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("JsonRpcClient"));
        assert!(debug_str.contains("/tmp/test.sock"));
    }

    #[tokio::test]
    async fn test_call_nonexistent_socket() {
        let client = JsonRpcClient::new("/tmp/nonexistent-jsonrpc-test-12345.sock").unwrap();
        let result = client.call("health_check", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_notify_nonexistent_socket() {
        let client = JsonRpcClient::new("/tmp/nonexistent-jsonrpc-notify-12345.sock").unwrap();
        let result = client.notify("some.method", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_topology_data_structure() {
        let data = TopologyData {
            nodes: vec![serde_json::json!({"id": "n1"})],
            edges: vec![serde_json::json!({"from": "n1", "to": "n2"})],
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["nodes"].as_array().unwrap().len(), 1);
        assert_eq!(json["edges"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_topology_data_empty() {
        let data = TopologyData {
            nodes: vec![],
            edges: vec![],
        };
        let json = serde_json::to_value(&data).unwrap();
        assert!(json["nodes"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_client_clone() {
        let client = JsonRpcClient::new("/tmp/clone-test.sock").unwrap();
        let cloned = client.clone();
        assert_eq!(client.socket_path(), cloned.socket_path());
    }
}
