// SPDX-License-Identifier: AGPL-3.0-only

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::unix_socket_rpc_handlers::RpcHandlers;
use anyhow::Result;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error};

/// Handle a single Unix socket connection: read JSON-RPC requests, dispatch to handlers, write responses
pub async fn handle_connection(handler: &RpcHandlers, stream: UnixStream) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
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
}
