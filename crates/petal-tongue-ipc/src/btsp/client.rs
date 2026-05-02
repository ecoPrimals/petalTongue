// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog BTSP provider client (JSON-RPC over UDS).

use super::error::BtspHandshakeError;

/// Call a BearDog BTSP RPC method via UDS (`btsp.session.*`, `btsp.negotiate`).
pub(super) async fn provider_call(
    socket: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, BtspHandshakeError> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut stream = tokio::net::UnixStream::connect(socket).await.map_err(|e| {
        BtspHandshakeError::ProviderConnect {
            path: socket.to_path_buf(),
            source: e,
        }
    })?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut line = serde_json::to_vec(&request).map_err(|e| BtspHandshakeError::Json {
        context: "serialize RPC request",
        source: e,
    })?;
    line.push(b'\n');
    stream
        .write_all(&line)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write RPC request",
            source: e,
        })?;
    stream.flush().await.map_err(|e| BtspHandshakeError::Io {
        context: "flush RPC request",
        source: e,
    })?;

    let mut reader = BufReader::new(stream);
    let mut response_buf: Vec<u8> = Vec::with_capacity(4096);
    reader
        .read_until(b'\n', &mut response_buf)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read RPC response",
            source: e,
        })?;

    let resp: serde_json::Value =
        serde_json::from_slice(&response_buf).map_err(|e| BtspHandshakeError::Json {
            context: "parse RPC response",
            source: e,
        })?;
    if let Some(err) = resp.get("error") {
        return Err(BtspHandshakeError::ProviderRpcError(err.clone()));
    }
    resp.get("result")
        .cloned()
        .ok_or(BtspHandshakeError::NoResult)
}
