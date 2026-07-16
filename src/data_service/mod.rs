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

mod mesh;
mod types;

#[cfg(test)]
mod tests;

use petal_tongue_core::GraphEngine;

use crate::error::AppError;

type Result<T> = std::result::Result<T, AppError>;
use petal_tongue_discovery::NeuralApiProvider;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

pub use types::*;

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

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());

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
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "public API for external consumers and TUI mode; tests exercise it"
        )
    )]
    pub fn snapshot_sync(&self) -> Option<DataSnapshot> {
        let graph = self
            .graph
            .read()
            .inspect_err(|e| tracing::warn!("graph lock poisoned: {e}"))
            .ok()?;
        let primals = graph.nodes().iter().map(|node| node.info.clone()).collect();
        let edges = graph.edges().to_vec();
        drop(graph);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());

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

    /// Get current mesh peer state (songBird `mesh.peers` concept).
    ///
    /// Currently returns statically derived peers from the gate topology.
    /// When songBird IPC is available, this will query live peer state.
    #[must_use]
    pub fn mesh_peers() -> Vec<petal_tongue_core::gate_mesh::MeshPeer> {
        mesh::mesh_peers()
    }

    /// Get live topology for TOPO-VIS visualization.
    ///
    /// Returns live Neural API data (primals + edges) when available,
    /// with static gate mesh data as fallback. This is the primary source
    /// for the `/api/topology/live` endpoint.
    pub fn live_topology(&self) -> LiveTopology {
        let has_api = self.neural_api.is_some();
        let graph = self.graph.read().ok();
        mesh::live_topology(has_api, graph.as_deref())
    }

    /// Check if Neural API is available.
    #[must_use]
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
