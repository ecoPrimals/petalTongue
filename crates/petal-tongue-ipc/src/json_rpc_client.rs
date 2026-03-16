// SPDX-License-Identifier: AGPL-3.0-only
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
//! let client = JsonRpcClient::new("/tmp/petaltongue-nat0-default.sock")?;
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

        let mut request_bytes = request_json.into_bytes();
        request_bytes.push(b'\n');
        let request_bytes: Bytes = Bytes::from(request_bytes);
        timeout(self.timeout, writer.write_all(&request_bytes))
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
        let mut request_bytes = request_json.into_bytes();
        request_bytes.push(b'\n');
        let request_bytes: Bytes = Bytes::from(request_bytes);
        timeout(self.timeout, writer.write_all(&request_bytes))
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
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
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
        let client = JsonRpcClient::with_timeout("/tmp/test.sock", Duration::from_secs(10))
            .expect("valid path");
        assert_eq!(client.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_debug_impl() {
        let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("JsonRpcClient"));
        assert!(debug_str.contains("/tmp/test.sock"));
    }

    #[tokio::test]
    async fn test_call_nonexistent_socket() {
        let client =
            JsonRpcClient::new("/tmp/nonexistent-jsonrpc-test-12345.sock").expect("valid path");
        let result = client.call("health.check", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_notify_nonexistent_socket() {
        let client =
            JsonRpcClient::new("/tmp/nonexistent-jsonrpc-notify-12345.sock").expect("valid path");
        let result = client.notify("some.method", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_topology_data_structure() {
        let data = TopologyData {
            nodes: vec![serde_json::json!({"id": "n1"})],
            edges: vec![serde_json::json!({"from": "n1", "to": "n2"})],
        };
        let json = serde_json::to_value(&data).expect("serialize");
        assert_eq!(json["nodes"].as_array().expect("nodes").len(), 1);
        assert_eq!(json["edges"].as_array().expect("edges").len(), 1);
    }

    #[test]
    fn test_topology_data_empty() {
        let data = TopologyData {
            nodes: vec![],
            edges: vec![],
        };
        let json = serde_json::to_value(&data).expect("serialize");
        assert!(json["nodes"].as_array().expect("nodes").is_empty());
    }

    #[test]
    fn test_client_clone() {
        let client = JsonRpcClient::new("/tmp/clone-test.sock").expect("valid path");
        let cloned = client.clone();
        assert_eq!(client.socket_path(), cloned.socket_path());
    }

    #[test]
    fn test_json_rpc_client_error_display() {
        let err = JsonRpcClientError::Connection("test".to_string());
        let s = format!("{err}");
        assert!(s.contains("Connection"));
        assert!(s.contains("test"));
    }

    #[test]
    fn test_json_rpc_client_rpc_error() {
        let err = JsonRpcClientError::RpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };
        let s = format!("{err}");
        assert!(s.contains("-32601"));
        assert!(s.contains("Method not found"));
    }

    #[test]
    fn test_json_rpc_client_empty_path_error() {
        let result = JsonRpcClient::new("");
        assert!(result.is_err());
        if let Err(JsonRpcClientError::Connection(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected Connection error");
        }
    }

    #[test]
    fn test_json_rpc_request_format() {
        use crate::json_rpc::JsonRpcRequest;
        let req = JsonRpcRequest::new(
            "test.method",
            serde_json::json!({"a": 1}),
            serde_json::json!(42),
        );
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test.method");
        assert_eq!(req.id, serde_json::json!(42));
    }

    #[test]
    fn test_next_id_increments() {
        let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
        let id1 = client.next_id();
        let id2 = client.next_id();
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_extract_result_no_result() {
        use crate::json_rpc::JsonRpcResponse;
        let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: serde_json::json!(1),
        };
        let result = client.extract_result(resp, 1);
        assert!(result.is_err());
        if let Err(JsonRpcClientError::InvalidResponse(msg)) = result {
            assert!(msg.contains("no result"));
        } else {
            panic!("Expected InvalidResponse");
        }
    }

    #[test]
    fn test_extract_result_success() {
        use crate::json_rpc::JsonRpcResponse;
        let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: serde_json::json!(1),
        };
        let result = client.extract_result(resp, 1);
        assert!(result.is_ok());
        let val = result.expect("ok");
        assert_eq!(val["status"], "ok");
    }

    #[test]
    fn test_json_rpc_client_rpc_error_with_data() {
        let err = JsonRpcClientError::RpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(serde_json::json!({"param": "id"})),
        };
        let s = format!("{err}");
        assert!(s.contains("-32602"));
        assert!(s.contains("Invalid params"));
    }

    #[tokio::test]
    async fn test_batch_nonexistent_socket() {
        let client = JsonRpcClient::new("/tmp/nonexistent-batch-99999.sock").expect("valid path");
        let requests = vec![
            ("health.check", serde_json::json!({})),
            ("topology.get", serde_json::json!({})),
        ];
        let result = client.batch(requests).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_json_rpc_response_error_extraction() {
        use crate::json_rpc::JsonRpcResponse;
        let resp = JsonRpcResponse::error(
            serde_json::json!(1),
            crate::json_rpc::error_codes::METHOD_NOT_FOUND,
            "Method not found",
        );
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_request_serialization_roundtrip() {
        use crate::json_rpc::JsonRpcRequest;
        let req = JsonRpcRequest::new(
            "topology.get",
            serde_json::json!({"filter": "nodes"}),
            serde_json::json!(42),
        );
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.method, "topology.get");
        assert_eq!(parsed.params["filter"], "nodes");
        assert_eq!(parsed.id, serde_json::json!(42));
    }

    #[test]
    fn test_response_success_deserialization() {
        use crate::json_rpc::JsonRpcResponse;
        let json = r#"{"jsonrpc":"2.0","result":{"nodes":[],"edges":[]},"id":1}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert!(resp.result.as_ref().expect("result")["nodes"].is_array());
    }

    #[test]
    fn test_response_error_deserialization() {
        use crate::json_rpc::JsonRpcResponse;
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.result.is_none());
        let err = resp.error.as_ref().expect("error");
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
    }

    #[test]
    fn test_method_name_formatting() {
        use crate::json_rpc::JsonRpcRequest;
        let req = JsonRpcRequest::new(
            "capability.list",
            serde_json::json!({}),
            serde_json::json!(1),
        );
        assert_eq!(req.method, "capability.list");
        let req2 = JsonRpcRequest::new("health.check", serde_json::json!({}), serde_json::json!(2));
        assert_eq!(req2.method, "health.check");
    }

    #[test]
    fn test_request_includes_id() {
        use crate::json_rpc::JsonRpcRequest;
        let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(99));
        assert_eq!(req.id, serde_json::json!(99));
        let req_null = JsonRpcRequest::new(
            "notify.method",
            serde_json::json!({}),
            serde_json::Value::Null,
        );
        assert!(req_null.id.is_null());
    }

    #[test]
    fn test_json_rpc_client_error_timeout() {
        let err = JsonRpcClientError::Timeout("Connection timeout".to_string());
        let s = format!("{err}");
        assert!(s.contains("Timeout"));
        assert!(s.contains("Connection timeout"));
    }

    #[test]
    fn test_json_rpc_client_error_serialization() {
        let err = JsonRpcClientError::Serialization("JSON parse failed".to_string());
        let s = format!("{err}");
        assert!(s.contains("Serialization"));
        assert!(s.contains("JSON parse failed"));
    }

    #[test]
    fn test_json_rpc_client_error_invalid_response() {
        let err = JsonRpcClientError::InvalidResponse("Malformed JSON".to_string());
        let s = format!("{err}");
        assert!(s.contains("Invalid response"));
        assert!(s.contains("Malformed JSON"));
    }

    #[test]
    fn test_json_rpc_client_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = JsonRpcClientError::Io(io_err);
        let s = format!("{err}");
        assert!(s.contains("refused") || s.contains("I/O"));
    }

    #[test]
    fn test_batch_request_structure() {
        let requests = [
            ("health.check", serde_json::json!({})),
            ("topology.get", serde_json::json!({})),
        ];
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].0, "health.check");
        assert_eq!(requests[1].0, "topology.get");
    }

    #[test]
    fn test_topology_data_serialization() {
        let data = TopologyData {
            nodes: vec![
                serde_json::json!({"id": "n1", "x": 0.0}),
                serde_json::json!({"id": "n2", "x": 1.0}),
            ],
            edges: vec![serde_json::json!({"from": "n1", "to": "n2"})],
        };
        let json = serde_json::to_value(&data).expect("serialize");
        assert_eq!(json["nodes"].as_array().expect("nodes").len(), 2);
        assert_eq!(json["edges"].as_array().expect("edges").len(), 1);
    }

    #[tokio::test]
    async fn test_send_request_invalid_json_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("invalid-json.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut reader, mut writer) = stream.into_split();
            let mut buf = [0u8; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
            writer.write_all(b"{not valid json\n").await.expect("write");
            writer.flush().await.expect("flush");
        });
        let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500))
            .expect("client");
        let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
        let result = client.send_request(&req).await;
        assert!(result.is_err());
        if let Err(JsonRpcClientError::InvalidResponse(msg)) = result {
            assert!(msg.contains("Invalid JSON") || msg.contains("invalid"));
        } else {
            panic!("Expected InvalidResponse");
        }
    }

    #[tokio::test]
    async fn test_send_request_rpc_error_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("rpc-error.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut reader, mut writer) = stream.into_split();
            let mut buf = [0u8; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
            let err_resp =
                r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
            writer
                .write_all(format!("{err_resp}\n").as_bytes())
                .await
                .expect("write");
            writer.flush().await.expect("flush");
        });
        let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500))
            .expect("client");
        let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
        let result = client.send_request(&req).await;
        assert!(result.is_err());
        if let Err(JsonRpcClientError::RpcError { code, message, .. }) = result {
            assert_eq!(code, -32601);
            assert!(message.contains("Method not found"));
        } else {
            panic!("Expected RpcError");
        }
    }

    #[tokio::test]
    async fn test_send_request_empty_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("empty.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut reader, mut writer) = stream.into_split();
            let mut buf = [0u8; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
            writer.write_all(b"\n").await.expect("write");
            writer.flush().await.expect("flush");
        });
        let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500))
            .expect("client");
        let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
        let result = client.send_request(&req).await;
        assert!(result.is_err());
        assert!(result.is_err());
        let err_str = format!("{}", result.unwrap_err());
        assert!(
            err_str.contains("Empty") || err_str.contains("empty") || err_str.contains("Invalid"),
            "expected empty/invalid response error: {err_str}"
        );
    }

    #[tokio::test]
    async fn test_call_success_via_mock_server() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("success.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = stream.into_split();
            let mut line = String::new();
            tokio::io::AsyncBufReadExt::read_line(
                &mut tokio::io::BufReader::new(reader),
                &mut line,
            )
            .await
            .expect("read");
            let resp = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
            writer
                .write_all(format!("{resp}\n").as_bytes())
                .await
                .expect("write");
            writer.flush().await.expect("flush");
        });
        let client = JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500))
            .expect("client");
        let result = client.call("test.method", serde_json::json!({})).await;
        assert!(result.is_ok());
        let val = result.expect("ok");
        assert_eq!(val["ok"], true);
    }

    #[tokio::test]
    async fn test_connection_timeout_to_nonexistent() {
        let client = JsonRpcClient::with_timeout(
            "/tmp/nonexistent-timeout-test-99999.sock",
            Duration::from_millis(10),
        )
        .expect("client");
        let result = client.call("health.check", serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = format!("{err}");
        assert!(
            err_str.contains("Timeout")
                || err_str.contains("Connection")
                || err_str.contains("Failed"),
            "expected timeout or connection error, got: {err_str}"
        );
    }

    #[test]
    fn test_topology_data_deserialization_roundtrip() {
        let json = serde_json::json!({
            "nodes": [{"id": "n1", "x": 0.0}, {"id": "n2", "x": 1.0}],
            "edges": [{"from": "n1", "to": "n2"}]
        });
        let data: TopologyData = serde_json::from_value(json).expect("deserialize");
        let serialized = serde_json::to_value(&data).expect("serialize");
        assert_eq!(serialized["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(serialized["edges"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_request_notification_null_id() {
        use crate::json_rpc::JsonRpcRequest;
        let req = JsonRpcRequest::new(
            "notify.method",
            serde_json::json!({}),
            serde_json::Value::Null,
        );
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("null"));
        assert!(req.id.is_null());
    }

    #[test]
    fn test_primal_info_deserialization_from_discover_format() {
        let json = serde_json::json!([{
            "id": "p1",
            "name": "petaltongue",
            "primal_type": "petaltongue",
            "endpoint": "/primal/petaltongue",
            "capabilities": ["ui.render", "graph.topology"],
            "health": "Healthy",
            "last_seen": 1_234_567_890
        }]);
        let primals: Vec<petal_tongue_core::PrimalInfo> =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].id.as_str(), "p1");
        assert_eq!(primals[0].name, "petaltongue");
        assert!(matches!(
            primals[0].health,
            petal_tongue_core::PrimalHealthStatus::Healthy
        ));
    }
}
