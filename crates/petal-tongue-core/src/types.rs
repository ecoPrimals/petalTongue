//! Core types for petalTongue visualization system

use crate::property::Properties;
use serde::{Deserialize, Serialize};

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique identifier for the primal
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of primal (e.g., "Compute", "Storage", "Security")
    pub primal_type: String,
    /// Network endpoint (e.g., <http://localhost:8080>)
    pub endpoint: String,
    /// List of capabilities this primal provides
    pub capabilities: Vec<String>,
    /// Health status ("Healthy", "Warning", "Critical", "Unknown")
    pub health: PrimalHealthStatus,
    /// Last time this primal was seen (Unix timestamp)
    pub last_seen: u64,

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
    /// DEPRECATED: Use properties["`trust_level`"] instead
    /// This field is kept temporarily for backward compatibility and will be removed
    /// once all data sources migrate to the properties field.
    #[deprecated(note = "Use properties field instead - this will be removed in a future version")]
    #[serde(default)]
    pub trust_level: Option<u8>,

    /// Family ID (genetic lineage)
    ///
    /// DEPRECATED: Use properties["`family_id`"] instead
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
            properties: Properties::new(),
            #[allow(deprecated)]
            trust_level: None,
            #[allow(deprecated)]
            family_id: None,
        }
    }

    /// Migrate deprecated fields to properties
    ///
    /// This method ensures that `trust_level` and `family_id` from the deprecated fields
    /// are copied into the properties field for adapter-based rendering.
    ///
    /// Call this after deserializing from JSON to ensure backward compatibility.
    #[allow(deprecated)]
    pub fn migrate_deprecated_fields(&mut self) {
        use crate::property::PropertyValue;

        // Migrate trust_level if present and not already in properties
        if let Some(trust) = self.trust_level
            && !self.properties.contains_key("trust_level") {
                self.properties.insert(
                    "trust_level".to_string(),
                    PropertyValue::Number(f64::from(trust)),
                );
            }

        // Migrate family_id if present and not already in properties
        if let Some(ref family) = self.family_id
            && !self.properties.contains_key("family_id") {
                self.properties.insert(
                    "family_id".to_string(),
                    PropertyValue::String(family.clone()),
                );
            }
    }

    /// Add trust information to this primal
    ///
    /// DEPRECATED: Populate the `properties` field directly instead:
    /// ```ignore
    /// info.properties.insert("trust_level".to_string(), PropertyValue::Number(2.0));
    /// info.properties.insert("family_id".to_string(), PropertyValue::String("family-abc".to_string()));
    /// ```
    #[deprecated(note = "Use properties field directly instead")]
    #[must_use]
    #[allow(deprecated)]
    pub fn with_trust(mut self, trust_level: u8, family_id: Option<String>) -> Self {
        self.trust_level = Some(trust_level);
        self.family_id = family_id.clone();

        // Also populate properties for forward compatibility
        use crate::property::PropertyValue;
        self.properties.insert(
            "trust_level".to_string(),
            PropertyValue::Number(f64::from(trust_level)),
        );
        if let Some(fid) = family_id {
            self.properties
                .insert("family_id".to_string(), PropertyValue::String(fid));
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
    /// Type of relationship (e.g., `api_call`, `capability`)
    pub edge_type: String,
    /// Optional label
    pub label: Option<String>,
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
