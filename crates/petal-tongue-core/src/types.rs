// SPDX-License-Identifier: AGPL-3.0-or-later
//! Core types for petalTongue visualization system

use crate::property::Properties;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::sync::Arc;

/// Zero-copy primal identifier.
///
/// Wraps `Arc<str>` for cheap cloning when IDs are passed around.
/// Implements `Borrow<str>` for `HashMap` lookups and `PartialEq<str>` for comparisons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrimalId(Arc<str>);

impl PrimalId {
    /// Create a new `PrimalId` from any string-like type.
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    /// Get the underlying string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for PrimalId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for PrimalId {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl PartialEq<&str> for PrimalId {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}

impl PartialEq<PrimalId> for str {
    fn eq(&self, other: &PrimalId) -> bool {
        other == self
    }
}

impl std::fmt::Display for PrimalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for PrimalId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PrimalId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s)))
    }
}

impl From<&str> for PrimalId {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<String> for PrimalId {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

/// Well-known property key for trust level (0-3: None, Limited, Elevated, Full)
pub const PROP_TRUST_LEVEL: &str = "trust_level";
/// Well-known property key for family/lineage identifier
pub const PROP_FAMILY_ID: &str = "family_id";

/// Endpoints for different protocols (biomeOS format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Unix socket path (e.g., `"/run/user/1000/biomeos/primal-node-alpha.sock"`)
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
///
/// Trust and family data live in `properties` under well-known keys
/// [`PROP_TRUST_LEVEL`] and [`PROP_FAMILY_ID`]. Use the convenience
/// accessors ([`trust_level()`](Self::trust_level), [`family_id()`](Self::family_id))
/// or builder methods ([`with_trust_level()`](Self::with_trust_level),
/// [`with_family_id()`](Self::with_family_id)) for ergonomic access.
///
/// Deserialization automatically migrates legacy `trust_level` / `family_id`
/// JSON fields into `properties` for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "PrimalInfoWire")]
pub struct PrimalInfo {
    /// Unique identifier for the primal
    pub id: PrimalId,
    /// Human-readable name
    pub name: String,
    /// Type of primal (e.g., "Compute", "Storage", "Security")
    #[serde(alias = "type")]
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

    /// Generic properties for ecosystem-specific data
    ///
    /// This field holds all ecosystem-specific data in a generic format.
    /// Adapters interpret these properties at runtime to provide rich UI.
    ///
    /// Well-known keys: [`PROP_TRUST_LEVEL`], [`PROP_FAMILY_ID`].
    #[serde(default)]
    pub properties: Properties,
}

/// Wire format for backward-compatible deserialization.
///
/// Accepts legacy `trust_level` and `family_id` JSON fields and migrates
/// them into `properties` during conversion to [`PrimalInfo`].
#[derive(Deserialize)]
struct PrimalInfoWire {
    id: PrimalId,
    name: String,
    #[serde(alias = "type")]
    primal_type: String,
    endpoint: String,
    capabilities: Vec<String>,
    health: PrimalHealthStatus,
    last_seen: u64,
    #[serde(default)]
    endpoints: Option<PrimalEndpoints>,
    #[serde(default)]
    metadata: Option<PrimalMetadata>,
    #[serde(default)]
    properties: Properties,
    #[serde(default)]
    trust_level: Option<u8>,
    #[serde(default)]
    family_id: Option<String>,
}

impl From<PrimalInfoWire> for PrimalInfo {
    fn from(wire: PrimalInfoWire) -> Self {
        use crate::property::PropertyValue;

        let mut properties = wire.properties;

        if let Some(trust) = wire.trust_level {
            properties
                .entry(PROP_TRUST_LEVEL.to_string())
                .or_insert_with(|| PropertyValue::Number(f64::from(trust)));
        }
        if let Some(ref family) = wire.family_id {
            properties
                .entry(PROP_FAMILY_ID.to_string())
                .or_insert_with(|| PropertyValue::String(family.clone()));
        }

        if let Some(ref metadata) = wire.metadata {
            if let Some(ref version) = metadata.version {
                properties
                    .entry("version".to_owned())
                    .or_insert_with(|| PropertyValue::String(version.clone()));
            }
            if let Some(ref family) = metadata.family_id {
                properties
                    .entry(PROP_FAMILY_ID.to_string())
                    .or_insert_with(|| PropertyValue::String(family.clone()));
            }
            if let Some(ref node_id) = metadata.node_id {
                properties
                    .entry("node_id".to_owned())
                    .or_insert_with(|| PropertyValue::String(node_id.clone()));
            }
        }

        let mut info = Self {
            id: wire.id,
            name: wire.name,
            primal_type: wire.primal_type,
            endpoint: wire.endpoint,
            capabilities: wire.capabilities,
            health: wire.health,
            last_seen: wire.last_seen,
            endpoints: wire.endpoints,
            metadata: wire.metadata,
            properties,
        };

        if let Some(ref endpoints) = info.endpoints
            && (info.endpoint.is_empty() || info.endpoint == "unknown")
        {
            if let Some(ref unix_socket) = endpoints.unix_socket {
                info.endpoint = format!("unix://{unix_socket}");
            } else if let Some(ref http) = endpoints.http {
                info.endpoint = http.clone();
            }
        }

        info
    }
}

impl PrimalInfo {
    /// Create a new `PrimalInfo` with basic information.
    ///
    /// Use [`with_trust_level()`](Self::with_trust_level) /
    /// [`with_family_id()`](Self::with_family_id) for ecosystem-specific data.
    #[must_use]
    pub fn new(
        id: impl Into<PrimalId>,
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
        }
    }

