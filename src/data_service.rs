// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unified Data Service
//!
//! Single source of truth for all modalities (display, TUI, Web, Headless)
//!
//! TRUE PRIMAL:
//! - Data fetching happens ONCE
//! - All UIs consume the SAME data
//! - Zero duplication
//! - Capability-based discovery

use petal_tongue_core::{GraphEngine, PrimalInfo, TopologyEdge};

use crate::error::AppError;

type Result<T> = std::result::Result<T, AppError>;
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
            let primals = api.as_ref().get_primals().await.map_err(|e| {
                AppError::NeuralApi(format!("Failed to get primals from Neural API: {e}"))
            })?;

            // Fetch topology
            let topology = api.as_ref().get_topology().await.map_err(|e| {
                AppError::NeuralApi(format!("Failed to get topology from Neural API: {e}"))
            })?;

            // Update graph
            {
                let mut graph = self
                    .graph
                    .write()
                    .map_err(|e| AppError::GraphLockPoisoned(e.to_string()))?;

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
                    .map_err(|e| AppError::RefreshLockPoisoned(e.to_string()))?;
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
                .map_err(|e| AppError::GraphLockPoisoned(e.to_string()))?;

            // Extract PrimalInfo from Node wrappers
            let primals = graph.nodes().iter().map(|node| node.info.clone()).collect();
            let edges = graph.edges().to_vec();
            drop(graph);

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

    /// Subscribe to data updates (streaming consumers, display/TUI wiring pending).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn subscribe(&self) -> broadcast::Receiver<DataUpdate> {
        self.update_tx.subscribe()
    }

    /// Check if Neural API is available.
    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn has_neural_api(&self) -> bool {
        self.neural_api.is_some()
    }

    /// Send a test update (for subscription tests when `neural_api` is None).
    #[cfg(test)]
    pub(crate) fn send_test_update(&self) {
        let _ = self.update_tx.send(DataUpdate::TopologyUpdated);
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
        drop(guard);
    }

    #[tokio::test]
    async fn test_update_subscription() {
        let service = DataService::new();
        let mut rx = service.subscribe();

        // Trigger update (refresh doesn't send when neural_api is None)
        service.send_test_update();

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

    #[tokio::test]
    async fn test_init_without_neural_api() {
        let mut service = DataService::new();
        let result = service.init().await;
        assert!(result.is_ok());
        // Without a running API endpoint, neural_api stays None
    }

    #[tokio::test]
    async fn test_snapshot_serialization() {
        let service = DataService::new();
        let snapshot = service.snapshot().await.unwrap();
        let json = serde_json::to_string(&snapshot).unwrap();
        let deser: DataSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.primals.len(), snapshot.primals.len());
        assert_eq!(deser.edges.len(), snapshot.edges.len());
        assert_eq!(deser.timestamp, snapshot.timestamp);
    }

    #[tokio::test]
    async fn test_graph_shared_across_clones() {
        let service = DataService::new();
        let graph1 = service.graph();
        let graph2 = service.graph();
        assert!(Arc::ptr_eq(&graph1, &graph2));
    }

    #[tokio::test]
    async fn test_multiple_snapshots_consistent() {
        let service = DataService::new();
        let snap1 = service.snapshot().await.unwrap();
        let snap2 = service.snapshot().await.unwrap();
        assert_eq!(snap1.primals.len(), snap2.primals.len());
        assert_eq!(snap1.edges.len(), snap2.edges.len());
    }

    #[tokio::test]
    async fn test_data_update_debug() {
        let update = DataUpdate::TopologyUpdated;
        let debug = format!("{update:?}");
        assert!(debug.contains("TopologyUpdated"));
    }

    #[tokio::test]
    async fn test_data_update_clone() {
        let update = DataUpdate::TopologyUpdated;
        let cloned = Clone::clone(&update);
        assert!(matches!(cloned, DataUpdate::TopologyUpdated));
    }

    #[tokio::test]
    async fn test_subscribe_multiple_receivers() {
        let service = DataService::new();
        let _rx1 = service.subscribe();
        let _rx2 = service.subscribe();
    }

    #[tokio::test]
    async fn test_refresh_then_snapshot() {
        let service = DataService::new();
        service.refresh().await.unwrap();
        let snapshot = service.snapshot().await.unwrap();
        assert!(snapshot.primals.is_empty());
    }

    #[tokio::test]
    async fn test_data_snapshot_debug() {
        let service = DataService::new();
        let snapshot = service.snapshot().await.unwrap();
        let debug_str = format!("{snapshot:?}");
        assert!(debug_str.contains("primals"));
        assert!(debug_str.contains("edges"));
    }

    #[tokio::test]
    async fn test_data_snapshot_clone() {
        let service = DataService::new();
        let snapshot = service.snapshot().await.unwrap();
        let cloned = snapshot.clone();
        assert_eq!(cloned.primals.len(), snapshot.primals.len());
        assert_eq!(cloned.edges.len(), snapshot.edges.len());
        assert_eq!(cloned.timestamp, snapshot.timestamp);
    }
}
