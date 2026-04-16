// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`TUIState`] — async-safe central state.

use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use petal_tongue_core::{PrimalInfo, TopologyEdge};

use super::types::{LogMessage, SystemStatus, TUIStats, View};

/// Central TUI state
///
/// All state is managed here, with proper async locking.
/// No global state, no unsafe code.
/// Uses Arc-wrapped collections for zero-copy reads (`Arc::clone` is O(1)).
#[derive(Clone)]
pub struct TUIState {
    /// Current view
    view: Arc<RwLock<View>>,

    /// Discovered primals (capability-based) — Arc for zero-copy `get_primals`
    primals: Arc<RwLock<Arc<Vec<PrimalInfo>>>>,

    /// Topology edges (from registry/discovery provider) — Arc for zero-copy `get_topology`
    topology: Arc<RwLock<Arc<Vec<TopologyEdge>>>>,

    /// Log messages (ring buffer) — Vec; `add_log` is write-heavy, clone on read acceptable
    logs: Arc<RwLock<Vec<LogMessage>>>,

    /// System status — Arc for zero-copy `get_status`
    status: Arc<RwLock<Arc<SystemStatus>>>,

    /// Selected item in current view (generic selection)
    selected_index: Arc<RwLock<usize>>,

    /// Available primal capabilities (runtime discovered)
    capabilities: Arc<DashMap<String, Vec<String>>>,

    /// Running in standalone mode (no other primals)
    standalone_mode: Arc<RwLock<bool>>,
}

impl TUIState {
    /// Create new TUI state
    #[must_use]
    pub fn new() -> Self {
        Self {
            view: Arc::new(RwLock::new(View::Dashboard)),
            primals: Arc::new(RwLock::new(Arc::new(Vec::new()))),
            topology: Arc::new(RwLock::new(Arc::new(Vec::new()))),
            logs: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(Arc::new(SystemStatus::default()))),
            selected_index: Arc::new(RwLock::new(0)),
            capabilities: Arc::new(DashMap::new()),
            standalone_mode: Arc::new(RwLock::new(false)),
        }
    }

    /// Get current view
    pub async fn get_view(&self) -> View {
        *self.view.read().await
    }

    /// Set current view
    pub async fn set_view(&self, view: View) {
        *self.view.write().await = view;
        // Reset selection when switching views
        *self.selected_index.write().await = 0;
    }

    /// Get primals (zero-copy: returns Arc, `Arc::clone` is O(1))
    pub async fn get_primals(&self) -> Arc<Vec<PrimalInfo>> {
        Arc::clone(&*self.primals.read().await)
    }

    /// Get primal count (no clone)
    pub async fn primal_count(&self) -> usize {
        self.primals.read().await.len()
    }

    /// Update primals (from discovery)
    pub async fn update_primals(&self, primals: Vec<PrimalInfo>) {
        *self.primals.write().await = Arc::new(primals);
        self.update_status().await;
    }

    /// Get topology (zero-copy: returns Arc, `Arc::clone` is O(1))
    pub async fn get_topology(&self) -> Arc<Vec<TopologyEdge>> {
        Arc::clone(&*self.topology.read().await)
    }

    /// Get topology edge count (no clone)
    pub async fn topology_edge_count(&self) -> usize {
        self.topology.read().await.len()
    }

    /// Update topology (from registry/discovery provider)
    pub async fn update_topology(&self, topology: Vec<TopologyEdge>) {
        *self.topology.write().await = Arc::new(topology);
    }

    /// Add log message
    pub async fn add_log(&self, log: LogMessage) {
        let mut logs = self.logs.write().await;
        logs.push(log);

        // Keep only last 1000 logs (ring buffer)
        if logs.len() > 1000 {
            let excess = logs.len() - 1000;
            logs.drain(0..excess);
        }
    }

    /// Get logs (clone required; `add_log` is write-heavy so Arc would hurt append path)
    pub async fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.read().await.clone()
    }

    /// Get log count (no clone)
    pub async fn log_count(&self) -> usize {
        self.logs.read().await.len()
    }

    /// Get system status (zero-copy: returns Arc, `Arc::clone` is O(1))
    pub async fn get_status(&self) -> Arc<SystemStatus> {
        Arc::clone(&*self.status.read().await)
    }

    /// Update system status
    async fn update_status(&self) {
        let primal_count = self.primals.read().await.len();
        let (discovered_devices, uptime) = {
            let old = self.status.read().await;
            (old.discovered_devices, old.uptime)
        };
        let new_status = Arc::new(SystemStatus {
            active_primals: primal_count,
            discovered_devices,
            uptime,
            last_update: Utc::now(),
        });
        *self.status.write().await = new_status;
    }

    /// Get selected index
    pub async fn get_selected_index(&self) -> usize {
        *self.selected_index.read().await
    }

    /// Set selected index
    pub async fn set_selected_index(&self, index: usize) {
        *self.selected_index.write().await = index;
    }

    /// Move selection up
    pub async fn select_previous(&self, max: usize) {
        let mut selected = self.selected_index.write().await;
        if *selected > 0 {
            *selected -= 1;
        } else if max > 0 {
            *selected = max - 1; // Wrap to bottom
        }
    }

    /// Move selection down
    pub async fn select_next(&self, max: usize) {
        let mut selected = self.selected_index.write().await;
        if max > 0 {
            *selected = (*selected + 1) % max;
        }
    }

    /// Register primal capability (runtime discovered)
    pub fn register_capability(&self, primal: String, capabilities: Vec<String>) {
        self.capabilities.insert(primal, capabilities);
    }

    /// Check if primal has capability
    #[must_use]
    pub fn has_capability(&self, primal: &str, capability: &str) -> bool {
        self.capabilities
            .get(primal)
            .is_some_and(|caps| caps.iter().any(|c| c == capability))
    }

    /// Get all capabilities for a primal
    #[must_use]
    pub fn get_capabilities(&self, primal: &str) -> Option<Vec<String>> {
        self.capabilities.get(primal).map(|caps| caps.clone())
    }

    /// Set standalone mode
    pub async fn set_standalone_mode(&self, standalone: bool) {
        *self.standalone_mode.write().await = standalone;
    }

    /// Check if in standalone mode
    pub async fn is_standalone(&self) -> bool {
        *self.standalone_mode.read().await
    }

    /// Get statistics
    pub async fn stats(&self) -> TUIStats {
        TUIStats {
            view: self.get_view().await,
            primal_count: self.primals.read().await.len(),
            topology_edge_count: self.topology.read().await.len(),
            log_count: self.logs.read().await.len(),
            standalone: self.is_standalone().await,
            registered_capabilities: self.capabilities.len(),
        }
    }
}

impl Default for TUIState {
    fn default() -> Self {
        Self::new()
    }
}
