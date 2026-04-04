// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 client for petalTongue IPC
//!
//! Per wateringHole `UNIVERSAL_IPC_STANDARD_V3.md`, JSON-RPC 2.0 is the PRIMARY protocol
//! for local IPC. This client connects to Unix domain sockets and communicates
//! using the JSON-RPC 2.0 specification.
//!
//! # Features
//!
//! - Connects to Unix domain sockets (primary transport)
//! - JSON-RPC 2.0 request/response protocol
//! - Semantic method naming: `{domain}.{operation}`
//! - Timeout configuration
//! - Zero-copy where possible (`bytes::Bytes` for payloads)
//! - Async (tokio-based)
//! - Proper error handling (no panics)
//!
//! # Example
//!
//! ```no_run
//! use petal_tongue_ipc::JsonRpcClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = JsonRpcClient::new("/tmp/biomeos/petaltongue.sock")?;
//! let health = client.call("health.check", serde_json::json!({})).await?;
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

/// Topology data returned by `get_topology`
///
/// Matches the format returned by the Unix socket server's `get_topology` method.
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
    /// * `socket_path` - Path to the Unix domain socket (e.g., `/tmp/biomeos/petaltongue.sock`)
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
            timeout: petal_tongue_core::constants::default_rpc_timeout(),
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
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Send a JSON-RPC call and receive response
    ///
    /// # Arguments
    /// * `method` - Method name (e.g., "health.check", "topology.get")
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
    /// Returns `METHOD_NOT_FOUND` if the server doesn't support this method
    /// (e.g., when connecting to petalTongue's own socket).
    pub async fn discover_primals(&self) -> JsonRpcResult<Vec<PrimalInfo>> {
        let result = match self.call("discovery.primals", serde_json::json!({})).await {
            Ok(r) => r,
            Err(_) => self.call("primal.list", serde_json::json!({})).await?,
        };

        let primals: Vec<PrimalInfo> = serde_json::from_value(result).map_err(|e| {
            JsonRpcClientError::Serialization(format!("Failed to parse primals: {e}"))
        })?;
        Ok(primals)
    }

    /// Get topology (semantic: topology.get)
    ///
    /// Returns the current graph topology from the server.
    pub async fn get_topology(&self) -> JsonRpcResult<TopologyData> {
        let result = match self.call("topology.get", serde_json::json!({})).await {
            Ok(r) => r,
            Err(_) => match self.call("get_topology", serde_json::json!({})).await {
                Ok(r) => r,
                Err(_) => {
                    self.call("neural_api.get_topology", serde_json::json!({}))
                        .await?
                }
            },
        };

        // Parse as TopologyData (flexible format)
        let data: TopologyData = serde_json::from_value(result).map_err(|e| {
            JsonRpcClientError::Serialization(format!("Failed to parse topology: {e}"))
        })?;
        Ok(data)
    }

    /// Health check (semantic: health.check)
    pub async fn health_check(&self) -> JsonRpcResult<Value> {
        match self.call("health.check", serde_json::json!({})).await {
            Ok(r) => Ok(r),
            Err(_) => match self.call("health_check", serde_json::json!({})).await {
                Ok(r) => Ok(r),
                Err(_) => self.call("health.get", serde_json::json!({})).await,
            },
        }
    }

    /// Get capabilities (semantic: capability.list)
    pub async fn get_capabilities(&self) -> JsonRpcResult<Value> {
        match self.call("capability.list", serde_json::json!({})).await {
            Ok(r) => Ok(r),
            Err(_) => match self
                .call("capability.announce", serde_json::json!({}))
                .await
            {
                Ok(r) => Ok(r),
                Err(_) => match self.call("get_capabilities", serde_json::json!({})).await {
                    Ok(r) => Ok(r),
                    Err(_) => {
                        self.call("announce_capabilities", serde_json::json!({}))
                            .await
                    }
                },
            },
        }
    }

    /// Returns the next request ID. Exposed for testing ID monotonicity.
    #[cfg_attr(not(test), doc(hidden))]
    pub fn next_id(&self) -> u64 {
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

        let mut request_bytes = serde_json::to_vec(request)
            .map_err(|e| JsonRpcClientError::Serialization(format!("Serialize request: {e}")))?;
        request_bytes.push(b'\n');

        let wire_bytes: Bytes = Bytes::from(request_bytes);
        timeout(self.timeout, writer.write_all(&wire_bytes))
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Write timeout".to_string()))??;
        timeout(self.timeout, writer.flush())
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Flush timeout".to_string()))??;

        let mut line = Vec::new();
        timeout(self.timeout, reader.read_until(b'\n', &mut line))
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Read timeout".to_string()))??;

        if line.is_empty() {
            return Err(JsonRpcClientError::InvalidResponse(
                "Empty response from server".to_string(),
            ));
        }

        let response: JsonRpcResponse = serde_json::from_slice(&line).map_err(|e| {
            JsonRpcClientError::InvalidResponse(format!("Invalid JSON response: {e}"))
        })?;

        trace!("JSON-RPC response: {:?}", response);

        if let Some(err) = response.error {
            return Err(JsonRpcClientError::RpcError {
                code: err.code,
                message: err.message,
                data: err.data,
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
        let mut request_bytes = serde_json::to_vec(request)
            .map_err(|e| JsonRpcClientError::Serialization(format!("Serialize request: {e}")))?;
        request_bytes.push(b'\n');

        let wire_bytes: Bytes = Bytes::from(request_bytes);
        timeout(self.timeout, writer.write_all(&wire_bytes))
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Write timeout".to_string()))??;
        timeout(self.timeout, writer.flush())
            .await
            .map_err(|_| JsonRpcClientError::Timeout("Flush timeout".to_string()))??;
        Ok(())
    }

    #[expect(
        clippy::unused_self,
        reason = "trait method; self required for dispatch"
    )]
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
            .field("request_id", &self.request_id)
            .finish()
    }
}

#[cfg(test)]
#[path = "json_rpc_client_tests.rs"]
mod tests;
