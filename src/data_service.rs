// SPDX-License-Identifier: AGPL-3.0-only
//! Unified Data Service
//!
//! Single source of truth for all UI modes (GUI, TUI, Web, Headless)
//!
//! TRUE PRIMAL:
//! - Data fetching happens ONCE
//! - All UIs consume the SAME data
//! - Zero duplication
//! - Capability-based discovery

use anyhow::{Context, Result};
use petal_tongue_core::{GraphEngine, PrimalInfo, TopologyEdge};
use petal_tongue_discovery::NeuralApiProvider;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Unified data service for all UI modes
///
/// This is the SINGLE source of truth for:
/// - Primal discovery
/// - Topology
/// - System metrics
pub struct DataService {
    /// Graph engine (shared across all UIs)
    graph: Arc<RwLock<GraphEngine>>,

    /// Neural API provider
    neural_api: Option<Arc<NeuralApiProvider>>,

    /// Broadcast channel for data updates
    update_tx: broadcast::Sender<DataUpdate>,

    /// Last refresh time
    last_refresh: Arc<RwLock<std::time::Instant>>,
}

/// Data update notification
#[derive(Clone, Debug)]
pub enum DataUpdate {
    /// Graph topology updated
    TopologyUpdated,
}

/// Complete data snapshot for UI consumption
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DataSnapshot {
    /// Discovered primals
    pub primals: Vec<PrimalInfo>,

    /// Topology edges
    pub edges: Vec<TopologyEdge>,

    /// Timestamp (as seconds since UNIX epoch)
    pub timestamp: u64,
}

impl DataService {
    /// Create new data service
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);

        Self {
            graph: Arc::new(RwLock::new(GraphEngine::new())),
            neural_api: None,
            update_tx: tx,
            last_refresh: Arc::new(RwLock::new(std::time::Instant::now())),
        }
    }

    /// Initialize with Neural API discovery
    pub async fn init(&mut self) -> Result<()> {
        // Discover Neural API provider
        match NeuralApiProvider::discover(None).await {
            Ok(provider) => {
                tracing::info!("✅ Neural API discovered");
                self.neural_api = Some(Arc::new(provider));

                // Initial data fetch
                self.refresh().await?;
            }
            Err(e) => {
                tracing::warn!("⚠️ Neural API not available: {}", e);
                tracing::info!("📊 Using fallback data (tutorial mode)");
            }
        }

        Ok(())
    }

    /// Refresh data from Neural API
    pub async fn refresh(&self) -> Result<()> {
        if let Some(api) = &self.neural_api {
            use petal_tongue_discovery::VisualizationDataProvider;

            // Fetch primals
            let primals = api
                .as_ref()
                .get_primals()
                .await
                .context("Failed to get primals from Neural API")?;

            // Fetch topology
            let topology = api
                .as_ref()
                .get_topology()
                .await
                .context("Failed to get topology from Neural API")?;

            // Update graph
            {
                let mut graph = self
                    .graph
                    .write()
                    .map_err(|e| anyhow::anyhow!("Graph lock poisoned: {e}"))?;

                // Clear and rebuild
                *graph = GraphEngine::new();

                for primal in &primals {
                    graph.add_node(primal.clone());
                }

                for edge in topology {
                    graph.add_edge(edge);
                }
            }

            // Update refresh time
            {
                let mut last_refresh = self
                    .last_refresh
                    .write()
                    .map_err(|e| anyhow::anyhow!("Refresh time lock poisoned: {e}"))?;
                *last_refresh = std::time::Instant::now();
            }

            // Notify subscribers
            let _ = self.update_tx.send(DataUpdate::TopologyUpdated);

            tracing::debug!("✅ Data refreshed: {} primals", primals.len());
        }

        Ok(())
    }

    /// Get current data snapshot
    #[expect(clippy::unused_async, reason = "async for future async graph access")]
    pub async fn snapshot(&self) -> Result<DataSnapshot> {
        // Get primals and edges from graph
        let (primals, edges) = {
            let graph = self
                .graph
                .read()
                .map_err(|e| anyhow::anyhow!("Graph lock poisoned: {e}"))?;

            // Extract PrimalInfo from Node wrappers
            let primals = graph.nodes().iter().map(|node| node.info.clone()).collect();
            let edges = graph.edges().to_vec();

            (primals, edges)
        };

        // Get current timestamp (fallback to 0 if system clock is invalid)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(DataSnapshot {
            primals,
            edges,
            timestamp,
        })
    }

    /// Get graph engine (for direct rendering)
    pub fn graph(&self) -> Arc<RwLock<GraphEngine>> {
        Arc::clone(&self.graph)
    }

    /// Subscribe to data updates (public API for streaming consumers).
    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<DataUpdate> {
        self.update_tx.subscribe()
    }

    /// Check if Neural API is available.
    #[allow(dead_code)]
    pub const fn has_neural_api(&self) -> bool {
        self.neural_api.is_some()
    }
}

impl Default for DataService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_data_service_creation() {
        let service = DataService::new();
        assert!(!service.has_neural_api());
    }

    #[tokio::test]
    async fn test_data_service_default() {
        let service = DataService::default();
        assert!(!service.has_neural_api());
    }

    #[tokio::test]
    async fn test_snapshot_without_neural_api() {
        let service = DataService::new();
        let snapshot = service.snapshot().await.unwrap();

        assert!(snapshot.primals.is_empty());
        assert!(snapshot.edges.is_empty());
        // Timestamp is always valid (epoch or later)
        let _ = snapshot.timestamp;
    }

    #[tokio::test]
    async fn test_snapshot_timestamp() {
        let service = DataService::new();
        let snapshot = service.snapshot().await.unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        assert!(
            snapshot.timestamp <= now + 1,
            "Timestamp should be reasonable"
        );
    }

    #[tokio::test]
    async fn test_graph_access() {
        let service = DataService::new();
        let graph = service.graph();
        let guard = graph.read().unwrap();
        assert!(guard.nodes().is_empty());
        assert!(guard.edges().is_empty());
    }

    #[tokio::test]
    async fn test_update_subscription() {
        let service = DataService::new();
        let mut rx = service.subscribe();

        // Send update
        let _ = service.update_tx.send(DataUpdate::TopologyUpdated);

        // Should receive it (with timeout to avoid blocking forever)
        let update = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        assert!(matches!(update, DataUpdate::TopologyUpdated));
    }

    #[tokio::test]
    async fn test_refresh_without_neural_api() {
        let service = DataService::new();
        let result = service.refresh().await;
        assert!(result.is_ok());
    }
}
