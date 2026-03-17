// SPDX-License-Identifier: AGPL-3.0-or-later
//! Demo visualization data provider - Test/sandbox only
//!
//! **ISOLATED**: This module is only compiled when `test-fixtures` feature is enabled
//! or when running tests. Production builds (default) do NOT include this code.
//!
//! Used for:
//! - `cargo test` (tests instantiate `DemoVisualizationProvider` directly)
//! - `--features mock` in petal-tongue-ui (graceful fallback when no providers found)
//!
//! Never used in production discovery path—`discover_visualization_providers()` returns
//! empty vec when no real providers found; `DemoVisualizationProvider` is only injected
//! by init code when mock feature is enabled.

use crate::errors::DiscoveryResult;
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
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        #[expect(
            clippy::cast_sign_loss,
            reason = "Unix timestamp for current time is always non-negative"
        )]
        let now = chrono::Utc::now().timestamp() as u64;

        // Create primals using modern properties approach (capability-domain identifiers)
        let mut security_props = Properties::new();
        security_props.insert("trust_level".to_string(), PropertyValue::Number(3.0));
        security_props.insert(
            "family_id".to_string(),
            PropertyValue::String("demo-family".to_string()),
        );

        let mut discovery_props = Properties::new();
        discovery_props.insert("trust_level".to_string(), PropertyValue::Number(2.0));
        discovery_props.insert(
            "family_id".to_string(),
            PropertyValue::String("demo-family".to_string()),
        );

        let mut compute_props = Properties::new();
        compute_props.insert("trust_level".to_string(), PropertyValue::Number(1.0));

        Ok(vec![
            PrimalInfo {
                id: "demo-security-1".into(),
                name: "Security Provider (Demo)".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "capability://security.trust:demo".to_string(),
                capabilities: vec![
                    "security.trust".to_string(),
                    "security.identity".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: security_props,
                #[expect(deprecated)]
                trust_level: Some(3), // Keep for backward compatibility
                #[expect(deprecated)]
                family_id: Some("demo-family".to_string()),
            },
            PrimalInfo {
                id: "demo-discovery-1".into(),
                name: "Discovery Provider (Demo)".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "capability://discovery.primals:demo".to_string(),
                capabilities: vec![
                    "discovery.primals".to_string(),
                    "orchestration.federation".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: discovery_props,
                #[expect(deprecated)]
                trust_level: Some(2),
                #[expect(deprecated)]
                family_id: Some("demo-family".to_string()),
            },
            PrimalInfo {
                id: "demo-compute-1".into(),
                name: "Compute Provider (Demo)".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "capability://compute.container:demo".to_string(),
                capabilities: vec![
                    "compute.container".to_string(),
                    "compute.workload".to_string(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: compute_props,
                #[expect(deprecated)]
                trust_level: Some(1),
                #[expect(deprecated)]
                family_id: None,
            },
        ])
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        Ok(vec![
            TopologyEdge {
                from: "demo-security-1".into(),
                to: "demo-discovery-1".into(),
                edge_type: "trust".to_string(),
                capability: None,
                metrics: None,
                label: Some("Trusted".to_string()),
            },
            TopologyEdge {
                from: "demo-discovery-1".into(),
                to: "demo-compute-1".into(),
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

    async fn health_check(&self) -> DiscoveryResult<String> {
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
        assert_eq!(primals[0].id, "demo-security-1");
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
        let b = DemoVisualizationProvider;
        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
    }

    #[test]
    fn demo_provider_default_impl() {
        let provider = DemoVisualizationProvider;
        assert_eq!(
            std::mem::size_of_val(&provider),
            std::mem::size_of_val(&DemoVisualizationProvider::new())
        );
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
        assert_eq!(topology[0].from, "demo-security-1");
        assert_eq!(topology[0].to, "demo-discovery-1");
        assert_eq!(topology[0].edge_type, "trust");
        assert_eq!(topology[0].label, Some("Trusted".to_string()));
        assert_eq!(topology[1].label, None);
    }

    #[tokio::test]
    async fn demo_provider_primals_third_has_no_family_id() {
        let provider = DemoVisualizationProvider::new();
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals[2].id, "demo-compute-1");
        assert!(!primals[2].properties.contains_key("family_id"));
    }
}
