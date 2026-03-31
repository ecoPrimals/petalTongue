// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::unix_socket_rpc_handlers::RpcHandlers;
use serde_json::json;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error};

/// Errors from handling a Unix socket connection.
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// I/O error.
    #[error("Connection I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Failed to serialize response: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Handle a single connection: read newline-delimited JSON-RPC requests,
/// dispatch to handlers, write newline-delimited responses.
///
/// Transport-generic: works with `UnixStream`, `TcpStream`, or any
/// `AsyncRead + AsyncWrite` implementor.
pub async fn handle_connection<S>(handler: &RpcHandlers, stream: S) -> Result<(), ConnectionError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::with_capacity(65_536, reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            break;
        }

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                debug!(
                    "Received request: method={}, id={}",
                    request.method, request.id
                );
                handler.handle_request(request).await
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                JsonRpcResponse::error(
                    json!(null),
                    error_codes::PARSE_ERROR,
                    format!("Parse error: {e}"),
                )
            }
        };

        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_json_rpc_request_line() {
        let line = r#"{"jsonrpc":"2.0","method":"health.get","params":{},"id":42}"#;
        let request: JsonRpcRequest = serde_json::from_str(line).expect("parse");
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "health.get");
        assert_eq!(request.id, serde_json::json!(42));
    }

    #[test]
    fn parse_json_rpc_request_with_params() {
        let line = r#"{"jsonrpc":"2.0","method":"capability.list","params":{"filter":"compute"},"id":"req-1"}"#;
        let request: JsonRpcRequest = serde_json::from_str(line).expect("parse");
        assert_eq!(request.method, "capability.list");
        assert_eq!(request.id, serde_json::json!("req-1"));
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let line = r#"{"jsonrpc":"2.0","method":}"#;
        let result: Result<JsonRpcRequest, _> = serde_json::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn json_rpc_response_error_serialization() {
        let response = JsonRpcResponse::error(
            json!(null),
            error_codes::PARSE_ERROR,
            "Parse error: invalid json",
        );
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"error\""));
        let restored: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.error.is_some());
        assert_eq!(restored.error.unwrap().code, error_codes::PARSE_ERROR);
    }

    #[test]
    fn json_rpc_response_error_with_data() {
        let response = JsonRpcResponse::error_with_data(
            json!(1),
            error_codes::INVALID_PARAMS,
            "Invalid params",
            json!({"field": "session_id"}),
        );
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.data.is_some());
    }

    #[test]
    fn json_rpc_request_missing_params_deserializes() {
        let line = r#"{"jsonrpc":"2.0","method":"health.get","id":1}"#;
        let request: JsonRpcRequest = serde_json::from_str(line).expect("parse");
        assert_eq!(request.method, "health.get");
    }

    #[test]
    fn connection_error_display() {
        let err = ConnectionError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "refused",
        ));
        assert!(err.to_string().contains("Connection"));
        assert!(err.to_string().contains("refused"));
    }
}
