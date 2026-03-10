// SPDX-License-Identifier: AGPL-3.0-only
//! Mock visualization data provider for development/testing
//!
//! **ISOLATED**: This module is only compiled when `test-fixtures` feature is enabled
//! or when running tests. Production builds (default) do NOT include this code.

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue, TopologyEdge};

/// Mock provider for development and testing
///
/// Returns hardcoded test data without any network calls.
pub struct MockVisualizationProvider;

impl MockVisualizationProvider {
    /// Create a new mock provider
    #[must_use]
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
        #[expect(
            clippy::cast_sign_loss,
            reason = "Unix timestamp for current time is always non-negative"
        )]
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
                id: "mock-beardog-1".into(),
                name: "BearDog Security (Mock)".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://mock-beardog:9000".to_string(),
                capabilities: vec![
                    "security.trust".to_string(),
                    "security.identity".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: beardog_props,
                #[expect(deprecated)]
                trust_level: Some(3), // Keep for backward compatibility
                #[expect(deprecated)]
                family_id: Some("mock-family".to_string()),
            },
            PrimalInfo {
                id: "mock-songbird-1".into(),
                name: "Songbird Discovery (Mock)".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "http://mock-songbird:8080".to_string(),
                capabilities: vec![
                    "discovery.primals".to_string(),
                    "orchestration.federation".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: songbird_props,
                #[expect(deprecated)]
                trust_level: Some(2),
                #[expect(deprecated)]
                family_id: Some("mock-family".to_string()),
            },
            PrimalInfo {
                id: "mock-toadstool-1".into(),
                name: "ToadStool Compute (Mock)".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://mock-toadstool:8002".to_string(),
                capabilities: vec![
                    "compute.container".to_string(),
                    "compute.workload".to_string(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: toadstool_props,
                #[expect(deprecated)]
                trust_level: Some(1),
                #[expect(deprecated)]
                family_id: None,
            },
        ])
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        Ok(vec![
            TopologyEdge {
                from: "mock-beardog-1".into(),
                to: "mock-songbird-1".into(),
                edge_type: "trust".to_string(),
                capability: None,
                metrics: None,
                label: Some("Trusted".to_string()),
            },
            TopologyEdge {
                from: "mock-songbird-1".into(),
                to: "mock-toadstool-1".into(),
                edge_type: "orchestrates".to_string(),
                label: None,
                capability: None,
                metrics: None,
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
        // Use properties field instead of deprecated trust_level
        assert_eq!(
            primals[0]
                .properties
                .get("trust_level")
                .and_then(petal_tongue_core::PropertyValue::as_u8),
            Some(3)
        );
        assert_eq!(
            primals[1]
                .properties
                .get("trust_level")
                .and_then(petal_tongue_core::PropertyValue::as_u8),
            Some(2)
        );

        // Test topology
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology.len(), 2);

        // Test metadata
        let metadata = provider.get_metadata();
        assert_eq!(metadata.name, "Mock Provider");
    }
}
