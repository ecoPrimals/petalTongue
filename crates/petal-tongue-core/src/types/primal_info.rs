// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal discovery and metadata types.

use crate::property::Properties;
use serde::{Deserialize, Serialize};

use super::PrimalId;
use super::health::PrimalHealthStatus;

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
