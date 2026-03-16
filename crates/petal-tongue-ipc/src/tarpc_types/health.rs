// SPDX-License-Identifier: AGPL-3.0-or-later
//! Health, version, and protocol status types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status
///
/// Operational health and status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Status string ("healthy", "degraded", "unhealthy")
    pub status: String,

    /// Primal version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Current capabilities available
    pub capabilities: Vec<String>,

    /// Optional health details
    #[serde(default)]
    pub details: HashMap<String, String>,
}

/// Version information
///
/// Version and compatibility details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Primal version string (e.g., "1.2.0")
    pub version: String,

    /// tarpc protocol version
    pub tarpc_version: String,

    /// JSON-RPC protocol version
    pub jsonrpc_version: String,

    /// HTTPS API version (if enabled)
    pub https_version: Option<String>,

    /// Supported capabilities
    pub capabilities: Vec<String>,
}

/// Protocol information
///
/// Details about a supported communication protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    /// Protocol name ("tarpc", "jsonrpc", "https")
    pub name: String,

    /// Endpoint (e.g., "<tarpc://localhost:9001>", "<unix:///tmp/petaltongue.sock>")
    pub endpoint: String,

    /// Whether this protocol is currently enabled
    pub enabled: bool,

    /// Protocol priority (1 = primary, 2 = secondary, 3 = fallback)
    pub priority: u8,

    /// Optional additional info
    #[serde(default)]
    pub info: HashMap<String, String>,
}
