// SPDX-License-Identifier: AGPL-3.0-only
//! TUI State Management
//!
//! Central state for the Rich TUI. All state is managed here,
//! with zero global state or unsafe code.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use petal_tongue_core::{PrimalInfo, TopologyEdge};

/// The active view in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// System overview
    Dashboard,
    /// ASCII graph visualization
    Topology,
    /// Device management
    Devices,
    /// Primal status
    Primals,
    /// Real-time log streaming
    Logs,
    /// neuralAPI graph orchestration
    NeuralAPI,
    /// NUCLEUS secure discovery
    Nucleus,
    /// `LiveSpore` live deployment
    LiveSpore,
}

impl View {
    /// Get the keyboard shortcut for this view
    #[must_use]
    pub const fn shortcut(&self) -> char {
        match self {
            Self::Dashboard => '1',
            Self::Topology => '2',
            Self::Devices => '3',
            Self::Primals => '4',
            Self::Logs => '5',
            Self::NeuralAPI => '6',
            Self::Nucleus => '7',
            Self::LiveSpore => '8',
        }
    }

    /// Get the display name for this view
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Topology => "Topology",
            Self::Devices => "Devices",
            Self::Primals => "Primals",
            Self::Logs => "Logs",
            Self::NeuralAPI => "neuralAPI",
            Self::Nucleus => "NUCLEUS",
            Self::LiveSpore => "LiveSpore",
        }
    }

    /// Get all views
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::Dashboard,
            Self::Topology,
            Self::Devices,
            Self::Primals,
            Self::Logs,
            Self::NeuralAPI,
            Self::Nucleus,
            Self::LiveSpore,
        ]
    }
}

/// Log message with metadata
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source primal (if known)
    pub source: Option<String>,
    /// Log level
    pub level: LogLevel,
    /// Message content
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

/// System status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Number of active primals
    pub active_primals: usize,
    /// Number of discovered devices
    pub discovered_devices: usize,
    /// System uptime
    pub uptime: std::time::Duration,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            active_primals: 0,
            discovered_devices: 0,
            uptime: std::time::Duration::from_secs(0),
            last_update: Utc::now(),
        }
    }
}

/// Central TUI state
///
/// All state is managed here, with proper async locking.
/// No global state, no unsafe code.
#[derive(Clone)]
pub struct TUIState {
    /// Current view
    view: Arc<RwLock<View>>,

    /// Discovered primals (capability-based)
    primals: Arc<RwLock<Vec<PrimalInfo>>>,

    /// Topology edges (if Songbird available)
    topology: Arc<RwLock<Vec<TopologyEdge>>>,

    /// Log messages (ring buffer)
    logs: Arc<RwLock<Vec<LogMessage>>>,

    /// System status
    status: Arc<RwLock<SystemStatus>>,

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
            primals: Arc::new(RwLock::new(Vec::new())),
            topology: Arc::new(RwLock::new(Vec::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(SystemStatus::default())),
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

    /// Get primals
    pub async fn get_primals(&self) -> Vec<PrimalInfo> {
        self.primals.read().await.clone()
    }

    /// Get primal count (no clone)
    pub async fn primal_count(&self) -> usize {
        self.primals.read().await.len()
    }

    /// Update primals (from discovery)
    pub async fn update_primals(&self, primals: Vec<PrimalInfo>) {
        *self.primals.write().await = primals;
        self.update_status().await;
    }

    /// Get topology
    pub async fn get_topology(&self) -> Vec<TopologyEdge> {
        self.topology.read().await.clone()
    }

    /// Get topology edge count (no clone)
    pub async fn topology_edge_count(&self) -> usize {
        self.topology.read().await.len()
    }

    /// Update topology (from Songbird)
    pub async fn update_topology(&self, topology: Vec<TopologyEdge>) {
        *self.topology.write().await = topology;
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

    /// Get logs
    pub async fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.read().await.clone()
    }

    /// Get log count (no clone)
    pub async fn log_count(&self) -> usize {
        self.logs.read().await.len()
    }

    /// Get system status
    pub async fn get_status(&self) -> SystemStatus {
        self.status.read().await.clone()
    }

    /// Update system status
    async fn update_status(&self) {
        let primals = self.primals.read().await;
        let mut status = self.status.write().await;
        status.active_primals = primals.len();
        status.last_update = Utc::now();
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

/// TUI statistics
#[derive(Debug, Clone)]
pub struct TUIStats {
    /// Current view
    pub view: View,
    /// Number of primals
    pub primal_count: usize,
    /// Number of topology edges
    pub topology_edge_count: usize,
    /// Number of log messages
    pub log_count: usize,
    /// Standalone mode
    pub standalone: bool,
    /// Number of registered capabilities
    pub registered_capabilities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_view_switching() {
        let state = TUIState::new();

        assert_eq!(state.get_view().await, View::Dashboard);

        state.set_view(View::Topology).await;
        assert_eq!(state.get_view().await, View::Topology);
    }

    #[tokio::test]
    async fn test_view_shortcuts() {
        assert_eq!(View::Dashboard.shortcut(), '1');
        assert_eq!(View::Topology.shortcut(), '2');
        assert_eq!(View::NeuralAPI.shortcut(), '6');
    }

    #[tokio::test]
    async fn test_log_ring_buffer() {
        let state = TUIState::new();

        // Add 1100 logs
        for i in 0..1100 {
            state
                .add_log(LogMessage {
                    timestamp: Utc::now(),
                    source: None,
                    level: LogLevel::Info,
                    message: format!("Log {i}"),
                })
                .await;
        }

        // Should keep only last 1000
        let logs = state.get_logs().await;
        assert_eq!(logs.len(), 1000);
        assert!(logs.last().unwrap().message.contains("1099"));
    }

    #[tokio::test]
    async fn test_selection_wrapping() {
        let state = TUIState::new();

        // Select next with 5 items
        state.select_next(5).await;
        assert_eq!(state.get_selected_index().await, 1);

        // Select previous
        state.select_previous(5).await;
        assert_eq!(state.get_selected_index().await, 0);

        // Wrap to bottom
        state.select_previous(5).await;
        assert_eq!(state.get_selected_index().await, 4);

        // Wrap to top
        state.select_next(5).await;
        assert_eq!(state.get_selected_index().await, 0);
    }

    #[tokio::test]
    async fn test_capability_registration() {
        let state = TUIState::new();

        state.register_capability(
            "songbird".to_string(),
            vec!["discovery".to_string(), "events".to_string()],
        );

        assert!(state.has_capability("songbird", "discovery"));
        assert!(state.has_capability("songbird", "events"));
        assert!(!state.has_capability("songbird", "compute"));
        assert!(!state.has_capability("toadstool", "compute"));

        let caps = state.get_capabilities("songbird").unwrap();
        assert_eq!(caps.len(), 2);
    }

    #[tokio::test]
    async fn test_standalone_mode() {
        let state = TUIState::new();

        assert!(!state.is_standalone().await);

        state.set_standalone_mode(true).await;
        assert!(state.is_standalone().await);

        state.set_standalone_mode(false).await;
        assert!(!state.is_standalone().await);
    }

    #[tokio::test]
    async fn test_stats() {
        let state = TUIState::new();

        state.set_view(View::Primals).await;
        state.register_capability("songbird".to_string(), vec!["discovery".to_string()]);

        let stats = state.stats().await;
        assert_eq!(stats.view, View::Primals);
        assert_eq!(stats.registered_capabilities, 1);
    }
}
