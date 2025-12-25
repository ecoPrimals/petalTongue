//! `BiomeOS` API Client
//!
//! Connects to BiomeOS/Songbird for live primal discovery and health monitoring.

use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
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
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            mock_mode: false,
        }
    }

    /// Enable mock mode (for development/testing)
    #[must_use]
    pub fn with_mock_mode(mut self, enabled: bool) -> Self {
        self.mock_mode = enabled;
        self
    }

    /// Discover primals from `BiomeOS`/Songbird
    pub async fn discover_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        if self.mock_mode {
            return Ok(self.mock_discover_primals());
        }

        // Try to query BiomeOS discovery endpoint
        let url = format!("{}/api/v1/primals", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                let discovery: DiscoveryResponse = response.json().await?;
                Ok(discovery.primals.into_iter().map(|p| p.into()).collect())
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to BiomeOS at {}: {}. Using mock data.",
                    url,
                    e
                );
                // Fallback to mock data
                Ok(self.mock_discover_primals())
            }
        }
    }

    /// Get topology edges (connections between primals)
    pub async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        if self.mock_mode {
            return Ok(self.mock_topology());
        }

        let url = format!("{}/api/v1/topology", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                let edges: Vec<TopologyEdge> = response.json().await?;
                Ok(edges)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to get topology from BiomeOS: {}. Using mock data.",
                    e
                );
                Ok(self.mock_topology())
            }
        }
    }

    /// Mock primal discovery (for development)
    fn mock_discover_primals(&self) -> Vec<PrimalInfo> {
        vec![
            PrimalInfo {
                id: "beardog-1".to_string(),
                name: "BearDog Security".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://localhost:8001".to_string(),
                capabilities: vec![
                    "authentication".to_string(),
                    "authorization".to_string(),
                    "encryption".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "toadstool-1".to_string(),
                name: "ToadStool Compute".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://localhost:8002".to_string(),
                capabilities: vec![
                    "container_runtime".to_string(),
                    "workload_execution".to_string(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "songbird-1".to_string(),
                name: "Songbird Discovery".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "http://localhost:8003".to_string(),
                capabilities: vec![
                    "service_discovery".to_string(),
                    "capability_matching".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "nestgate-1".to_string(),
                name: "NestGate Storage".to_string(),
                primal_type: "Storage".to_string(),
                endpoint: "http://localhost:8004".to_string(),
                capabilities: vec![
                    "permanent_storage".to_string(),
                    "content_addressing".to_string(),
                    "attribution".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "squirrel-1".to_string(),
                name: "Squirrel AI".to_string(),
                primal_type: "AI".to_string(),
                endpoint: "http://localhost:8005".to_string(),
                capabilities: vec!["intent_parsing".to_string(), "task_planning".to_string()],
                health: PrimalHealthStatus::Critical,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
        ]
    }

    /// Mock topology (for development)
    fn mock_topology(&self) -> Vec<TopologyEdge> {
        vec![
            TopologyEdge {
                from: "beardog-1".to_string(),
                to: "toadstool-1".to_string(),
                edge_type: "authenticates".to_string(),
                label: Some("Auth Flow".to_string()),
            },
            TopologyEdge {
                from: "songbird-1".to_string(),
                to: "beardog-1".to_string(),
                edge_type: "discovers".to_string(),
                label: None,
            },
            TopologyEdge {
                from: "toadstool-1".to_string(),
                to: "nestgate-1".to_string(),
                edge_type: "stores_to".to_string(),
                label: Some("Data Flow".to_string()),
            },
            TopologyEdge {
                from: "squirrel-1".to_string(),
                to: "songbird-1".to_string(),
                edge_type: "queries".to_string(),
                label: None,
            },
            TopologyEdge {
                from: "squirrel-1".to_string(),
                to: "toadstool-1".to_string(),
                edge_type: "orchestrates".to_string(),
                label: Some("Task Execution".to_string()),
            },
        ]
    }
}

/// Convert `DiscoveredPrimal` to `PrimalInfo`
impl From<DiscoveredPrimal> for PrimalInfo {
    fn from(primal: DiscoveredPrimal) -> Self {
        Self {
            id: primal.id,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_mode() {
        let client = BiomeOSClient::new("http://localhost:9000").with_mock_mode(true);

        let primals = client.discover_primals().await.unwrap();
        assert_eq!(primals.len(), 5);
        assert_eq!(primals[0].id, "beardog-1");

        let topology = client.get_topology().await.unwrap();
        assert_eq!(topology.len(), 5);
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
