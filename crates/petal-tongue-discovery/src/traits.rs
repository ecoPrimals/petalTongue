// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traits for visualization data providers
//!
//! Any primal can implement `VisualizationDataProvider` to provide data
//! to petalTongue. No hardcoded knowledge of specific primals required!

use crate::errors::DiscoveryResult;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::future::Future;

/// Provider metadata for display and debugging
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    /// Provider name (for display only, never for logic!)
    pub name: String,
    /// Provider endpoint URL
    pub endpoint: String,
    /// Protocol (http, tarpc, grpc, etc.)
    pub protocol: String,
    /// Capabilities this provider offers
    pub capabilities: Vec<String>,
}

/// Generic trait for any primal that provides visualization data
///
/// # Philosophy
///
/// This trait is capability-based. We don't care if the provider is:
/// - biomeOS (orchestrator)
/// - A discovery/registry provider
/// - Custom aggregator
/// - Multiple providers
///
/// We only care that they provide the data we need!
pub trait VisualizationDataProvider: Send + Sync {
    /// Get list of discovered primals
    ///
    /// This is the core capability - providing the list of primals
    /// in the ecosystem.
    fn get_primals(&self) -> impl Future<Output = DiscoveryResult<Vec<PrimalInfo>>> + Send;

    /// Get topology edges (connections between primals)
    ///
    /// Optional - if not implemented, petalTongue will infer topology
    /// from primal capabilities.
    fn get_topology(&self) -> impl Future<Output = DiscoveryResult<Vec<TopologyEdge>>> + Send {
        async { Ok(Vec::new()) }
    }

    /// Health check - verify provider is available
    fn health_check(&self) -> impl Future<Output = DiscoveryResult<String>> + Send;

    /// Get provider metadata
    ///
    /// Returns information about this provider for logging/debugging.
    /// The name is for display ONLY - never use it for routing logic!
    fn get_metadata(&self) -> ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata_creation() {
        let meta = ProviderMetadata {
            name: "Test Provider".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            protocol: "http".to_string(),
            capabilities: vec!["primals".to_string(), "topology".to_string()],
        };
        assert_eq!(meta.name, "Test Provider");
        assert_eq!(meta.endpoint, "http://localhost:8080");
        assert_eq!(meta.capabilities.len(), 2);
    }

    #[test]
    fn provider_metadata_clone() {
        let meta = ProviderMetadata {
            name: "A".to_string(),
            endpoint: "e".to_string(),
            protocol: "p".to_string(),
            capabilities: vec![],
        };
        let cloned = meta.clone();
        assert_eq!(cloned.name, meta.name);
    }

    struct MockProvider {
        metadata: ProviderMetadata,
    }

    impl VisualizationDataProvider for MockProvider {
        fn get_primals(&self) -> impl Future<Output = DiscoveryResult<Vec<PrimalInfo>>> + Send {
            async { Ok(vec![]) }
        }

        fn get_topology(&self) -> impl Future<Output = DiscoveryResult<Vec<TopologyEdge>>> + Send {
            async { Ok(vec![]) }
        }

        fn health_check(&self) -> impl Future<Output = DiscoveryResult<String>> + Send {
            let meta = self.metadata.clone();
            async move { Ok(meta.name) }
        }

        fn get_metadata(&self) -> ProviderMetadata {
            self.metadata.clone()
        }
    }

    #[tokio::test]
    async fn mock_provider_trait_impl() {
        let provider = MockProvider {
            metadata: ProviderMetadata {
                name: "Mock".to_string(),
                endpoint: "mock://".to_string(),
                protocol: "mock".to_string(),
                capabilities: vec![],
            },
        };
        let primals = provider.get_primals().await.unwrap();
        assert!(primals.is_empty());
        let topology = provider.get_topology().await.unwrap();
        assert!(topology.is_empty());
        let health = provider.health_check().await.unwrap();
        assert_eq!(health, "Mock");
        let meta = provider.get_metadata();
        assert_eq!(meta.name, "Mock");
    }
}
