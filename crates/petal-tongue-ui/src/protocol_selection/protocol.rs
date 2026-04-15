// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endpoint scheme detection and HTTPS URL fallback helpers.

use tracing::debug;

/// Protocol priority for primal-to-primal communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    /// tarpc - PRIMARY (highest priority)
    Tarpc = 1,
    /// JSON-RPC - SECONDARY
    JsonRpc = 2,
    /// HTTPS - FALLBACK (lowest priority)
    Https = 3,
}

/// Detected protocol for an endpoint
#[derive(Debug, Clone)]
pub struct DetectedProtocol {
    /// Protocol type
    pub protocol: Protocol,
    /// Endpoint string
    pub endpoint: String,
}

/// Detect protocol from endpoint string
///
/// # Arguments
/// * `endpoint` - Endpoint URL (e.g., "<tarpc://localhost:9001>", "<unix:///tmp/service.sock>")
///
/// # Returns
/// Detected protocol or HTTPS as fallback
pub fn detect_protocol(endpoint: &str) -> Protocol {
    if endpoint.starts_with("tarpc://") {
        Protocol::Tarpc
    } else if endpoint.starts_with("unix://") || endpoint.starts_with("ipc://") {
        Protocol::JsonRpc // Unix sockets use JSON-RPC
    } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        Protocol::Https
    } else {
        debug!(
            "Unknown protocol for endpoint '{}', defaulting to HTTPS",
            endpoint
        );
        Protocol::Https
    }
}

#[must_use]
pub fn https_fallback_urls(endpoint: &str) -> Vec<String> {
    if endpoint.starts_with("https://") {
        vec![
            endpoint.to_string(),
            endpoint.replacen("https://", "http://", 1),
        ]
    } else {
        vec![endpoint.to_string()]
    }
}
