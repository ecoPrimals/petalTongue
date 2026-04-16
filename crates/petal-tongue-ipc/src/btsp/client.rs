// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog BTSP provider client (JSON-RPC over UDS).

/// Call a BearDog `btsp.session.*` RPC via UDS.
pub(super) async fn provider_call(
    socket: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut stream = tokio::net::UnixStream::connect(socket)
        .await
        .map_err(|e| format!("BTSP provider {}: {e}", socket.display()))?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    line.push('\n');
    stream
        .write_all(line.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stream.flush().await.map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| e.to_string())?;

    let resp: serde_json::Value =
        serde_json::from_str(&response_line).map_err(|e| e.to_string())?;
    if let Some(err) = resp.get("error") {
        return Err(format!("BTSP provider error: {err}"));
    }
    resp.get("result")
        .cloned()
        .ok_or_else(|| "no result in provider response".to_owned())
}
