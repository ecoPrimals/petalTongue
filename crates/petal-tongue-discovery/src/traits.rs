// SPDX-License-Identifier: AGPL-3.0-only
//! Traits for visualization data providers
//!
//! Any primal can implement `VisualizationDataProvider` to provide data
//! to petalTongue. No hardcoded knowledge of specific primals required!

use crate::errors::DiscoveryResult;
use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};

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
/// - Songbird (discovery primal)
/// - Custom aggregator
/// - Multiple providers
///
/// We only care that they provide the data we need!
#[async_trait]
pub trait VisualizationDataProvider: Send + Sync {
    /// Get list of discovered primals
    ///
    /// This is the core capability - providing the list of primals
    /// in the ecosystem.
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>>;

    /// Get topology edges (connections between primals)
    ///
    /// Optional - if not implemented, petalTongue will infer topology
    /// from primal capabilities.
    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        // Default: empty topology (will be inferred)
        Ok(Vec::new())
    }

    /// Health check - verify provider is available
    async fn health_check(&self) -> DiscoveryResult<String>;

    /// Get provider metadata
    ///
    /// Returns information about this provider for logging/debugging.
    /// The name is for display ONLY - never use it for routing logic!
    fn get_metadata(&self) -> ProviderMetadata;
}

// Keep DiscoveredProvider as an alias for backward compatibility
#[expect(dead_code)] // Kept for backward compatibility but not actively used
pub type DiscoveredProvider = ProviderMetadata;

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

    #[async_trait::async_trait]
    impl VisualizationDataProvider for MockProvider {
        async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
            Ok(vec![])
        }

        async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
            Ok(vec![])
        }

        async fn health_check(&self) -> DiscoveryResult<String> {
            Ok("ok".to_string())
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
        assert_eq!(health, "ok");
        let meta = provider.get_metadata();
        assert_eq!(meta.name, "Mock");
    }
}
