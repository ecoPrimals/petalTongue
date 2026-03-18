// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC helpers for biomeOS provider communication.
//!
//! Builds requests, parses responses, and extracts health/niche data.

use crate::biomeos_integration::types::NicheTemplate;

#[must_use]
pub fn build_jsonrpc_request(
    method: &str,
    params: serde_json::Value,
    id: u64,
) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    })
}

#[must_use]
pub fn parse_jsonrpc_result(response: &serde_json::Value) -> Option<serde_json::Value> {
    response.get("result").cloned()
}

#[must_use]
pub fn parse_jsonrpc_error(response: &serde_json::Value) -> Option<serde_json::Value> {
    response.get("error").cloned()
}

#[must_use]
pub fn health_response_status(value: &serde_json::Value) -> String {
    value
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string()
}

#[must_use]
pub fn health_response_healthy(value: &serde_json::Value) -> Option<bool> {
    value.get("healthy").and_then(serde_json::Value::as_bool)
}

#[must_use]
pub fn build_assign_device_params(device_id: &str, primal_id: &str) -> serde_json::Value {
    serde_json::json!({
        "device_id": device_id,
        "primal_id": primal_id,
    })
}

#[must_use]
pub fn build_deploy_niche_params(niche: &NicheTemplate) -> serde_json::Value {
    serde_json::json!({
        "name": niche.name,
        "description": niche.description,
        "required_primals": niche.required_primals,
        "optional_primals": niche.optional_primals,
        "metadata": niche.metadata,
    })
}

#[must_use]
pub fn extract_niche_id_from_response(response: &serde_json::Value) -> Option<&serde_json::Value> {
    response.get("niche_id")
}

#[must_use]
pub fn build_subscribe_events_params() -> serde_json::Value {
    serde_json::json!({
        "events": ["device.added", "device.removed", "primal.status", "niche.deployed"]
    })
}
