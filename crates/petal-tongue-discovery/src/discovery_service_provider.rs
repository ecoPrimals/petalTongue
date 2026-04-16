// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovery service visualization provider
//!
//! Wraps `DiscoveryServiceClient` to implement the `VisualizationDataProvider` trait.
//! This allows any ecosystem discovery service to be used as a first-class provider.

use crate::discovery_service_client::DiscoveryServiceClient;
use crate::errors::DiscoveryResult;
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Infer topology edges from primals that advertise the same capabilities.
///
/// For each unordered pair of primals, emits one [`TopologyEdge`] per shared capability
/// (`from` / `to` follow discovery list order: lower index → higher index).
fn topology_edges_from_shared_capabilities(primals: &[PrimalInfo]) -> Vec<TopologyEdge> {
    if primals.len() < 2 {
        return Vec::new();
    }

    let mut edges = Vec::new();
    for i in 0..primals.len() {
        let caps_i: HashSet<&str> = primals[i].capabilities.iter().map(String::as_str).collect();
        for j in (i + 1)..primals.len() {
            let caps_j: HashSet<&str> =
                primals[j].capabilities.iter().map(String::as_str).collect();
            for cap in caps_i.intersection(&caps_j) {
                edges.push(TopologyEdge {
                    from: primals[i].id.clone(),
                    to: primals[j].id.clone(),
                    edge_type: "capability".to_string(),
                    label: None,
                    capability: Some((*cap).to_string()),
                    metrics: None,
                });
            }
        }
    }
    edges
}

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
    /// Create a new discovery-service visualization provider
    ///
    /// Locates the discovery registry Unix socket and wraps it in a provider.
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the discovery service is not found or health check fails.
    pub async fn discover(family_id: Option<&str>) -> DiscoveryResult<Self> {
        let client = DiscoveryServiceClient::discover(family_id)?;

        // Test connectivity
        client.health_check().await?;

        let socket_path = client.socket_path().display().to_string();

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
        let socket_path = client.socket_path().display().to_string();

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

impl VisualizationDataProvider for DiscoveryServiceProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!("Querying discovery service for all registered primals");
        let client = self.client.read().await;
        client.get_all_primals().await
    }

    async fn get_topology(
        &self,
    ) -> crate::errors::DiscoveryResult<Vec<petal_tongue_core::TopologyEdge>> {
        let primals = {
            let c = self.client.read().await;
            c.get_all_primals().await
        };
        match primals {
            Ok(primals) => {
                if primals.len() < 2 {
                    debug!("Topology: no edges (fewer than two primals)");
                    return Ok(Vec::new());
                }
                let edges = topology_edges_from_shared_capabilities(&primals);
                debug!(
                    count = edges.len(),
                    primals = primals.len(),
                    "Topology: inferred capability edges from discovery primals"
                );
                Ok(edges)
            }
            Err(e) => {
                debug!(error = %e, "Topology: degrading to empty (get_primals failed)");
                Ok(Vec::new())
            }
        }
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
    use petal_tongue_core::PrimalHealthStatus;
    use std::path::PathBuf;

    fn sample_primal(id: &str, caps: &[&str]) -> PrimalInfo {
        PrimalInfo::new(
            id,
            id,
            "test",
            "unix:///tmp/test.sock",
            caps.iter().map(|s| (*s).to_string()).collect(),
            PrimalHealthStatus::Healthy,
            0,
        )
    }

    #[test]
    fn topology_edges_empty_for_zero_or_one_primal() {
        assert!(topology_edges_from_shared_capabilities(&[]).is_empty());
        assert!(topology_edges_from_shared_capabilities(&[sample_primal("a", &["x"])]).is_empty());
    }

    #[test]
    fn topology_edges_one_shared_capability() {
        let p = vec![
            sample_primal("a", &["compute", "storage"]),
            sample_primal("b", &["compute", "ai"]),
        ];
        let edges = topology_edges_from_shared_capabilities(&p);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "a");
        assert_eq!(edges[0].to, "b");
        assert_eq!(edges[0].edge_type, "capability");
        assert_eq!(edges[0].capability.as_deref(), Some("compute"));
    }

    #[test]
    fn topology_edges_multiple_shared_no_duplicate_directed() {
        let p = vec![
            sample_primal("p1", &["x", "y"]),
            sample_primal("p2", &["x", "y"]),
        ];
        let edges = topology_edges_from_shared_capabilities(&p);
        assert_eq!(edges.len(), 2);
        let caps: HashSet<_> = edges
            .iter()
            .filter_map(|e| e.capability.as_deref())
            .collect();
        assert_eq!(caps, HashSet::from(["x", "y"]));
        for e in &edges {
            assert_eq!(e.from, "p1");
            assert_eq!(e.to, "p2");
            assert_eq!(e.edge_type, "capability");
        }
    }

    #[test]
    fn topology_edges_dedupes_duplicate_caps_per_primal() {
        let mut a = sample_primal("a", &["shared"]);
        a.capabilities.push("shared".to_string());
        let b = sample_primal("b", &["shared"]);
        let edges = topology_edges_from_shared_capabilities(&[a, b]);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].capability.as_deref(), Some("shared"));
    }

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
