// SPDX-License-Identifier: AGPL-3.0-only
//! Core types for petalTongue visualization system

use crate::property::Properties;
use serde::{Deserialize, Serialize};

// OPTIMIZATION: Common property keys as static constants to avoid allocations
const PROP_TRUST_LEVEL: &str = "trust_level";
const PROP_FAMILY_ID: &str = "family_id";

/// Endpoints for different protocols (biomeOS format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Unix socket path (e.g., "/tmp/beardog-node-alpha.sock")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unix_socket: Option<String>,

    /// HTTP endpoint (e.g., `<http://localhost:8080>`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<String>,
}

/// Metadata about a primal (biomeOS format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Primal version (e.g., "v0.15.2")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Family ID (genetic lineage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,

    /// Node ID within the family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

/// Connection metrics (biomeOS format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    /// Total request count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_count: Option<u64>,

    /// Average latency in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms: Option<f64>,

    /// Error count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_count: Option<u64>,
}

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique identifier for the primal
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of primal (e.g., "Compute", "Storage", "Security")
    #[serde(alias = "type")] // biomeOS uses "type" field
    pub primal_type: String,
    /// Network endpoint (e.g., <http://localhost:8080>, `<unix:///tmp/primal.sock>`)
    pub endpoint: String,
    /// List of capabilities this primal provides
    pub capabilities: Vec<String>,
    /// Health status ("Healthy", "Warning", "Critical", "Unknown")
    pub health: PrimalHealthStatus,
    /// Last time this primal was seen (Unix timestamp)
    pub last_seen: u64,

    /// Endpoints for different protocols (biomeOS format)
    /// Supports both HTTP and Unix socket endpoints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<PrimalEndpoints>,

    /// Metadata from primal (version, `node_id`, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PrimalMetadata>,

    // === UNIVERSAL PROPERTIES (ecosystem-agnostic) ===
    /// Generic properties for ecosystem-specific data
    ///
    /// This field holds all ecosystem-specific data in a generic format.
    /// Adapters interpret these properties at runtime to provide rich UI.
    ///
    /// Examples:
    /// - "`trust_level"`: `PropertyValue::Number(2.0)`
    /// - "`family_id"`: `PropertyValue::String("family-abc`")
    /// - "dna": `PropertyValue::String("ACTG`...")
    #[serde(default)]
    pub properties: Properties,

    // === DEPRECATED FIELDS (kept temporarily for backward compatibility) ===
    /// Trust level (0-3: None, Limited, Elevated, Full)
    ///
    /// DEPRECATED: Use properties[`trust_level`] instead
    /// This field is kept temporarily for backward compatibility and will be removed
    /// once all data sources migrate to the properties field.
    #[deprecated(note = "Use properties field instead - this will be removed in a future version")]
    #[serde(default)]
    pub trust_level: Option<u8>,

    /// Family ID (genetic lineage)
    ///
    /// DEPRECATED: Use properties[`family_id`] instead
    /// This field is kept temporarily for backward compatibility and will be removed
    /// once all data sources migrate to the properties field.
    #[deprecated(note = "Use properties field instead - this will be removed in a future version")]
    #[serde(default)]
    pub family_id: Option<String>,
}

