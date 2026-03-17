// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ecosystem-standard structured IPC error types.
//!
//! Absorbed from rhizoCrypt, loamSpine, sweetGrass, and healthSpring.
//! Provides phase-aware error classification and NDJSON stream item types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;

/// JSON-RPC 2.0 method-not-found error code.
pub const JSON_RPC_METHOD_NOT_FOUND: i64 = -32601;

/// Phase of IPC communication that failed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IpcErrorPhase {
    /// Socket connection failed.
    #[error("Connect: {0}")]
    Connect(String),

    /// Writing request failed.
    #[error("Write: {0}")]
    Write(String),

    /// Reading response failed.
    #[error("Read: {0}")]
    Read(String),

    /// Response was not valid JSON.
    #[error("InvalidJson: {0}")]
    InvalidJson(String),

    /// Server returned a JSON-RPC error.
    #[error("JSON-RPC error (code {code}): {message}")]
    JsonRpcError {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable message.
        message: String,
    },

    /// Operation timed out.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Request/response serialization failed.
    #[error("Serialization: {0}")]
    Serialization(String),
}

impl IpcErrorPhase {
    /// Returns true if the error is recoverable (transient, may succeed on retry).
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::Read(_) | Self::Timeout(_))
    }

    /// Returns true if the operation should be retried.
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::Read(_) | Self::Timeout(_))
    }

    /// Returns true if this is a timeout error.
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Returns true when JsonRpcError with code -32601 (method not found).
    pub const fn is_method_not_found(&self) -> bool {
        matches!(
            self,
            Self::JsonRpcError { code, .. } if *code == JSON_RPC_METHOD_NOT_FOUND
        )
    }
}

/// Extracts error code and message from a JSON-RPC response value.
///
/// Looks for `response["error"]["code"]` (as i64) and `response["error"]["message"]` (as string).
/// Returns `None` if the structure is missing or invalid.
pub fn extract_rpc_error(response: &Value) -> Option<(i64, String)> {
    let error = response.get("error")?;
    let code = error.get("code")?.as_i64()?;
    let message = error
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some((code, message))
}

/// NDJSON pipeline stream item (ecosystem pattern from rhizoCrypt/sweetGrass/biomeOS).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamItem {
    /// Payload data.
    Data(Value),

    /// Progress update.
    Progress {
        /// Progress percent (0.0..=1.0).
        percent: f64,
        /// Status message.
        message: String,
    },

    /// Stream complete.
    End {
        /// Optional summary payload.
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<Value>,
    },

    /// Stream error.
    Error {
        /// Error code.
        code: i64,
        /// Error message.
        message: String,
    },
}

impl StreamItem {
    /// Returns true if this item terminates the stream (End or Error).
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::End { .. } | Self::Error { .. })
    }

    /// Serializes to a single NDJSON line (includes trailing newline).
    pub fn to_ndjson_line(&self) -> String {
        serde_json::to_string(self).map_or_else(|_| String::new(), |s| s + "\n")
    }

    /// Deserializes from a JSON line.
    pub fn parse_ndjson_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

