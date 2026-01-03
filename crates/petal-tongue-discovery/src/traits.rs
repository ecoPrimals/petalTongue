//! Traits for visualization data providers
//!
//! Any primal can implement `VisualizationDataProvider` to provide data
//! to petalTongue. No hardcoded knowledge of specific primals required!

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
    async fn get_primals(&self) -> anyhow::Result<Vec<PrimalInfo>>;

    /// Get topology edges (connections between primals)
    ///
    /// Optional - if not implemented, petalTongue will infer topology
    /// from primal capabilities.
    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        // Default: empty topology (will be inferred)
        Ok(Vec::new())
    }

    /// Health check - verify provider is available
    async fn health_check(&self) -> anyhow::Result<String>;

    /// Get provider metadata
    ///
    /// Returns information about this provider for logging/debugging.
    /// The name is for display ONLY - never use it for routing logic!
    fn get_metadata(&self) -> ProviderMetadata;
}

// Keep DiscoveredProvider as an alias for backward compatibility
pub type DiscoveredProvider = ProviderMetadata;