impl PrimalInfo {
    /// Create a new `PrimalInfo` with basic information
    ///
    /// For ecosystem-specific data (trust, family, etc.), use the `properties` field
    /// or the deprecated `with_trust` method for backward compatibility.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        primal_type: impl Into<String>,
        endpoint: impl Into<String>,
        capabilities: Vec<String>,
        health: PrimalHealthStatus,
        last_seen: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            primal_type: primal_type.into(),
            endpoint: endpoint.into(),
            capabilities,
            health,
            last_seen,
            endpoints: None,
            metadata: None,
            properties: Properties::new(),
            #[expect(deprecated)]
            trust_level: None,
            #[expect(deprecated)]
            family_id: None,
        }
    }

    /// Migrate deprecated fields to properties
    ///
    /// This method ensures that `trust_level` and `family_id` from the deprecated fields
    /// are copied into the properties field for adapter-based rendering.
    ///
    /// Also migrates biomeOS metadata fields to properties.
    ///
    /// Call this after deserializing from JSON to ensure backward compatibility.
    #[expect(deprecated)]
    pub fn migrate_deprecated_fields(&mut self) {
        use crate::property::PropertyValue;

        // Migrate trust_level if present and not already in properties
        // OPTIMIZATION: Use static string constant
        if let Some(trust) = self.trust_level
            && !self.properties.contains_key(PROP_TRUST_LEVEL)
        {
            self.properties.insert(
                PROP_TRUST_LEVEL.to_string(),
                PropertyValue::Number(f64::from(trust)),
            );
        }

        // Migrate family_id if present and not already in properties
        // OPTIMIZATION: Use static string constant
        if let Some(ref family) = self.family_id
            && !self.properties.contains_key(PROP_FAMILY_ID)
        {
            self.properties.insert(
                PROP_FAMILY_ID.to_string(),
                PropertyValue::String(family.clone()),
            );
        }

        // Migrate biomeOS metadata to properties
        if let Some(ref metadata) = self.metadata {
            if let Some(ref version) = metadata.version {
                self.properties.insert(
                    "version".to_string(),
                    PropertyValue::String(version.clone()),
                );
            }
            if let Some(ref family) = metadata.family_id {
                self.properties.insert(
                    PROP_FAMILY_ID.to_string(),
                    PropertyValue::String(family.clone()),
                );
            }
            if let Some(ref node_id) = metadata.node_id {
                self.properties.insert(
                    "node_id".to_string(),
                    PropertyValue::String(node_id.clone()),
                );
            }
        }

        // Set endpoint from endpoints if available and primary endpoint is empty
        if let Some(ref endpoints) = self.endpoints
            && (self.endpoint.is_empty() || self.endpoint == "unknown")
        {
            // Prefer Unix socket for local primals
            if let Some(ref unix_socket) = endpoints.unix_socket {
                self.endpoint = format!("unix://{unix_socket}");
            } else if let Some(ref http) = endpoints.http {
                self.endpoint = http.clone();
            }
        }
    }

    /// Add trust information to this primal
    ///
    /// DEPRECATED: Populate the `properties` field directly instead:
    /// ```ignore
    /// use petal_tongue_core::property::PropertyValue;
    /// info.properties.insert(PROP_TRUST_LEVEL.to_string(), PropertyValue::Number(2.0));
    /// info.properties.insert(PROP_FAMILY_ID.to_string(), PropertyValue::String("family-abc".to_string()));
    /// ```
    #[deprecated(note = "Use properties field directly instead")]
    #[must_use]
    #[expect(deprecated)]
    pub fn with_trust(mut self, trust_level: u8, family_id: Option<String>) -> Self {
        use crate::property::PropertyValue;

        self.trust_level = Some(trust_level);
        self.family_id.clone_from(&family_id);

        // Also populate properties for forward compatibility
        // OPTIMIZATION: Use static string constants
        self.properties.insert(
            PROP_TRUST_LEVEL.to_string(),
            PropertyValue::Number(f64::from(trust_level)),
        );
        if let Some(fid) = family_id {
            self.properties
                .insert(PROP_FAMILY_ID.to_string(), PropertyValue::String(fid));
        }

        self
    }
}

/// Health status of a primal (visualization-specific)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalHealthStatus {
    /// Primal is operating normally
    Healthy,
    /// Primal has minor issues but is functional
    Warning,
    /// Primal has major issues
    Critical,
    /// Health status is unknown
    Unknown,
}

impl PrimalHealthStatus {
    /// Get the string representation
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
            Self::Unknown => "Unknown",
        }
    }

    /// Parse from string
    #[must_use]
    pub fn parse_health_status(s: &str) -> Self {
        match s {
            "Healthy" => Self::Healthy,
            "Warning" => Self::Warning,
            "Critical" => Self::Critical,
            _ => Self::Unknown,
        }
    }
}

/// Connection status to a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Successfully connected
    Connected,
    /// Attempting to connect
    Connecting,
    /// Not connected
    Disconnected,
    /// Connection error
    Error(String),
}

/// Connection to a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConnection {
    /// Primal name
    pub name: String,
    /// Type of primal
    pub primal_type: String,
    /// Connection status
    pub status: ConnectionStatus,
    /// Network endpoint
    pub endpoint: String,
    /// Last heartbeat timestamp (Unix timestamp)
    pub last_heartbeat: Option<u64>,
}

/// Topology graph containing primals and their relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyGraph {
    /// List of primals in the topology
    pub nodes: Vec<PrimalInfo>,
    /// Connections between primals
    pub edges: Vec<TopologyEdge>,
    /// When this topology was captured
    pub timestamp: u64,
}

/// Edge (connection) in the topology graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Type of relationship (e.g., `api_call`, `capability`, `capability_invocation`)
    #[serde(default = "default_edge_type", alias = "type")]
    pub edge_type: String,
    /// Optional label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Specific capability being invoked (biomeOS format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,

    /// Connection metrics (biomeOS format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<ConnectionMetrics>,
}

fn default_edge_type() -> String {
    "connection".to_string()
}

/// Real-time flow event showing message between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    /// Event ID
    pub id: String,
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Type of message
    pub message_type: String,
    /// When the event occurred (Unix timestamp)
    pub timestamp: u64,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Traffic statistics between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Number of messages
    pub message_count: u64,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Time period for these stats (Unix timestamp)
    pub period_start: u64,
    /// End of time period (Unix timestamp)
    pub period_end: u64,
}
