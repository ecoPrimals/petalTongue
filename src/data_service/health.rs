// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-primal health liveness queries via UDS.
//!
//! G65 primals enforce riboCipher transport signal (0xEC prefix).
//! Some primals (beardog) require a full BTSP handshake on their main
//! socket but offer a `-default.sock` for plaintext health checks.
//!
//! Strategy: try BTSP-framed query first; on connection reset / EOF,
//! retry with plain JSON-RPC (handles coralReef's G65 plain mode).

use serde::Serialize;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

const QUERY_TIMEOUT: Duration = Duration::from_secs(3);

/// Known primal socket mappings (name → primary socket, optional fallback).
///
/// After G65, beardog's main socket requires a full BTSP handshake;
/// use `beardog-default.sock` for plaintext health. skunkBat runs as
/// root with a family-qualified socket under `/run/user/0/biomeos/`.
const PRIMAL_SOCKETS: &[(&str, &str)] = &[
    ("sweetgrass", "/run/membrane/sweetgrass.sock"),
    ("loamspine", "/run/membrane/loamspine.sock"),
    ("rhizocrypt", "/run/membrane/rhizocrypt.sock"),
    ("beardog", "/run/membrane/beardog-default.sock"),
    ("squirrel", "/run/membrane/squirrel.sock"),
    ("toadstool", "/run/membrane/toadstool.sock"),
    ("biomeos", "/run/membrane/biomeos.sock"),
    ("songbird", "/run/membrane/songbird.sock"),
    ("barracuda", "/run/membrane/barracuda.sock"),
    ("coralreef", "/run/membrane/coralreef.sock"),
    ("skunkbat", "/run/user/0/biomeos/skunkbat-e8b62b6e.sock"),
    ("nestgate", "/run/membrane/nestgate-e8b62b6e.sock"),
    ("petaltongue", "/run/user/1000/biomeos/petaltongue-e8b62b6e.sock"),
];

#[derive(Debug, Clone, Serialize)]
pub struct PrimalHealth {
    pub primal: String,
    pub alive: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Query `health.liveness` on a single primal via UDS.
///
/// Tries BTSP-framed request first (riboCipher 0xEC 0x01 prefix),
/// then falls back to plain JSON-RPC for primals that accept it
/// natively (beardog-default, coralReef).
async fn query_health(primal: &str, socket_path: &str) -> PrimalHealth {

    let result = tokio::time::timeout(QUERY_TIMEOUT, async {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 1
        });
        let payload = serde_json::to_vec(&request)?;

        // Try BTSP-framed first
        let resp = send_uds(socket_path, &payload, true).await;
        if let Ok(val) = resp {
            return Ok(val);
        }

        // Fallback: plain JSON-RPC (for coralReef, beardog-default, etc.)
        send_uds(socket_path, &payload, false).await
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
            let alive = resp.get("alive").and_then(serde_json::Value::as_bool).unwrap_or(false)
                || resp
                    .get("status")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s == "alive" || s == "ok");
            let status = resp
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or(if alive { "alive" } else { "unknown" })
                .to_string();
            let version = resp
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from);

            PrimalHealth {
                primal: primal.to_string(),
                alive,
                status,
                version,
                error: None,
            }
        }
        Ok(Err(e)) => PrimalHealth {
            primal: primal.to_string(),
            alive: false,
            status: "error".to_string(),
            version: None,
            error: Some(e.to_string()),
        },
        Err(_) => PrimalHealth {
            primal: primal.to_string(),
            alive: false,
            status: "timeout".to_string(),
            version: None,
            error: Some("UDS query timed out".to_string()),
        },
    }
}

/// Send a JSON-RPC payload over UDS, optionally with BTSP framing.
async fn send_uds(
    path: &str,
    payload: &[u8],
    btsp: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(path).await?;

    if btsp {
        let mut frame = vec![0xEC, 0x01];
        frame.extend_from_slice(payload);
        stream.write_all(&frame).await?;
    } else {
        stream.write_all(payload).await?;
    }
    stream.shutdown().await?;

    let mut buf = Vec::with_capacity(4096);
    stream.read_to_end(&mut buf).await?;

    let json_start = if buf.len() >= 2 && buf[0] == 0xEC && buf[1] == 0x01 {
        2
    } else {
        0
    };

    parse_first_result(&buf[json_start..])
}

/// Parse the first JSON-RPC response that contains a "result" field.
/// Handles multi-object streams (e.g., bearDog sends error + result).
fn parse_first_result(
    raw: &[u8],
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let text = std::str::from_utf8(raw)?;

    // Try to find multiple JSON objects separated by newlines or concatenated
    let mut decoder = serde_json::Deserializer::from_str(text).into_iter::<serde_json::Value>();

    while let Some(Ok(val)) = decoder.next() {
        if val.get("result").is_some() {
            return Ok(val["result"].clone());
        }
    }

    // Fallback: try parsing the whole thing as one object
    let val: serde_json::Value = serde_json::from_str(text)?;
    if let Some(result) = val.get("result") {
        return Ok(result.clone());
    }

    Err("no result field in response".into())
}

/// Query health.liveness on all known primals concurrently.
pub async fn query_all_health() -> Vec<PrimalHealth> {
    let mut set = tokio::task::JoinSet::new();
    for (name, sock) in PRIMAL_SOCKETS {
        let name = name.to_string();
        let sock = sock.to_string();
        set.spawn(async move { query_health(&name, &sock).await });
    }

    let mut results = Vec::with_capacity(PRIMAL_SOCKETS.len());
    while let Some(Ok(health)) = set.join_next().await {
        results.push(health);
    }
    results.sort_by(|a, b| a.primal.cmp(&b.primal));
    results
}
