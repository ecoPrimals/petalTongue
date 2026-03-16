// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovery types for primal endpoint information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primal endpoint information
///
/// Represents a discovered primal's connection details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Unique primal identifier (UUID)
    pub primal_id: String,

    /// Human-readable primal name (optional)
    pub name: Option<String>,

    /// Endpoint URL (e.g., "<tarpc://hostname:9001>")
    pub endpoint: String,

    /// Capabilities this primal provides
    pub capabilities: Vec<String>,

    /// Primal type (e.g., "petalTongue", "Toadstool", "Songbird")
    pub primal_type: String,

    /// Protocol used (e.g., "tarpc", "jsonrpc", "https")
    pub protocol: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}
