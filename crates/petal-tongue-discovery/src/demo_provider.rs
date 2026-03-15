// SPDX-License-Identifier: AGPL-3.0-only
//! Demo visualization data provider - Test/sandbox only
//!
//! **ISOLATED**: This module is only compiled when `test-fixtures` feature is enabled
//! or when running tests. Production builds (default) do NOT include this code.
//!
//! Used for:
//! - `cargo test` (tests instantiate DemoVisualizationProvider directly)
//! - `--features mock` in petal-tongue-ui (graceful fallback when no providers found)
//!
//! Never used in production discovery path—`discover_visualization_providers()` returns
//! empty vec when no real providers found; DemoVisualizationProvider is only injected
//! by init code when mock feature is enabled.

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue, TopologyEdge};

/// Demo provider for development and graceful fallback
///
/// Returns hardcoded demo data without network calls. Used when no real
/// discovery providers are available (tutorial mode, offline demos).
pub struct DemoVisualizationProvider;

impl DemoVisualizationProvider {
    /// Create a new demo provider
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for DemoVisualizationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VisualizationDataProvider for DemoVisualizationProvider {
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
            PropertyValue::String("demo-family".to_string()),
        );

        let mut songbird_props = Properties::new();
        songbird_props.insert("trust_level".to_string(), PropertyValue::Number(2.0));
        songbird_props.insert(
            "family_id".to_string(),
            PropertyValue::String("demo-family".to_string()),
        );

        let mut toadstool_props = Properties::new();
        toadstool_props.insert("trust_level".to_string(), PropertyValue::Number(1.0));

        Ok(vec![
            PrimalInfo {
                id: "demo-beardog-1".into(),
                name: "BearDog Security (Demo)".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://demo-beardog:9000".to_string(),
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
                family_id: Some("demo-family".to_string()),
            },
            PrimalInfo {
                id: "demo-songbird-1".into(),
                name: "Songbird Discovery (Demo)".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "http://demo-songbird:8080".to_string(),
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
                family_id: Some("demo-family".to_string()),
            },
            PrimalInfo {
                id: "demo-toadstool-1".into(),
                name: "ToadStool Compute (Demo)".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://demo-toadstool:8002".to_string(),
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
                from: "demo-beardog-1".into(),
                to: "demo-songbird-1".into(),
                edge_type: "trust".to_string(),
                capability: None,
                metrics: None,
                label: Some("Trusted".to_string()),
            },
            TopologyEdge {
                from: "demo-songbird-1".into(),
                to: "demo-toadstool-1".into(),
                edge_type: "orchestrates".to_string(),
                label: None,
                capability: None,
                metrics: None,
            },
        ])
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Demo Provider".to_string(),
            endpoint: "demo://local".to_string(),
            protocol: "demo".to_string(),
            capabilities: vec![],
        }
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        Ok("Demo provider is always healthy".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_provider() {
        let provider = DemoVisualizationProvider::new();

        // Test primal discovery
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 3);
        assert_eq!(primals[0].id, "demo-beardog-1");
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
        assert_eq!(metadata.name, "Demo Provider");
    }

    #[test]
    fn demo_provider_default_equals_new() {
        let a = DemoVisualizationProvider::new();
        let b = DemoVisualizationProvider::default();
        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
    }

    #[tokio::test]
    async fn demo_provider_health_check() {
        let provider = DemoVisualizationProvider::new();
        let health = provider.health_check().await.unwrap();
        assert!(!health.is_empty());
        assert!(health.contains("healthy"));
    }

    #[tokio::test]
    async fn demo_provider_metadata_fields() {
        let provider = DemoVisualizationProvider::new();
        let meta = provider.get_metadata();
        assert_eq!(meta.endpoint, "demo://local");
        assert_eq!(meta.protocol, "demo");
        assert!(meta.capabilities.is_empty());
    }

    #[tokio::test]
    async fn demo_provider_topology_structure() {
        let provider = DemoVisualizationProvider::new();
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology[0].from, "demo-beardog-1");
        assert_eq!(topology[0].to, "demo-songbird-1");
        assert_eq!(topology[0].edge_type, "trust");
        assert_eq!(topology[0].label, Some("Trusted".to_string()));
        assert_eq!(topology[1].label, None);
    }

    #[tokio::test]
    async fn demo_provider_primals_third_has_no_family_id() {
        let provider = DemoVisualizationProvider::new();
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals[2].id, "demo-toadstool-1");
        assert!(primals[2].properties.get("family_id").is_none());
    }
}
