// SPDX-License-Identifier: AGPL-3.0-only
//! `BiomeOS` API Client
//!
//! Connects to BiomeOS/Songbird for live primal discovery and health monitoring.

use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// `BiomeOS` API client for live primal discovery
pub struct BiomeOSClient {
    /// `BiomeOS` API base URL
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Enable mock mode for development
    mock_mode: bool,
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
            .timeout(Duration::from_secs(30)) // Increased timeout
            .connect_timeout(Duration::from_secs(10)) // Separate connect timeout
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
            .pool_max_idle_per_host(10) // More idle connections
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive
            .http2_keep_alive_interval(Some(Duration::from_secs(30))) // HTTP/2 keep-alive
            .http2_keep_alive_timeout(Duration::from_secs(10))
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
            mock_mode: false,
        }
    }

    /// Enable mock mode (for development/testing)
    #[must_use]
    pub const fn with_mock_mode(mut self, enabled: bool) -> Self {
        self.mock_mode = enabled;
        self
    }

    /// Check if `BiomeOS` API is available
    pub async fn health_check(&self) -> anyhow::Result<bool> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.mock_mode {
            return Ok(true); // Mock mode is always "healthy" (test/dev only)
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.mock_mode {
            anyhow::bail!(
                "Mock mode requires test-fixtures feature. Use real biomeOS connection or build with --features test-fixtures."
            );
        }

        let url = format!("{}/api/v1/health", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .map_or_else(|_| Ok(false), |response| Ok(response.status().is_success()))
    }

    /// Discover primals from `BiomeOS`/Songbird
    ///
    /// **PRODUCTION MODE**: Returns error if API fails (no mock fallback)
    /// **TEST MODE**: Set `mock_mode` to use test data
    pub async fn discover_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.mock_mode {
            tracing::warn!("Mock mode enabled - using test data (TESTING ONLY)");
            return Ok(self.mock_discover_primals());
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.mock_mode {
            anyhow::bail!(
                "Mock mode requires test-fixtures feature. Use real biomeOS connection or build with --features test-fixtures."
            );
        }

        // Query BiomeOS discovery endpoint
        let url = format!("{}/api/v1/primals", self.base_url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to connect to biomeOS at {url}: {e}\n\
                \n\
                Troubleshooting:\n\
                - Ensure biomeOS API server is running\n\
                - Check BIOMEOS_URL environment variable\n\
                - Verify network connectivity\n\
                - Check firewall settings"
            )
        })?;

        if !response.status().is_success() {
            anyhow::bail!(
                "biomeOS API returned error status: {}\n\
                URL: {}\n\
                \n\
                This indicates the biomeOS API server is reachable but returned an error.",
                response.status(),
                url
            );
        }

        let discovery = response.json::<DiscoveryResponse>().await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse biomeOS response: {e}\n\
                \n\
                This may indicate:\n\
                - API format mismatch\n\
                - biomeOS API is not fully implemented\n\
                - Response is not valid JSON\n\
                \n\
                Expected format: {{\"primals\": [...]}}"
            )
        })?;

        tracing::info!(
            "✅ Successfully discovered {} primals from biomeOS",
            discovery.primals.len()
        );

        Ok(discovery.primals.into_iter().map(|p| p.into()).collect())
    }

    /// Get topology edges (connections between primals)
    ///
    /// **PRODUCTION MODE**: Returns error if API fails (no mock fallback)
    /// **TEST MODE**: Set `mock_mode` to use test data
    ///
    /// **Updated**: Now supports biomeOS's new topology format with nodes + edges
    pub async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.mock_mode {
            tracing::warn!("Mock mode enabled - using test topology (TESTING ONLY)");
            return Ok(self.mock_topology());
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.mock_mode {
            anyhow::bail!(
                "Mock mode requires test-fixtures feature. Use real biomeOS connection or build with --features test-fixtures."
            );
        }

        let url = format!("{}/api/v1/topology", self.base_url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            anyhow::anyhow!("Failed to connect to biomeOS topology endpoint at {url}: {e}")
        })?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Topology endpoint returned error status: {}\n\
                URL: {}",
                response.status(),
                url
            );
        }

        // Try new format first (nodes + edges + mode)
        let topology = response.json::<TopologyResponse>().await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse topology response: {e}\n\
                \n\
                Expected format: {{\"nodes\": [...], \"edges\": [...], \"mode\": \"...\"}}"
            )
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

    /// Mock primal discovery (TEST/DEV ONLY - never in production)
    /// Gated behind test-fixtures feature. Production builds return error when `mock_mode` is requested.
    #[cfg(any(test, feature = "test-fixtures"))]
    fn mock_discover_primals(&self) -> Vec<PrimalInfo> {
        let now = chrono::Utc::now().timestamp() as u64;
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
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
            #[expect(deprecated)]
            trust_level: None,
            #[expect(deprecated)]
            family_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_mode() {
        let client = BiomeOSClient::new("http://test-mock:9000").with_mock_mode(true);

        let primals = client.discover_primals().await.unwrap();
        assert_eq!(primals.len(), 5);
        assert_eq!(primals[0].id, "primal-alpha");

        let topology = client.get_topology().await.unwrap();
        assert_eq!(topology.len(), 5);

        // Health check should always succeed in mock mode
        let health = client.health_check().await.unwrap();
        assert!(health);
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let client = BiomeOSClient::new("http://nonexistent:99999");

        let health = client.health_check().await.unwrap();
        assert!(!health);
    }

    #[tokio::test]
    async fn test_convert_discovered_primal() {
        let discovered = DiscoveredPrimal {
            id: "test-1".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://test:8000".to_string(),
            capabilities: vec!["test".to_string()],
            health: "healthy".to_string(),
            last_seen: 1_234_567_890,
        };

        let primal_info: PrimalInfo = discovered.into();
        assert_eq!(primal_info.id, "test-1");
        assert_eq!(primal_info.health, PrimalHealthStatus::Healthy);
    }
}
