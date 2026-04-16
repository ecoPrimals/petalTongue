// SPDX-License-Identifier: AGPL-3.0-or-later
//! Enum dispatch for [`VisualizationDataProvider`] (replaces `Box<dyn …>` with RPITIT).

#[cfg(any(test, feature = "test-fixtures"))]
use crate::DemoVisualizationProvider;
use crate::errors::DiscoveryResult;
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use crate::{
    DiscoveryServiceProvider, DynamicScenarioProvider, JsonRpcProvider, MdnsVisualizationProvider,
    NeuralApiProvider, ScenarioVisualizationProvider,
};
use petal_tongue_core::{PrimalInfo, TopologyEdge};

/// Built-in visualization providers discovered by this crate (enum dispatch for RPITIT).
pub enum KnownVisualizationProvider {
    /// Neural API (central coordinator).
    Neural(NeuralApiProvider),
    /// Ecosystem discovery service.
    DiscoveryService(DiscoveryServiceProvider),
    /// JSON-RPC over Unix socket.
    JsonRpc(JsonRpcProvider),
    /// mDNS-discovered provider.
    Mdns(MdnsVisualizationProvider),
    /// Dynamic scenario (schema-driven).
    Dynamic(DynamicScenarioProvider),
    /// Static scenario file provider.
    Scenario(ScenarioVisualizationProvider),
    /// Demo / tutorial data.
    #[cfg(any(test, feature = "test-fixtures"))]
    Demo(DemoVisualizationProvider),
    /// Test-only: health check always fails.
    #[cfg(any(test, feature = "test-fixtures"))]
    FailingHealth(FailingHealthCheckProvider),
    /// Health check never completes (chaos / timeout tests; not for production wiring).
    #[cfg(any(test, feature = "test-fixtures"))]
    HangHealth(HangHealthCheckProvider),
}

/// Intentionally failing provider for unit tests (`concurrent`, etc.).
/// Always fails [`VisualizationDataProvider::health_check`] (for tests).
#[cfg(any(test, feature = "test-fixtures"))]
#[derive(Debug, Clone, Copy)]
pub struct FailingHealthCheckProvider;

#[cfg(any(test, feature = "test-fixtures"))]
#[derive(Debug, Clone, Copy)]
pub struct HangHealthCheckProvider;

#[cfg(any(test, feature = "test-fixtures"))]
impl VisualizationDataProvider for HangHealthCheckProvider {
    fn get_primals(&self) -> impl Future<Output = DiscoveryResult<Vec<PrimalInfo>>> + Send {
        async { Ok(vec![]) }
    }

    fn health_check(&self) -> impl Future<Output = DiscoveryResult<String>> + Send {
        async { std::future::pending().await }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "hang".to_string(),
            endpoint: "hang://".to_string(),
            protocol: "test".to_string(),
            capabilities: vec![],
        }
    }
}

#[cfg(any(test, feature = "test-fixtures"))]
impl VisualizationDataProvider for FailingHealthCheckProvider {
    fn get_primals(&self) -> impl Future<Output = DiscoveryResult<Vec<PrimalInfo>>> + Send {
        async { Ok(vec![]) }
    }

    fn health_check(&self) -> impl Future<Output = DiscoveryResult<String>> + Send {
        async {
            Err(crate::errors::DiscoveryError::ConfigError(
                "Intentional failure".to_string(),
            ))
        }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Failing".to_string(),
            endpoint: "fail://".to_string(),
            protocol: "fail".to_string(),
            capabilities: vec![],
        }
    }
}

impl VisualizationDataProvider for KnownVisualizationProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        match self {
            Self::Neural(p) => p.get_primals().await,
            Self::DiscoveryService(p) => p.get_primals().await,
            Self::JsonRpc(p) => p.get_primals().await,
            Self::Mdns(p) => p.get_primals().await,
            Self::Dynamic(p) => p.get_primals().await,
            Self::Scenario(p) => p.get_primals().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Demo(p) => p.get_primals().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::FailingHealth(p) => p.get_primals().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::HangHealth(p) => p.get_primals().await,
        }
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        match self {
            Self::Neural(p) => p.get_topology().await,
            Self::DiscoveryService(p) => p.get_topology().await,
            Self::JsonRpc(p) => p.get_topology().await,
            Self::Mdns(p) => p.get_topology().await,
            Self::Dynamic(p) => p.get_topology().await,
            Self::Scenario(p) => p.get_topology().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Demo(p) => p.get_topology().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::FailingHealth(p) => p.get_topology().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::HangHealth(p) => p.get_topology().await,
        }
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        match self {
            Self::Neural(p) => p.health_check().await,
            Self::DiscoveryService(p) => p.health_check().await,
            Self::JsonRpc(p) => p.health_check().await,
            Self::Mdns(p) => p.health_check().await,
            Self::Dynamic(p) => p.health_check().await,
            Self::Scenario(p) => p.health_check().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Demo(p) => p.health_check().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::FailingHealth(p) => p.health_check().await,
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::HangHealth(p) => p.health_check().await,
        }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        match self {
            Self::Neural(p) => p.get_metadata(),
            Self::DiscoveryService(p) => p.get_metadata(),
            Self::JsonRpc(p) => p.get_metadata(),
            Self::Mdns(p) => p.get_metadata(),
            Self::Dynamic(p) => p.get_metadata(),
            Self::Scenario(p) => p.get_metadata(),
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::Demo(p) => p.get_metadata(),
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::FailingHealth(p) => p.get_metadata(),
            #[cfg(any(test, feature = "test-fixtures"))]
            Self::HangHealth(p) => p.get_metadata(),
        }
    }
}