    /// Read the trust level from properties.
    #[must_use]
    pub fn trust_level(&self) -> Option<u8> {
        self.properties
            .get(PROP_TRUST_LEVEL)
            .and_then(crate::property::PropertyValue::as_number)
            .map(|n| {
                #[expect(clippy::cast_sign_loss, reason = "trust level is always 0-3")]
                let level = n as u8;
                level
            })
    }

    /// Read the family id from properties.
    #[must_use]
    pub fn family_id(&self) -> Option<&str> {
        self.properties
            .get(PROP_FAMILY_ID)
            .and_then(crate::property::PropertyValue::as_string)
    }

    /// Set trust level in properties.
    pub fn set_trust_level(&mut self, level: u8) {
        self.properties.insert(
            PROP_TRUST_LEVEL.to_string(),
            crate::property::PropertyValue::Number(f64::from(level)),
        );
    }

    /// Set family id in properties.
    pub fn set_family_id(&mut self, family_id: impl Into<String>) {
        self.properties.insert(
            PROP_FAMILY_ID.to_string(),
            crate::property::PropertyValue::String(family_id.into()),
        );
    }

    /// Builder: set trust level.
    #[must_use]
    pub fn with_trust_level(mut self, level: u8) -> Self {
        self.set_trust_level(level);
        self
    }

    /// Builder: set family id.
    #[must_use]
    pub fn with_family_id(mut self, family_id: impl Into<String>) -> Self {
        self.set_family_id(family_id);
        self
    }

    /// Migrate biomeOS metadata fields into properties.
    ///
    /// Call after deserializing from JSON when not using serde
    /// (the serde path handles this via the internal `PrimalInfoWire` type).
    pub fn migrate_metadata_to_properties(&mut self) {
        use crate::property::PropertyValue;

        if let Some(ref metadata) = self.metadata {
            if let Some(ref version) = metadata.version {
                self.properties
                    .entry("version".to_owned())
                    .or_insert_with(|| PropertyValue::String(version.clone()));
            }
            if let Some(ref family) = metadata.family_id {
                self.properties
                    .entry(PROP_FAMILY_ID.to_string())
                    .or_insert_with(|| PropertyValue::String(family.clone()));
            }
            if let Some(ref node_id) = metadata.node_id {
                self.properties
                    .entry("node_id".to_owned())
                    .or_insert_with(|| PropertyValue::String(node_id.clone()));
            }
        }

        if let Some(ref endpoints) = self.endpoints
            && (self.endpoint.is_empty() || self.endpoint == "unknown")
        {
            if let Some(ref unix_socket) = endpoints.unix_socket {
                self.endpoint = format!("unix://{unix_socket}");
            } else if let Some(ref http) = endpoints.http {
                self.endpoint = http.clone();
            }
        }
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
    pub from: PrimalId,
    /// Target primal ID
    pub to: PrimalId,
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
    "connection".to_owned()
}

/// Real-time flow event showing message between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    /// Event ID
    pub id: PrimalId,
    /// Source primal ID
    pub from: PrimalId,
    /// Target primal ID
    pub to: PrimalId,
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
    pub from: PrimalId,
    /// Target primal ID
    pub to: PrimalId,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test code")]

