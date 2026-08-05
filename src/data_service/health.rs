// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-primal health liveness queries via UDS.
//!
//! Connects to each primal's UDS socket and issues a `health.liveness`
//! JSON-RPC call with BTSP framing. Returns structured health state
//! for the nestgate.io dashboard.

use serde::Serialize;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

const QUERY_TIMEOUT: Duration = Duration::from_secs(2);

/// Known primal socket mappings (name → full socket path).
///
/// Most primals use `/run/membrane/{name}.sock`, but some use
/// family-qualified names or live in the user biomeos directory.
const PRIMAL_SOCKETS: &[(&str, &str)] = &[
    ("sweetgrass", "/run/membrane/sweetgrass.sock"),
    ("loamspine", "/run/membrane/loamspine.sock"),
    ("rhizocrypt", "/run/membrane/rhizocrypt.sock"),
    ("beardog", "/run/membrane/beardog.sock"),
    ("squirrel", "/run/membrane/squirrel.sock"),
    ("toadstool", "/run/membrane/toadstool.sock"),
    ("biomeos", "/run/membrane/biomeos.sock"),
    ("songbird", "/run/membrane/songbird.sock"),
    ("barracuda", "/run/membrane/barracuda.sock"),
    ("coralreef", "/run/membrane/coralreef.sock"),
    ("skunkbat", "/run/membrane/security.sock"),
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

/// Query `health.liveness` on a single primal via UDS with BTSP framing.
async fn query_health(primal: &str, socket_path: &str) -> PrimalHealth {

    let result = tokio::time::timeout(QUERY_TIMEOUT, async {
        let mut stream = UnixStream::connect(&socket_path).await?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 1
        });
        let payload = serde_json::to_vec(&request)?;

        // BTSP frame: 0xEC 0x01 prefix
        let mut frame = vec![0xEC, 0x01];
        frame.extend_from_slice(&payload);
        stream.write_all(&frame).await?;
        stream.shutdown().await?;

        let mut buf = Vec::with_capacity(4096);
        stream.read_to_end(&mut buf).await?;

        // Strip BTSP prefix if present
        let json_start = if buf.len() >= 2 && buf[0] == 0xEC && buf[1] == 0x01 {
            2
        } else {
            0
        };

        // Some primals send multiple JSON objects (e.g., an error then a result).
        // Find the first valid JSON-RPC response with a "result" field.
        let raw = &buf[json_start..];
        let response = parse_first_result(raw)?;

        Ok::<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>(response)
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
            let alive = resp.get("alive").and_then(|v| v.as_bool()).unwrap_or(false)
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
