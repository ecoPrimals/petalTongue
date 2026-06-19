// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline visualization data provider — test/sandbox only
//!
//! **ISOLATED**: Compiled only when `test-fixtures` feature is enabled or during
//! tests. Production builds (default) do NOT include this code.
//!
//! Used for:
//! - `cargo test` (tests instantiate `OfflineVisualizationProvider` directly)
//! - `--features offline-demo` in petal-tongue-ui (degraded fallback when no
//!   live providers are found)
//!
//! Never used in the production discovery path — `discover_visualization_providers()`
//! returns an empty vec when no real providers are found. `OfflineVisualizationProvider`
//! is injected only by init code when the `offline-demo` feature is enabled.

use crate::errors::DiscoveryResult;
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue, TopologyEdge};

/// Offline capability provider for degraded visualization data
///
/// Serves last-known-good / sample data when the ecosystem is unreachable.
/// Metadata and health checks clearly indicate offline/degraded state — this
/// provider never pretends to be a live discovery source.
pub struct OfflineVisualizationProvider;

impl OfflineVisualizationProvider {
    /// Create a new offline visualization provider
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Whether this provider is operating in an offline/degraded capacity
    #[must_use]
    pub const fn is_offline(&self) -> bool {
        true
    }
}

impl Default for OfflineVisualizationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationDataProvider for OfflineVisualizationProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        #[expect(
            clippy::cast_sign_loss,
            reason = "Unix timestamp for current time is always non-negative"
        )]
        let now = chrono::Utc::now().timestamp() as u64;

        // Sample data — capability-domain identifiers, clearly offline
        let mut security_props = Properties::new();
        security_props.insert("trust_level".to_owned(), PropertyValue::Number(3.0));
        security_props.insert(
            "family_id".to_owned(),
            PropertyValue::String("offline-sample".to_owned()),
        );

        let mut discovery_props = Properties::new();
        discovery_props.insert("trust_level".to_owned(), PropertyValue::Number(2.0));
        discovery_props.insert(
            "family_id".to_owned(),
            PropertyValue::String("offline-sample".to_owned()),
        );

        let mut compute_props = Properties::new();
        compute_props.insert("trust_level".to_owned(), PropertyValue::Number(1.0));

        Ok(vec![
            PrimalInfo {
                id: "offline-security-1".into(),
                name: "Security Provider (offline sample)".to_owned(),
                primal_type: "Security".to_owned(),
                endpoint: "capability://security.trust:offline".to_owned(),
                capabilities: vec!["security.trust".to_owned(), "security.identity".to_owned()],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: security_props,
            },
            PrimalInfo {
                id: "offline-discovery-1".into(),
                name: "Discovery Provider (offline sample)".to_owned(),
                primal_type: "Discovery".to_owned(),
                endpoint: "capability://discovery.primals:offline".to_owned(),
                capabilities: vec![
                    "discovery.primals".to_owned(),
                    "orchestration.federation".to_owned(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: discovery_props,
            },
            PrimalInfo {
                id: "offline-compute-1".into(),
                name: "Compute Provider (offline sample)".to_owned(),
                primal_type: "Compute".to_owned(),
                endpoint: "capability://compute.container:offline".to_owned(),
                capabilities: vec![
                    "compute.container".to_owned(),
                    "compute.workload".to_owned(),
                ],
                health: PrimalHealthStatus::Critical,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: compute_props,
            },
        ])
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        Ok(vec![
            TopologyEdge {
                from: "offline-security-1".into(),
                to: "offline-discovery-1".into(),
                edge_type: "trust".to_owned(),
                capability: None,
                metrics: None,
                label: Some("Cached".to_owned()),
            },
            TopologyEdge {
                from: "offline-discovery-1".into(),
                to: "offline-compute-1".into(),
                edge_type: "orchestrates".to_owned(),
                label: None,
                capability: None,
                metrics: None,
            },
        ])
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Offline Visualization Provider (degraded)".to_owned(),
            endpoint: "offline://local".to_owned(),
            protocol: "offline".to_owned(),
            capabilities: vec!["visualization.offline".to_owned()],
        }
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        Ok("degraded: ecosystem unavailable — serving offline sample data".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_offline_provider() {
        let provider = OfflineVisualizationProvider::new();
        assert!(provider.is_offline());

        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 3);
        assert_eq!(primals[0].id, "offline-security-1");
        assert_eq!(primals[0].trust_level(), Some(3));
        assert_eq!(primals[1].trust_level(), Some(2));

        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology.len(), 2);

        let metadata = provider.get_metadata();
        assert_eq!(metadata.protocol, "offline");
    }

    #[test]
    fn offline_provider_default_equals_new() {
        let a = OfflineVisualizationProvider::new();
        let b = OfflineVisualizationProvider;
        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
    }

    #[test]
    fn offline_provider_default_impl() {
        let provider = OfflineVisualizationProvider;
        assert_eq!(
            std::mem::size_of_val(&provider),
            std::mem::size_of_val(&OfflineVisualizationProvider::new())
        );
    }

    #[tokio::test]
    async fn offline_provider_health_check_indicates_degraded() {
        let provider = OfflineVisualizationProvider::new();
        let health = provider.health_check().await.unwrap();
        assert!(health.contains("degraded"));
        assert!(health.contains("offline"));
    }

    #[tokio::test]
    async fn offline_provider_metadata_fields() {
        let provider = OfflineVisualizationProvider::new();
        let meta = provider.get_metadata();
        assert_eq!(meta.endpoint, "offline://local");
        assert_eq!(meta.protocol, "offline");
        assert!(!meta.capabilities.is_empty());
    }

    #[tokio::test]
    async fn offline_provider_topology_structure() {
        let provider = OfflineVisualizationProvider::new();
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology[0].from, "offline-security-1");
        assert_eq!(topology[0].to, "offline-discovery-1");
        assert_eq!(topology[0].edge_type, "trust");
        assert_eq!(topology[0].label, Some("Cached".to_owned()));
        assert_eq!(topology[1].label, None);
    }

    #[tokio::test]
    async fn offline_provider_primals_third_has_no_family_id() {
        let provider = OfflineVisualizationProvider::new();
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals[2].id, "offline-compute-1");
        assert_eq!(primals[2].family_id(), None);
    }
}
