// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON parsing helpers for HTTP health/capabilities probes.

use petal_tongue_ipc::TarpcClientError;

#[must_use]
pub fn parse_health_from_json(value: &serde_json::Value) -> petal_tongue_ipc::HealthStatus {
    let status = value
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let uptime_seconds = value
        .get("uptime_seconds")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let capabilities = value
        .get("capabilities")
        .or_else(|| value.get("modalities_active"))
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    petal_tongue_ipc::HealthStatus {
        status,
        version,
        uptime_seconds,
        capabilities,
        details: std::collections::HashMap::new(),
    }
}

/// Parse capabilities array from JSON response.
///
/// # Errors
///
/// Returns an error if the response is missing the 'capabilities' array.
pub fn parse_capabilities_from_json(
    value: &serde_json::Value,
) -> Result<Vec<String>, TarpcClientError> {
    value
        .get("capabilities")
        .and_then(|c| c.as_array())
        .ok_or_else(|| {
            TarpcClientError::Configuration("Response missing 'capabilities' array".to_string())
        })
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
}
