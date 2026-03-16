// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 protocol types
//!
//! Implements the JSON-RPC 2.0 specification for inter-primal communication.
//! Follows the standard defined at: <https://www.jsonrpc.org/specification>

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Serde helper for base64-encoded Bytes
mod base64_bytes {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &Bytes, s: S) -> Result<S::Ok, S::Error> {
        let b64 = STANDARD.encode(v);
        b64.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
        let s = String::deserialize(d)?;
        let buf = STANDARD.decode(&s).map_err(serde::de::Error::custom)?;
        Ok(Bytes::from(buf))
    }
}

/// Binary payload for IPC (zero-copy with Bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPayload {
    /// Raw binary data (base64-encoded in JSON)
    #[serde(with = "base64_bytes")]
    pub data: Bytes,
    /// MIME type or content descriptor
    pub content_type: String,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (always "2.0")
    pub jsonrpc: String,

    /// Method name to invoke
    pub method: String,

    /// Method parameters (structured value or array)
    #[serde(default)]
    pub params: Value,

    /// Request identifier (can be string, number, or null)
    pub id: Value,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: impl Into<String>, params: Value, id: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

/// JSON-RPC 2.0 Response (success)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (always "2.0")
    pub jsonrpc: String,

    /// Result data (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error data (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request identifier (matches the request)
    pub id: Value,
}

impl JsonRpcResponse {
    /// Create a successful response
    #[must_use]
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }

    /// Create an error response with additional data
    pub fn error_with_data(id: Value, code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
            id,
        }
    }
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code (standard or application-defined)
    pub code: i32,

    /// Human-readable error message
    pub message: String,

    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC 2.0 error codes
pub mod error_codes {
    /// Invalid JSON was received by the server
    pub const PARSE_ERROR: i32 = -32700;

    /// The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;

    /// The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32601;

    /// Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32602;

    /// Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Reserved for implementation-defined server-errors
    /// Range: -32000 to -32099
    pub const SERVER_ERROR_START: i32 = -32000;
    /// Upper bound of the server error range
    pub const SERVER_ERROR_END: i32 = -32099;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest::new("capability.list", json!({}), json!(1));

        let json = serde_json::to_string(&request).expect("request serialization");
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"capability.list\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(json!(1), json!({"status": "ok"}));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response =
            JsonRpcResponse::error(json!(1), error_codes::METHOD_NOT_FOUND, "Method not found");

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response
            .error
            .expect("error response should have error field");
        assert_eq!(error.code, error_codes::METHOD_NOT_FOUND);
        assert_eq!(error.message, "Method not found");
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "health.get",
            "params": {},
            "id": 42
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json).expect("request deserialization");
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "health.get");
        assert_eq!(request.id, json!(42));
    }

    #[test]
    fn test_binary_payload_roundtrip() {
        use super::BinaryPayload;

        let payload = BinaryPayload {
            data: bytes::Bytes::from_static(b"hello binary"),
            content_type: "application/octet-stream".to_string(),
        };
        let json = serde_json::to_string(&payload).expect("payload serialization");
        assert!(json.contains("content_type"));
        let restored: BinaryPayload = serde_json::from_str(&json).expect("payload deserialization");
        assert_eq!(restored.data.as_ref(), b"hello binary");
        assert_eq!(restored.content_type, "application/octet-stream");
    }

    #[test]
    fn test_json_rpc_response_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"status": "healthy"},
            "id": 1
        }"#;

        let response: JsonRpcResponse =
            serde_json::from_str(json).expect("response deserialization");
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error_with_data() {
        let response = JsonRpcResponse::error_with_data(
            json!(1),
            error_codes::INVALID_PARAMS,
            "Invalid params",
            json!({"field": "session_id"}),
        );
        assert!(response.result.is_none());
        let err = response.error.expect("error");
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert_eq!(err.message, "Invalid params");
        assert_eq!(err.data, Some(json!({"field": "session_id"})));
    }

    #[test]
    fn test_error_codes_constants() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }
}
