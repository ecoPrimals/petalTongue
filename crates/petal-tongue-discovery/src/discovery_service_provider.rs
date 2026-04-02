// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovery service visualization provider
//!
//! Wraps `DiscoveryServiceClient` to implement the `VisualizationDataProvider` trait.
//! This allows any ecosystem discovery service to be used as a first-class provider.

use crate::discovery_service_client::DiscoveryServiceClient;
use crate::errors::DiscoveryResult;
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Discovery service visualization provider
///
/// Queries the ecosystem discovery service for the complete primal registry,
/// providing a comprehensive view of the ecosystem topology.
pub struct DiscoveryServiceProvider {
    /// Underlying discovery service client
    client: Arc<RwLock<DiscoveryServiceClient>>,
    /// Cached metadata
    metadata: ProviderMetadata,
}

impl DiscoveryServiceProvider {
    /// Create a new Songbird visualization provider
    ///
    /// Discovers Songbird's Unix socket and wraps it in a provider.
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the discovery service is not found or health check fails.
    pub async fn discover(family_id: Option<&str>) -> DiscoveryResult<Self> {
        let client = DiscoveryServiceClient::discover(family_id)?;

        // Test connectivity
        client.health_check().await?;

        let socket_path = format!("{}", client.socket_path().display());

        Ok(Self {
            client: Arc::new(RwLock::new(client)),
            metadata: ProviderMetadata {
                name: "Discovery Service Registry".to_string(),
                endpoint: socket_path,
                protocol: "unix+jsonrpc".to_string(),
                capabilities: vec![
                    "primal-discovery".to_string(),
                    "capability-query".to_string(),
                    "registry".to_string(),
                ],
            },
        })
    }

    /// Create from existing client (for testing)
    #[must_use]
    pub fn from_client(client: DiscoveryServiceClient) -> Self {
        let socket_path = format!("{}", client.socket_path().display());

        Self {
            client: Arc::new(RwLock::new(client)),
            metadata: ProviderMetadata {
                name: "Discovery Service Registry".to_string(),
                endpoint: socket_path,
                protocol: "unix+jsonrpc".to_string(),
                capabilities: vec![
                    "primal-discovery".to_string(),
                    "capability-query".to_string(),
                    "registry".to_string(),
                ],
            },
        }
    }
}

#[async_trait]
impl VisualizationDataProvider for DiscoveryServiceProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!("Querying discovery service for all registered primals");
        let client = self.client.read().await;
        client.get_all_primals().await
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        // Songbird provides primal list, but topology edges need to be inferred
        // from capabilities and connections. For now, return empty and let
        // petalTongue infer topology from capabilities.
        debug!(
            "Topology inference from capabilities (discovery service doesn't provide edges yet)"
        );
        Ok(Vec::new())
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        let client = self.client.read().await;
        client.health_check().await
    }

    fn get_metadata(&self) -> ProviderMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_create_from_client() {
        let client =
            DiscoveryServiceClient::with_socket_path(PathBuf::from("/tmp/test-songbird.sock"));
        let provider = DiscoveryServiceProvider::from_client(client);

        let metadata = provider.get_metadata();
        assert_eq!(metadata.name, "Discovery Service Registry");
        assert_eq!(metadata.protocol, "unix+jsonrpc");
        assert!(
            metadata
                .capabilities
                .contains(&"primal-discovery".to_string())
        );
    }

    #[test]
    fn test_metadata_contains_required_capabilities() {
        let client = DiscoveryServiceClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let provider = DiscoveryServiceProvider::from_client(client);

        let metadata = provider.get_metadata();
        assert!(
            metadata
                .capabilities
                .contains(&"primal-discovery".to_string())
        );
        assert!(
            metadata
                .capabilities
                .contains(&"capability-query".to_string())
        );
        assert!(metadata.capabilities.contains(&"registry".to_string()));
    }
}
