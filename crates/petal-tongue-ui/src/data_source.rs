// SPDX-License-Identifier: AGPL-3.0-only
//! Data source management for topology and primal discovery
//!
//! This module handles fetching topology data from `BiomeOS` via capability-based discovery.
//! NO HARDCODED PRIMAL NAMES - all discovery happens at runtime via `BiomeOS`.
//!
//! # Design Philosophy
//!
//! - **Runtime Discovery**: Never hardcode primal names or endpoints
//! - **Capability-Based**: Discover what's available, don't assume
//! - **Graceful Degradation**: Fall back to minimal data if `BiomeOS` unavailable
//! - **Self-Knowledge Only**: petalTongue knows itself, discovers others

use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{GraphEngine, PrimalInfo, TopologyEdge};
use std::sync::{Arc, RwLock};

/// Data source for topology information
///
/// Handles fetching primal discovery and topology data from `BiomeOS`.
/// Uses capability-based runtime discovery - no hardcoded primal names.
pub struct DataSource {
    /// `BiomeOS` client for live discovery
    client: BiomeOSClient,
}

impl DataSource {
    /// Create a new data source
    #[must_use]
    pub fn new(client: BiomeOSClient) -> Self {
        Self { client }
    }

    /// Refresh graph data from `BiomeOS`
    ///
    /// Discovers primals and topology via `BiomeOS` capability-based API.
    /// If `BiomeOS` is unavailable, returns empty data (no mock fallback in production).
    ///
    /// # Returns
    ///
    /// - `Ok((primals, edges))` on success
    /// - `Err(...)` if `BiomeOS` unavailable and no fallback
    pub async fn refresh_topology(&self) -> Result<(Vec<PrimalInfo>, Vec<TopologyEdge>), String> {
        // Discover primals via BiomeOS
        let primals = self
            .client
            .discover_primals()
            .await
            .map_err(|e| format!("Failed to discover primals: {e}"))?;

        // Get topology edges
        let edges = self
            .client
            .get_topology()
            .await
            .map_err(|e| format!("Failed to get topology: {e}"))?;

        Ok((primals, edges))
    }

    /// Update graph engine with discovered data
    ///
    /// Takes discovered primals and topology, updates the graph engine.
    pub fn update_graph(
        &self,
        graph: &Arc<RwLock<GraphEngine>>,
        primals: Vec<PrimalInfo>,
        edges: Vec<TopologyEdge>,
    ) -> Result<(), String> {
        let mut graph = graph
            .write()
            .map_err(|e| format!("Failed to acquire graph lock: {e}"))?;

        // Clear existing data
        graph.clear();

        // Add discovered primals
        for primal in primals {
            graph.add_node(primal);
        }

        // Add topology edges
        for edge in edges {
            graph.add_edge(edge);
        }

        Ok(())
    }

    /// Refresh and update graph in one operation
    ///
    /// Convenience method that discovers topology and updates graph.
    pub async fn refresh_and_update(&self, graph: &Arc<RwLock<GraphEngine>>) -> Result<(), String> {
        let (primals, edges) = self.refresh_topology().await?;
        self.update_graph(graph, primals, edges)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_source_creation() {
        let client = BiomeOSClient::new("http://test:3000").with_mock_mode(true);
        let data_source = DataSource::new(client);
        // Just verify it constructs
        assert!(std::mem::size_of_val(&data_source) > 0);
    }

    #[tokio::test]
    async fn test_refresh_topology_mock() {
        let client = BiomeOSClient::new("http://test:3000").with_mock_mode(true);
        let data_source = DataSource::new(client);

        let result = data_source.refresh_topology().await;
        assert!(result.is_ok());

        let (primals, edges) = result.unwrap();
        assert!(!primals.is_empty(), "Mock data should provide primals");
        assert!(!edges.is_empty(), "Mock data should provide edges");
    }

    #[tokio::test]
    async fn test_update_graph() {
        let client = BiomeOSClient::new("http://test:3000").with_mock_mode(true);
        let data_source = DataSource::new(client);

        let graph = Arc::new(RwLock::new(GraphEngine::new()));

        let (primals, edges) = data_source.refresh_topology().await.unwrap();
        let result = data_source.update_graph(&graph, primals, edges);

        assert!(result.is_ok());

        // Verify graph was updated
        let graph = graph
            .read()
            .expect("SAFETY: Lock poisoned - indicates panic in concurrent thread");
        let node_count = graph.nodes().len();
        assert!(node_count > 0, "Graph should have nodes after update");
    }

    #[tokio::test]
    async fn test_refresh_and_update() {
        let client = BiomeOSClient::new("http://test:3000").with_mock_mode(true);
        let data_source = DataSource::new(client);
        let graph = Arc::new(RwLock::new(GraphEngine::new()));

        let result = data_source.refresh_and_update(&graph).await;
        assert!(result.is_ok());

        let graph = graph
            .read()
            .expect("SAFETY: Lock poisoned - indicates panic in concurrent thread");
        assert!(graph.nodes().len() > 0);
    }
}
