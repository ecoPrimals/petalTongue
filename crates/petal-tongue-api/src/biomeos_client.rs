// SPDX-License-Identifier: AGPL-3.0-or-later
//! `BiomeOS` API Client
//!
//! Connects to the biomeOS orchestrator for live primal discovery and health monitoring.

use crate::biomeos_error::BiomeOsClientError;
use petal_tongue_core::constants::discovery_timeouts;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, TopologyEdge};
use serde::{Deserialize, Serialize};

/// `BiomeOS` API client for live primal discovery
pub struct BiomeOSClient {
    /// `BiomeOS` API base URL
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Enable fixture mode for development (deterministic data when biomeOS unavailable).
    fixture_mode: bool,
}

/// Response from `BiomeOS` discovery API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    /// List of discovered primals
    pub primals: Vec<DiscoveredPrimal>,
}

/// Response from `BiomeOS` topology API (new format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyResponse {
    /// Topology nodes (enriched with trust levels, family, etc.)
    #[serde(default)]
    pub nodes: Vec<TopologyNode>,
    /// Topology edges (connections between primals)
    pub edges: Vec<TopologyEdge>,
    /// Mode indicator (mock, live, etc.)
    #[serde(default)]
    pub mode: String,
}

/// Topology node (enriched node data from biomeOS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    /// Node ID
    pub id: String,
    /// Node name
    #[serde(default)]
    pub name: String,
    /// Node type
    #[serde(default, rename = "type")]
    pub node_type: String,
    /// Health status
    #[serde(default)]
    pub status: String,
    /// Trust level (0-3)
    #[serde(default)]
    pub trust_level: Option<u8>,
    /// Family ID (genetic lineage)
    #[serde(default)]
    pub family_id: Option<String>,
    /// Capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// A discovered primal from `BiomeOS`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal ID
    pub id: String,
    /// Primal name
    pub name: String,
    /// Primal type
    pub primal_type: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Health status
    pub health: String,
    /// Last seen timestamp (Unix)
    pub last_seen: u64,
}

