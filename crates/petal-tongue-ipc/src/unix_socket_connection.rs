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