    use super::*;
    use std::borrow::Borrow;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_id(id: &PrimalId) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        hasher.finish()
    }

    fn minimal_primal_json(extra: &str) -> String {
        format!(
            r#"{{
                "id": "p1",
                "name": "Test",
                "type": "Compute",
                "endpoint": "http://localhost",
                "capabilities": [],
                "health": "Healthy",
                "last_seen": 0
                {extra}
            }}"#
        )
    }

    #[test]
    fn primal_id_construction_and_traits() {
        let id = PrimalId::new("alpha");
        assert_eq!(id.as_str(), "alpha");
        assert_eq!(format!("{id}"), "alpha");

        let from_ref: PrimalId = "beta".into();
        let from_string: PrimalId = "gamma".to_string().into();
        assert_eq!(from_ref.as_str(), "beta");
        assert_eq!(from_string.as_str(), "gamma");

        let id2 = PrimalId::from("alpha");
        assert_eq!(id, id2);
        assert_eq!(id, "alpha");
        assert_eq!(*"alpha", id);
        assert_ne!(id, "other");

        let borrowed: &str = id.borrow();
        assert_eq!(borrowed, "alpha");

        let cloned = id.clone();
        assert_eq!(hash_id(&id), hash_id(&cloned));

        let json = serde_json::to_string(&id).unwrap();
        let roundtrip: PrimalId = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, id);
    }

    #[test]
    fn primal_health_status_as_str_and_parse() {
        assert_eq!(PrimalHealthStatus::Healthy.as_str(), "Healthy");
        assert_eq!(PrimalHealthStatus::Warning.as_str(), "Warning");
        assert_eq!(PrimalHealthStatus::Critical.as_str(), "Critical");
        assert_eq!(PrimalHealthStatus::Unknown.as_str(), "Unknown");

        assert_eq!(
            PrimalHealthStatus::parse_health_status("Healthy"),
            PrimalHealthStatus::Healthy
        );
        assert_eq!(
            PrimalHealthStatus::parse_health_status("Warning"),
            PrimalHealthStatus::Warning
        );
        assert_eq!(
            PrimalHealthStatus::parse_health_status("Critical"),
            PrimalHealthStatus::Critical
        );
        assert_eq!(
            PrimalHealthStatus::parse_health_status("bogus"),
            PrimalHealthStatus::Unknown
        );
    }

    #[test]
    fn connection_status_serde_roundtrip() {
        for status in [
            ConnectionStatus::Connected,
            ConnectionStatus::Connecting,
            ConnectionStatus::Disconnected,
            ConnectionStatus::Error("timeout".to_string()),
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let roundtrip: ConnectionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(roundtrip, status);
        }
    }

    #[test]
    fn primal_info_new_and_builders() {
        let info = PrimalInfo::new(
            "id-1",
            "Test",
            "compute",
            "http://localhost:8080",
            vec!["cap1".to_string()],
            PrimalHealthStatus::Healthy,
            12345,
        )
        .with_trust_level(2)
        .with_family_id("fam-1");

        assert_eq!(info.trust_level(), Some(2));
        assert_eq!(info.family_id(), Some("fam-1"));
    }

    #[test]
    fn primal_info_wire_migration_legacy_fields() {
        let json = minimal_primal_json(r#","trust_level": 2, "family_id": "legacy-fam""#);
        let info: PrimalInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.trust_level(), Some(2));
        assert_eq!(info.family_id(), Some("legacy-fam"));
        assert_eq!(
            info.properties.get(PROP_TRUST_LEVEL).unwrap().as_number(),
            Some(2.0)
        );
        assert_eq!(
            info.properties.get(PROP_FAMILY_ID).unwrap().as_string(),
            Some("legacy-fam")
        );
    }

    #[test]
    fn primal_info_wire_migration_metadata() {
        let json = minimal_primal_json(
            r#",
                "metadata": {
                    "version": "v0.15.2",
                    "family_id": "meta-fam",
                    "node_id": "node-7"
                }
            "#,
        );
        let info: PrimalInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(
            info.properties.get("version").unwrap().as_string(),
            Some("v0.15.2")
        );
        assert_eq!(info.family_id(), Some("meta-fam"));
        assert_eq!(
            info.properties.get("node_id").unwrap().as_string(),
            Some("node-7")
        );
    }

    #[test]
    fn primal_info_endpoint_fallback_from_unix_socket() {
        let json = r#"{
            "id": "p1",
            "name": "Test",
            "type": "Compute",
            "endpoint": "unknown",
            "capabilities": [],
            "health": "Healthy",
            "last_seen": 0,
            "endpoints": { "unix_socket": "/tmp/primal.sock" }
        }"#;
        let info: PrimalInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.endpoint, "unix:///tmp/primal.sock");
    }

    #[test]
    fn topology_edge_default_edge_type() {
        let json = r#"{"from": "a", "to": "b"}"#;
        let edge: TopologyEdge = serde_json::from_str(json).unwrap();
        assert_eq!(edge.edge_type, "connection");
    }

    #[test]
    fn primal_endpoints_skip_serializing_none() {
        let ep = PrimalEndpoints {
            unix_socket: None,
            http: None,
        };
        let json = serde_json::to_string(&ep).unwrap();
        assert_eq!(json, "{}");
        assert!(!json.contains("unix_socket"));
        assert!(!json.contains("http"));
    }

    #[test]
    fn connection_metrics_optional_fields() {
        let full = ConnectionMetrics {
            request_count: Some(10),
            avg_latency_ms: Some(5.5),
            error_count: Some(1),
        };
        let full_json = serde_json::to_string(&full).unwrap();
        assert!(full_json.contains("request_count"));
        assert!(full_json.contains("avg_latency_ms"));
        assert!(full_json.contains("error_count"));

        let empty = ConnectionMetrics {
            request_count: None,
            avg_latency_ms: None,
            error_count: None,
        };
        let empty_json = serde_json::to_string(&empty).unwrap();
        assert_eq!(empty_json, "{}");
    }

    #[test]
    fn migrate_metadata_to_properties() {
        let mut info = PrimalInfo::new(
            "p1",
            "Test",
            "compute",
            "unknown",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        );
        info.metadata = Some(PrimalMetadata {
            version: Some("v1.0".to_string()),
            family_id: Some("fam-x".to_string()),
            node_id: Some("node-3".to_string()),
        });
        info.endpoints = Some(PrimalEndpoints {
            unix_socket: Some("/run/sock".to_string()),
            http: None,
        });

        info.migrate_metadata_to_properties();

        assert_eq!(
            info.properties.get("version").unwrap().as_string(),
            Some("v1.0")
        );
        assert_eq!(info.family_id(), Some("fam-x"));
        assert_eq!(
            info.properties.get("node_id").unwrap().as_string(),
            Some("node-3")
        );
        assert_eq!(info.endpoint, "unix:///run/sock");
    }

    #[test]
    fn topology_graph_serde_roundtrip() {
        let graph = TopologyGraph {
            nodes: vec![PrimalInfo::new(
                "n1",
                "Node",
                "compute",
                "http://localhost",
                vec![],
                PrimalHealthStatus::Healthy,
                100,
            )],
            edges: vec![TopologyEdge {
                from: PrimalId::from("n1"),
                to: PrimalId::from("n2"),
                edge_type: "connection".to_string(),
                label: None,
                capability: None,
                metrics: None,
            }],
            timestamp: 100,
        };

        let json = serde_json::to_string(&graph).unwrap();
        let roundtrip: TopologyGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.nodes.len(), 1);
        assert_eq!(roundtrip.edges.len(), 1);
        assert_eq!(roundtrip.timestamp, 100);
        assert_eq!(roundtrip.nodes[0].id.as_str(), "n1");
        assert_eq!(roundtrip.edges[0].to.as_str(), "n2");
    }
}
