// SPDX-License-Identifier: AGPL-3.0-only
//! Core state types for visualization sessions.
//!
//! `VisualizationState` holds sessions and grammar scenes. `RenderSession`
//! tracks a single visualization's bindings, metadata, and backpressure state.

use super::super::types::BackpressureConfig;

/// Manages active visualization sessions from springs/primals
pub struct VisualizationState {
    /// Active visualization sessions keyed by session ID.
    pub sessions: std::collections::HashMap<String, RenderSession>,
    /// Grammar-compiled scene graphs keyed by `"session_id:binding_id"`.
    pub grammar_scenes:
        std::collections::HashMap<String, petal_tongue_scene::scene_graph::SceneGraph>,
    pub(super) backpressure_config: BackpressureConfig,
}

/// A single visualization session with its bindings and metadata
pub struct RenderSession {
    /// Human-readable title for the visualization
    pub title: String,
    /// Data bindings to render
    pub bindings: Vec<petal_tongue_core::DataBinding>,
    /// Optional threshold ranges for status coloring
    pub thresholds: Vec<petal_tongue_core::ThresholdRange>,
    /// Domain hint for theme selection
    pub domain: Option<String>,
    /// UI configuration preset from the pushing spring
    pub ui_config: Option<super::super::types::UiConfig>,
    /// Last update timestamp
    pub updated_at: std::time::Instant,
    /// Total stream updates received
    pub frame_count: u64,
    /// Timestamps of recent updates for rate calculation
    pub(super) recent_updates: std::collections::VecDeque<std::time::Instant>,
    /// Whether backpressure is currently active
    pub backpressure_active: bool,
    /// When backpressure cooldown ends
    pub(super) cooldown_until: Option<std::time::Instant>,
}

impl VisualizationState {
    /// Create a new empty visualization state
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            grammar_scenes: std::collections::HashMap::new(),
            backpressure_config: BackpressureConfig::default(),
        }
    }

    /// Create with a custom backpressure configuration.
    #[must_use]
    pub const fn with_backpressure(mut self, config: BackpressureConfig) -> Self {
        self.backpressure_config = config;
        self
    }
}

impl Default for VisualizationState {
    fn default() -> Self {
        Self::new()
    }
}
