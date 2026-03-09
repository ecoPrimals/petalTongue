// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC provider types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

/// JSON-RPC 2.0 provider for Unix socket communication
pub struct JsonRpcProvider {
    pub(crate) socket_path: PathBuf,
    pub(crate) request_id: AtomicU64,
    pub(crate) timeout: Duration,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
