// SPDX-License-Identifier: AGPL-3.0-or-later

//! Membrane stack reader — serves the live nested membrane profile via JSON-RPC.
//!
//! Reads `/opt/membrane/live-terminal/membrane-stack.json` (written by
//! `skunky-ingest::membrane_stack` every 5 minutes) and returns it
//! as a `pt.membrane_stack` JSON-RPC response.
//!
//! This is the petalTongue side of the convergence pipeline:
//! `skunky-ingest` computes → JSON file → petalTongue reads → WS → browser.

use std::path::Path;

const DEFAULT_STACK_PATH: &str = "/opt/membrane/live-terminal/membrane-stack.json";

/// Read the membrane stack JSON from disk.
///
/// Returns the parsed JSON value for direct inclusion in RPC responses.
pub(crate) fn read_membrane_stack() -> Result<serde_json::Value, String> {
    let path_str = std::env::var("MEMBRANE_STACK_JSON")
        .unwrap_or_else(|_| DEFAULT_STACK_PATH.to_owned());
    let path = Path::new(&path_str);

    let json_bytes = std::fs::read(path)
        .map_err(|e| format!("membrane-stack read: {e}"))?;
    let stack: serde_json::Value = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("membrane-stack parse: {e}"))?;

    Ok(stack)
}
