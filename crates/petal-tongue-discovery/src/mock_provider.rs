//! Mock visualization data provider for development/testing

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue, TopologyEdge};

/// Mock provider for development and testing
///
/// Returns hardcoded test data without any network calls.
pub struct MockVisualizationProvider;

impl MockVisualizationProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockVisualizationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VisualizationDataProvider for MockVisualizationProvider {
    async fn get_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        let now = chrono::Utc::now().timestamp() as u64;

        // Create primals using modern properties approach
        let mut beardog_props = Properties::new();
        beardog_props.insert("trust_level".to_string(), PropertyValue::Number(3.0));
        beardog_props.insert(
            "family_id".to_string(),
            PropertyValue::String("mock-family".to_string()),
        );

        let mut songbird_props = Properties::new();
        songbird_props.insert("trust_level".to_string(), PropertyValue::Number(2.0));
        songbird_props.insert(
            "family_id".to_string(),
            PropertyValue::String("mock-family".to_string()),
        );

        let mut toadstool_props = Properties::new();
        toadstool_props.insert("trust_level".to_string(), PropertyValue::Number(1.0));

        Ok(vec![
            PrimalInfo {
                id: "mock-beardog-1".to_string(),
                name: "BearDog Security (Mock)".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://mock-beardog:9000".to_string(),
                capabilities: vec![
                    "security.trust".to_string(),
                    "security.identity".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                properties: beardog_props,
                #[allow(deprecated)]
                trust_level: Some(3), // Keep for backward compatibility
                #[allow(deprecated)]
                family_id: Some("mock-family".to_string()),
            },
            PrimalInfo {
                id: "mock-songbird-1".to_string(),
                name: "Songbird Discovery (Mock)".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "http://mock-songbird:8080".to_string(),
                capabilities: vec![
                    "discovery.primals".to_string(),
                    "orchestration.federation".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                properties: songbird_props,
                #[allow(deprecated)]
                trust_level: Some(2),
                #[allow(deprecated)]
                family_id: Some("mock-family".to_string()),
            },
            PrimalInfo {
                id: "mock-toadstool-1".to_string(),
                name: "ToadStool Compute (Mock)".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://mock-toadstool:8002".to_string(),
                capabilities: vec![
                    "compute.container".to_string(),
                    "compute.workload".to_string(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                properties: toadstool_props,
                #[allow(deprecated)]
                trust_level: Some(1),
                #[allow(deprecated)]
                family_id: None,
            },
        ])
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        Ok(vec![
            TopologyEdge {
                from: "mock-beardog-1".to_string(),
                to: "mock-songbird-1".to_string(),
                edge_type: "trust".to_string(),
                label: Some("Trusted".to_string()),
            },
            TopologyEdge {
                from: "mock-songbird-1".to_string(),
                to: "mock-toadstool-1".to_string(),
                edge_type: "orchestrates".to_string(),
                label: None,
            },
        ])
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Mock Provider".to_string(), // Fixed: was "MockProvider", now "Mock Provider" to match test
            endpoint: "mock://local".to_string(),
            protocol: "mock".to_string(),
            capabilities: vec![],
        }
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        Ok("Mock provider is always healthy".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockVisualizationProvider::new();

        // Test primal discovery
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 3);
        assert_eq!(primals[0].id, "mock-beardog-1");
        assert_eq!(primals[0].trust_level, Some(3));
        assert_eq!(primals[1].trust_level, Some(2));

        // Test topology
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology.len(), 2);

        // Test metadata
        let metadata = provider.get_metadata();
        assert_eq!(metadata.name, "Mock Provider");
    }
}