impl BiomeOSClient {
    /// Create a new `BiomeOS` client
    pub fn new(base_url: impl Into<String>) -> Self {
        // Build HTTP client with robust configuration
        let client = reqwest::Client::builder()
            .timeout(discovery_timeouts::HTTP_TIMEOUT)
            .connect_timeout(discovery_timeouts::HTTP_CONNECT_TIMEOUT)
            .pool_idle_timeout(discovery_timeouts::HTTP_POOL_IDLE_TIMEOUT)
            .pool_max_idle_per_host(10) // More idle connections
            .tcp_keepalive(discovery_timeouts::HTTP_TCP_KEEPALIVE)
            .http2_keep_alive_interval(Some(discovery_timeouts::HTTP2_KEEPALIVE_INTERVAL))
            .http2_keep_alive_timeout(discovery_timeouts::HTTP2_KEEPALIVE_TIMEOUT)
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to build HTTP client with custom config: {}. Using default client.",
                    e
                );
                reqwest::Client::new()
            });

        Self {
            base_url: base_url.into(),
            client,
            fixture_mode: false,
        }
    }

    /// Enable fixture mode (deterministic data when biomeOS unavailable).
    /// Requires `test-fixtures` feature; production builds return
    /// [`BiomeOsClientError::FixtureModeUnavailable`] at runtime.
    #[must_use]
    pub const fn with_fixture_mode(mut self, enabled: bool) -> Self {
        self.fixture_mode = enabled;
        self
    }

    /// Check if `BiomeOS` API is available
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on network failure or when fixture mode is enabled without the feature.
    pub async fn health_check(&self) -> Result<bool, BiomeOsClientError> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.fixture_mode {
            return Ok(true);
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.fixture_mode {
            return Err(BiomeOsClientError::FixtureModeUnavailable);
        }

        let url = format!("{}/api/v1/health", self.base_url);
        Ok(self
            .client
            .get(&url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success()))
    }

    /// Discover primals from the biomeOS orchestrator
    ///
    /// **PRODUCTION MODE**: Returns error if API fails (no fixture fallback)
    /// **FIXTURE MODE**: Set `fixture_mode` for deterministic test data
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on network failure, non-success status, or JSON parse error.
    pub async fn discover_primals(&self) -> Result<Vec<PrimalInfo>, BiomeOsClientError> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.fixture_mode {
            tracing::debug!("fixture mode — returning deterministic primals");
            return Ok(self.mock_discover_primals());
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.fixture_mode {
            return Err(BiomeOsClientError::FixtureModeUnavailable);
        }

        // Query BiomeOS discovery endpoint
        let url = format!("{}/api/v1/primals", self.base_url);

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| BiomeOsClientError::Network {
                    url: url.clone(),
                    source: e,
                })?;

        if !response.status().is_success() {
            return Err(BiomeOsClientError::ServerError {
                status: response.status().as_u16(),
                url,
            });
        }

        let discovery = response.json::<DiscoveryResponse>().await.map_err(|e| {
            BiomeOsClientError::Parse(format!(
                "{e}\n\n\
                This may indicate:\n\
                - API format mismatch\n\
                - biomeOS API is not fully implemented\n\
                - Response is not valid JSON\n\
                \n\
                Expected format: {{\"primals\": [...]}}"
            ))
        })?;

        tracing::info!(
            "✅ Successfully discovered {} primals from biomeOS",
            discovery.primals.len()
        );

        Ok(discovery.primals.into_iter().map(|p| p.into()).collect())
    }

    /// Get topology edges (connections between primals)
    ///
    /// **PRODUCTION MODE**: Returns error if API fails (no fixture fallback)
    /// **FIXTURE MODE**: Set `fixture_mode` for deterministic topology data
    ///
    /// Supports biomeOS's topology format with nodes + edges.
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on network failure, non-success status, or JSON parse error.
    pub async fn get_topology(&self) -> Result<Vec<TopologyEdge>, BiomeOsClientError> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.fixture_mode {
            tracing::debug!("fixture mode — returning deterministic topology");
            return Ok(self.mock_topology());
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.fixture_mode {
            return Err(BiomeOsClientError::FixtureModeUnavailable);
        }

        let url = format!("{}/api/v1/topology", self.base_url);

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| BiomeOsClientError::Network {
                    url: url.clone(),
                    source: e,
                })?;

        if !response.status().is_success() {
            return Err(BiomeOsClientError::ServerError {
                status: response.status().as_u16(),
                url,
            });
        }

        // Try new format first (nodes + edges + mode)
        let topology = response.json::<TopologyResponse>().await.map_err(|e| {
            BiomeOsClientError::Parse(format!(
                "{e}\n\n\
                Expected format: {{\"nodes\": [...], \"edges\": [...], \"mode\": \"...\"}}"
            ))
        })?;

        tracing::debug!(
            "✅ Successfully retrieved topology: {} nodes, {} edges (mode: {})",
            topology.nodes.len(),
            topology.edges.len(),
            topology.mode
        );

        // Returns edges for topology graph construction.
        // Enriched node data (trust, family) is available via topology.nodes when needed.
        Ok(topology.edges)
    }

    /// Fixture primal discovery (test/dev only).
    /// Gated behind `test-fixtures` feature. Production builds return error when `fixture_mode` is set.
    #[cfg(any(test, feature = "test-fixtures"))]
    fn mock_discover_primals(&self) -> Vec<PrimalInfo> {
        let now = chrono::Utc::now().timestamp().cast_unsigned();
        vec![
            PrimalInfo {
                id: "primal-alpha".into(),
                name: "Security Primal".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://mock-primal-alpha:8001".to_string(),
                capabilities: vec![
                    "authentication".to_string(),
                    "authorization".to_string(),
                    "encryption".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(), // Generic properties
            },
            PrimalInfo {
                id: "primal-beta".into(),
                name: "Compute Primal".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://mock-primal-beta:8002".to_string(),
                capabilities: vec![
                    "container_runtime".to_string(),
                    "workload_execution".to_string(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
            PrimalInfo {
                id: "primal-gamma".into(),
                name: "Discovery Primal".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "http://mock-primal-gamma:8003".to_string(),
                capabilities: vec![
                    "service_discovery".to_string(),
                    "capability_matching".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
            PrimalInfo {
                id: "primal-delta".into(),
                name: "Storage Primal".to_string(),
                primal_type: "Storage".to_string(),
                endpoint: "http://mock-primal-delta:8004".to_string(),
                capabilities: vec![
                    "permanent_storage".to_string(),
                    "content_addressing".to_string(),
                    "attribution".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
            PrimalInfo {
                id: "primal-epsilon".into(),
                name: "AI Primal".to_string(),
                primal_type: "AI".to_string(),
                endpoint: "http://mock-primal-epsilon:8005".to_string(),
                capabilities: vec!["intent_parsing".to_string(), "task_planning".to_string()],
                health: PrimalHealthStatus::Critical,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
        ]
    }

    /// Mock topology (TEST/DEV ONLY - never in production)
    /// Gated behind test-fixtures feature.
    #[cfg(any(test, feature = "test-fixtures"))]
    fn mock_topology(&self) -> Vec<TopologyEdge> {
        vec![
            TopologyEdge {
                from: "primal-alpha".into(),
                to: "primal-beta".into(),
                edge_type: "authenticates".to_string(),
                capability: None,
                metrics: None,
                label: Some("Auth Flow".to_string()),
            },
            TopologyEdge {
                from: "primal-gamma".into(),
                to: "primal-alpha".into(),
                edge_type: "discovers".to_string(),
                capability: None,
                metrics: None,
                label: None,
            },
            TopologyEdge {
                from: "primal-beta".into(),
                to: "primal-delta".into(),
                edge_type: "stores_to".to_string(),
                capability: None,
                metrics: None,
                label: Some("Data Flow".to_string()),
            },
            TopologyEdge {
                from: "primal-epsilon".into(),
                to: "primal-gamma".into(),
                edge_type: "queries".to_string(),
                capability: None,
                metrics: None,
                label: None,
            },
            TopologyEdge {
                from: "primal-epsilon".into(),
                to: "primal-beta".into(),
                edge_type: "orchestrates".to_string(),
                capability: None,
                metrics: None,
                label: Some("Task Execution".to_string()),
            },
        ]
    }
}

/// Convert `DiscoveredPrimal` to `PrimalInfo`
impl From<DiscoveredPrimal> for PrimalInfo {
    fn from(primal: DiscoveredPrimal) -> Self {
        Self {
            id: primal.id.into(),
            name: primal.name,
            primal_type: primal.primal_type,
            endpoint: primal.endpoint,
            capabilities: primal.capabilities,
            health: match primal.health.to_lowercase().as_str() {
                "healthy" => PrimalHealthStatus::Healthy,
                "warning" => PrimalHealthStatus::Warning,
                "critical" => PrimalHealthStatus::Critical,
                _ => PrimalHealthStatus::Unknown,
            },
            last_seen: primal.last_seen,
            endpoints: None,
            metadata: None,
            properties: Properties::new(), // Start with empty properties
        }
    }
}

#[cfg(test)]
#[path = "biomeos_client_tests.rs"]
mod tests;
