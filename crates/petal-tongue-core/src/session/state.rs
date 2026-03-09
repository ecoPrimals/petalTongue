// SPDX-License-Identifier: AGPL-3.0-only

use crate::instance::InstanceId;
use crate::{LayoutAlgorithm, PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::SessionError;
use super::persistence;
use super::validation::SessionStateLike;

/// Persisted session state: graph topology, viewport, accessibility, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Schema version for migration.
    pub version: u32,
    /// Instance this session belongs to.
    pub instance_id: InstanceId,
    /// Unix timestamp of last modification.
    pub timestamp: u64,
    /// Optional display name for the session.
    pub name: Option<String>,
    /// Primal nodes in the topology graph.
    pub nodes: Vec<PrimalInfo>,
    /// Edges between primals.
    pub edges: Vec<TopologyEdge>,
    /// Layout algorithm for graph rendering.
    pub layout: LayoutAlgorithm,
    /// Node ID to (x, y) position for manual layout.
    pub node_positions: HashMap<String, (f32, f32)>,
    /// Window position (x, y) on screen.
    pub window_position: Option<(i32, i32)>,
    /// Window size (width, height).
    pub window_size: Option<(u32, u32)>,
    /// Zoom level for the graph view.
    pub zoom_level: f32,
    /// Pan offset (x, y) for the graph view.
    pub pan_offset: (f32, f32),
    /// IDs of open UI panels.
    pub panels_open: HashSet<String>,
    /// Accessibility preferences.
    pub accessibility: AccessibilitySettings,
    /// ID of the active scenario, if any.
    pub active_scenario: Option<String>,
    /// Whether topology auto-refresh is enabled.
    pub auto_refresh: bool,
    /// Seconds between topology refreshes.
    pub refresh_interval: f32,
    /// Cached trust summary for the topology.
    pub trust_summary: Option<TrustSummary>,
    /// Arbitrary key-value metadata.
    pub metadata: HashMap<String, String>,
}

/// Accessibility preferences for color, font, and audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Color scheme name (e.g. "standard", "high-contrast").
    pub color_scheme: String,
    /// Font size multiplier (1.0 = default).
    pub font_size: f32,
    /// Whether audio feedback is enabled.
    pub audio_enabled: bool,
    /// Audio volume (0.0–1.0).
    pub audio_volume: f32,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            color_scheme: "standard".to_string(),
            font_size: 1.0,
            audio_enabled: false,
            audio_volume: 0.7,
        }
    }
}

/// Cached summary of trust levels across the topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSummary {
    /// Total number of primals.
    pub total_primals: usize,
    /// Count per trust level (0–255).
    pub trust_distribution: HashMap<u8, usize>,
    /// Unique family identifiers.
    pub unique_families: HashSet<String>,
    /// Mean trust value across primals.
    pub average_trust: f32,
}

impl Default for TrustSummary {
    fn default() -> Self {
        Self {
            total_primals: 0,
            trust_distribution: HashMap::new(),
            unique_families: HashSet::new(),
            average_trust: 0.0,
        }
    }
}

impl SessionState {
    /// Current session schema version.
    pub const VERSION: u32 = 1;

    /// Creates a new empty session for the given instance.
    #[must_use]
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            version: Self::VERSION,
            instance_id,
            timestamp: persistence::current_timestamp(),
            name: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            layout: LayoutAlgorithm::ForceDirected,
            node_positions: HashMap::new(),
            window_position: None,
            window_size: None,
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
            panels_open: HashSet::new(),
            accessibility: AccessibilitySettings::default(),
            active_scenario: None,
            auto_refresh: true,
            refresh_interval: 5.0,
            trust_summary: None,
            metadata: HashMap::new(),
        }
    }

    /// Saves this state to the given path.
    pub fn save(&self, path: &Path) -> Result<(), SessionError> {
        persistence::save_session(self, path)
    }

    /// Loads session state from the given path.
    pub fn load(path: &Path) -> Result<Self, SessionError> {
        persistence::load_session(path)
    }

    /// Exports this state to the given path (alias for `save`).
    pub fn export(&self, path: &Path) -> Result<(), SessionError> {
        self.save(path)
    }

    /// Imports session state from the given path (alias for `load`).
    pub fn import(path: &Path) -> Result<Self, SessionError> {
        Self::load(path)
    }

    /// Merges nodes, edges, and positions from another state; skips duplicates.
    pub fn merge_graph(&mut self, other: &Self) {
        let existing_ids: HashSet<_> = self.nodes.iter().map(|n| n.id.clone()).collect();

        for node in &other.nodes {
            if !existing_ids.contains(&node.id) {
                self.nodes.push(node.clone());
            }
        }

        let existing_edges: HashSet<_> = self
            .edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone(), e.edge_type.clone()))
            .collect();

        for edge in &other.edges {
            let edge_key = (edge.from.clone(), edge.to.clone(), edge.edge_type.clone());
            if !existing_edges.contains(&edge_key) {
                self.edges.push(edge.clone());
            }
        }

        for (node_id, position) in &other.node_positions {
            self.node_positions.insert(node_id.clone(), *position);
        }

        self.timestamp = persistence::current_timestamp();

        tracing::info!(
            "Merged session: {} nodes, {} edges",
            self.nodes.len(),
            self.edges.len()
        );
    }

    /// Updates the timestamp to now.
    pub fn touch(&mut self) {
        self.timestamp = persistence::current_timestamp();
    }

    /// Adds or overwrites a metadata key-value pair.
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Returns seconds since last modification.
    #[must_use]
    pub fn age_seconds(&self) -> u64 {
        let now = persistence::current_timestamp();
        now.saturating_sub(self.timestamp)
    }
}

impl SessionStateLike for SessionState {
    fn version(&self) -> u32 {
        self.version
    }
}
