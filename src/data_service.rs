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

                // Initial data fetch — tolerate API method gaps (biomeOS may not
                // support primal.list yet). petalTongue stays alive with an empty
                // graph and populates on the next successful refresh.
                if let Err(e) = self.refresh().await {
                    tracing::warn!("⚠️ Initial refresh failed (degraded mode): {e}");
                }
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

    /// Synchronous snapshot for non-async contexts (SSE streams, etc.).
    ///
    /// Returns `None` if the graph lock is poisoned.
    pub fn snapshot_sync(&self) -> Option<DataSnapshot> {
        let graph = self.graph.read().ok()?;
        let primals = graph.nodes().iter().map(|node| node.info.clone()).collect();
        let edges = graph.edges().to_vec();
        drop(graph);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Some(DataSnapshot {
            primals,
            edges,
            timestamp,
        })
    }

    /// Subscribe to data updates.
    pub fn subscribe(&self) -> broadcast::Receiver<DataUpdate> {
        self.update_tx.subscribe()
    }

    /// Check if Neural API is available.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "used only in tests currently; public API for future modes"
        )
    )]
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
#[cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
mod tests {
    use super::*;
    use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
    use std::time::Duration;

    fn create_test_primal(id: &str, name: &str) -> PrimalInfo {
        PrimalInfo::new(
            id,
            name,
            "test",
            format!("http://test-{id}:8080"),
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        )
    }

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

    #[tokio::test]
    async fn test_snapshot_with_populated_graph() {
        let service = DataService::new();
        let graph = service.graph();
        {
            let mut guard = graph.write().unwrap();
            let p1 = create_test_primal("p1", "Primal 1");
            let p2 = create_test_primal("p2", "Primal 2");
            guard.add_node(p1);
            guard.add_node(p2);
            guard.add_edge(TopologyEdge {
                from: "p1".into(),
                to: "p2".into(),
                edge_type: "connection".to_string(),
                label: None,
                capability: None,
                metrics: None,
            });
        }
        let snapshot = service.snapshot().await.unwrap();
        assert_eq!(snapshot.primals.len(), 2);
        assert_eq!(snapshot.edges.len(), 1);
        assert_eq!(snapshot.primals[0].id.as_str(), "p1");
        assert_eq!(snapshot.primals[1].id.as_str(), "p2");
    }

    #[tokio::test]
    async fn test_snapshot_serialization_with_data() {
        let service = DataService::new();
        let graph = service.graph();
        {
            let mut guard = graph.write().unwrap();
            guard.add_node(create_test_primal("test-1", "Test Primal"));
        }
        let snapshot = service.snapshot().await.unwrap();
        let json = serde_json::to_string(&snapshot).unwrap();
        let deser: DataSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.primals.len(), 1);
        assert_eq!(deser.primals[0].id.as_str(), "test-1");
        assert_eq!(deser.primals[0].name, "Test Primal");
    }

    #[tokio::test]
    async fn test_broadcast_multiple_receivers() {
        let service = DataService::new();
        let mut rx1 = service.subscribe();
        let mut rx2 = service.subscribe();
        service.send_test_update();
        let update1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv())
            .await
            .expect("rx1 timed out")
            .expect("rx1 recv failed");
        let update2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv())
            .await
            .expect("rx2 timed out")
            .expect("rx2 recv failed");
        assert!(matches!(update1, DataUpdate::TopologyUpdated));
        assert!(matches!(update2, DataUpdate::TopologyUpdated));
    }

    #[tokio::test]
    async fn test_broadcast_multiple_updates() {
        let service = DataService::new();
        let mut rx = service.subscribe();
        service.send_test_update();
        service.send_test_update();
        let u1 = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");
        let u2 = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");
        assert!(matches!(u1, DataUpdate::TopologyUpdated));
        assert!(matches!(u2, DataUpdate::TopologyUpdated));
    }

    #[tokio::test]
    async fn test_graph_lock_poisoned_error_path() {
        let service = DataService::new();
        let graph = service.graph();
        let graph_clone = Arc::clone(&graph);
        let handle = std::thread::spawn(move || {
            let _guard = graph_clone.write().unwrap();
            panic!("intentional poison for test");
        });
        let _ = handle.join();
        let result = service.snapshot().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::GraphLockPoisoned(_)),
            "Expected GraphLockPoisoned, got: {err}"
        );
        assert!(err.to_string().contains("Graph lock poisoned"));
    }

    #[tokio::test]
    async fn test_snapshot_sync_returns_some() {
        let service = DataService::new();
        let snap = service.snapshot_sync();
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert!(snap.primals.is_empty());
        assert!(snap.edges.is_empty());
    }

    #[tokio::test]
    async fn test_snapshot_sync_with_populated_graph() {
        let service = DataService::new();
        let graph = service.graph();
        {
            let mut g = graph.write().unwrap();
            g.add_node(create_test_primal("sync-1", "SyncPrimal"));
        }
        let snap = service.snapshot_sync().unwrap();
        assert_eq!(snap.primals.len(), 1);
        assert_eq!(snap.primals[0].id.as_str(), "sync-1");
    }

    #[tokio::test]
    async fn test_snapshot_sync_poisoned_graph_returns_none() {
        let service = DataService::new();
        let graph = service.graph();
        let g2 = Arc::clone(&graph);
        let h = std::thread::spawn(move || {
            let _guard = g2.write().unwrap();
            panic!("intentional poison for snapshot_sync test");
        });
        let _ = h.join();
        assert!(service.snapshot_sync().is_none());
    }

    #[tokio::test]
    async fn test_data_snapshot_serialization_roundtrip_with_edges() {
        let snapshot = DataSnapshot {
            primals: vec![
                create_test_primal("a", "Alpha"),
                create_test_primal("b", "Beta"),
            ],
            edges: vec![TopologyEdge {
                from: "a".into(),
                to: "b".into(),
                edge_type: "api_call".to_string(),
                label: Some("invoke".to_string()),
                capability: None,
                metrics: None,
            }],
            timestamp: 12345,
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let deser: DataSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.primals.len(), 2);
        assert_eq!(deser.edges.len(), 1);
        assert_eq!(deser.edges[0].edge_type, "api_call");
        assert_eq!(deser.timestamp, 12345);
    }
}