impl fmt::Display for StreamItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(v) => write!(f, "Data({v})"),
            Self::Progress { percent, message } => {
                write!(f, "Progress({:.0}%: {})", percent * 100.0, message)
            }
            Self::End { summary } => {
                write!(f, "End(summary={})", summary.is_some())
            }
            Self::Error { code, message } => write!(f, "Error({code}: {message})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_error_phase_recoverable() {
        assert!(IpcErrorPhase::Connect("refused".into()).is_recoverable());
        assert!(IpcErrorPhase::Read("eof".into()).is_recoverable());
        assert!(IpcErrorPhase::Timeout("5s".into()).is_recoverable());
        assert!(!IpcErrorPhase::Write("broken".into()).is_recoverable());
        assert!(!IpcErrorPhase::InvalidJson("bad".into()).is_recoverable());
        assert!(
            !IpcErrorPhase::JsonRpcError {
                code: -32601,
                message: "method not found".into()
            }
            .is_recoverable()
        );
        assert!(!IpcErrorPhase::Serialization("invalid".into()).is_recoverable());
    }

    #[test]
    fn ipc_error_phase_retriable() {
        assert!(IpcErrorPhase::Connect("refused".into()).is_retriable());
        assert!(IpcErrorPhase::Read("eof".into()).is_retriable());
        assert!(IpcErrorPhase::Timeout("5s".into()).is_retriable());
        assert!(!IpcErrorPhase::Write("broken".into()).is_retriable());
    }

    #[test]
    fn ipc_error_phase_timeout() {
        assert!(IpcErrorPhase::Timeout("5s".into()).is_timeout());
        assert!(!IpcErrorPhase::Connect("x".into()).is_timeout());
    }

    #[test]
    fn ipc_error_phase_method_not_found() {
        assert!(
            IpcErrorPhase::JsonRpcError {
                code: JSON_RPC_METHOD_NOT_FOUND,
                message: "method not found".into()
            }
            .is_method_not_found()
        );
        assert!(
            !IpcErrorPhase::JsonRpcError {
                code: -32600,
                message: "parse error".into()
            }
            .is_method_not_found()
        );
        assert!(!IpcErrorPhase::Connect("x".into()).is_method_not_found());
    }

    #[test]
    fn extract_rpc_error_valid() {
        let resp = serde_json::json!({
            "error": { "code": -32601, "message": "Method not found" }
        });
        assert_eq!(
            extract_rpc_error(&resp),
            Some((-32601, "Method not found".into()))
        );
    }

    #[test]
    fn extract_rpc_error_missing_error() {
        let resp = serde_json::json!({ "result": null });
        assert_eq!(extract_rpc_error(&resp), None);
    }

    #[test]
    fn extract_rpc_error_missing_code() {
        let resp = serde_json::json!({ "error": { "message": "oops" } });
        assert_eq!(extract_rpc_error(&resp), None);
    }

    #[test]
    fn extract_rpc_error_empty_message() {
        let resp = serde_json::json!({ "error": { "code": -32000 } });
        assert_eq!(extract_rpc_error(&resp), Some((-32000, String::new())));
    }

    #[test]
    fn stream_item_terminal() {
        assert!(!StreamItem::Data(serde_json::json!(1)).is_terminal());
        assert!(
            !StreamItem::Progress {
                percent: 0.5,
                message: "half".into()
            }
            .is_terminal()
        );
        assert!(StreamItem::End { summary: None }.is_terminal());
        assert!(
            StreamItem::End {
                summary: Some(serde_json::json!({}))
            }
            .is_terminal()
        );
        assert!(
            StreamItem::Error {
                code: -1,
                message: "fail".into()
            }
            .is_terminal()
        );
    }

    #[test]
    fn stream_item_ndjson_roundtrip() {
        let items = [
            StreamItem::Data(serde_json::json!({"x": 42})),
            StreamItem::Progress {
                percent: 0.75,
                message: "almost done".into(),
            },
            StreamItem::End {
                summary: Some(serde_json::json!({"count": 10})),
            },
            StreamItem::Error {
                code: -32601,
                message: "method not found".into(),
            },
        ];
        for item in &items {
            let line = item.to_ndjson_line();
            let parsed = StreamItem::parse_ndjson_line(&line).expect("parse");
            assert_eq!(item, &parsed);
        }
    }

    #[test]
    fn stream_item_parse_ndjson_line() {
        let line = r#"{"data":{"id":1}}"#;
        let item = StreamItem::parse_ndjson_line(line).expect("parse");
        assert_eq!(item, StreamItem::Data(serde_json::json!({"id": 1})));

        let line_with_newline = r#"{"end":{}}"#;
        let item = StreamItem::parse_ndjson_line(line_with_newline).expect("parse");
        assert_eq!(item, StreamItem::End { summary: None });
    }

    #[test]
    fn stream_item_display() {
        assert!(format!("{}", StreamItem::Data(serde_json::json!(1))).contains("Data"));
        assert!(
            format!(
                "{}",
                StreamItem::Progress {
                    percent: 0.5,
                    message: "half".into()
                }
            )
            .contains("50%")
        );
    }
}
