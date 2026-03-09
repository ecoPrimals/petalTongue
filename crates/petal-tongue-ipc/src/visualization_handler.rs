// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for visualization.render and visualization.render.stream IPC methods.
//!
//! These methods allow springs and other primals to push data for rendering
//! without compile-time coupling -- they discover petalTongue at runtime and
//! send DataBinding payloads via JSON-RPC.

use petal_tongue_core::{DataBinding, ThresholdRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

/// UI configuration preset pushed from springs alongside data.
///
/// Allows springs to drive petalTongue panel visibility, mode, and zoom
/// without compile-time coupling (healthSpring V9 SAME DAVE motor channel).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    /// Panel visibility (e.g., "left_sidebar", "audio_panel", "trust_dashboard")
    #[serde(default)]
    pub show_panels: HashMap<String, bool>,
    /// Initial mode (e.g., "clinical", "research", "monitoring")
    #[serde(default)]
    pub mode: Option<String>,
    /// Zoom preset (e.g., "fit", "1.0", "2.0")
    #[serde(default)]
    pub initial_zoom: Option<String>,
    /// Whether to show the awakening sequence
    #[serde(default)]
    pub awakening_enabled: Option<bool>,
    /// Theme override (e.g., "clinical-dark", "ecology-light")
    #[serde(default)]
    pub theme: Option<String>,
}

/// Request payload for visualization.render
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRenderRequest {
    /// Unique identifier for this visualization session
    pub session_id: String,
    /// Human-readable title for the visualization
    pub title: String,
    /// Data bindings to render (the actual data + chart type)
    pub bindings: Vec<DataBinding>,
    /// Optional threshold ranges for status coloring
    #[serde(default)]
    pub thresholds: Vec<ThresholdRange>,
    /// Domain hint for theme selection (e.g., "health", "physics", "ecology")
    #[serde(default)]
    pub domain: Option<String>,
    /// Optional UI configuration preset from the pushing spring
    #[serde(default)]
    pub ui_config: Option<UiConfig>,
}

/// Response for visualization.render
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRenderResponse {
    /// Session ID (echoed back for tracking)
    pub session_id: String,
    /// Number of bindings accepted
    pub bindings_accepted: usize,
    /// Current rendering status
    pub status: String,
}

/// Request payload for visualization.render.stream (incremental update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdateRequest {
    /// Session ID to update (must match an existing render session)
    pub session_id: String,
    /// Which binding to update (by id)
    pub binding_id: String,
    /// The update operation
    pub operation: StreamOperation,
}

/// Types of incremental stream updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamOperation {
    /// Append new data points to a TimeSeries or Spectrum
    #[serde(rename = "append")]
    Append {
        /// X-axis values (timestamps for TimeSeries, frequencies for Spectrum)
        x_values: Vec<f64>,
        /// Y-axis values (measurements for TimeSeries, amplitudes for Spectrum)
        y_values: Vec<f64>,
    },
    /// Replace the current value of a Gauge
    #[serde(rename = "set_value")]
    SetValue {
        /// New gauge value
        value: f64,
    },
    /// Replace the full binding (for Heatmap, FieldMap, etc.)
    #[serde(rename = "replace")]
    Replace {
        /// Replacement binding
        binding: DataBinding,
    },
}

/// Response for visualization.render.stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdateResponse {
    /// Session ID (echoed back)
    pub session_id: String,
    /// Binding ID that was updated
    pub binding_id: String,
    /// Whether the update was accepted
    pub accepted: bool,
}

/// Manages active visualization sessions from springs/primals
pub struct VisualizationState {
    sessions: HashMap<String, RenderSession>,
}

/// A single visualization session with its bindings and metadata
pub struct RenderSession {
    /// Human-readable title for the visualization
    pub title: String,
    /// Data bindings to render
    pub bindings: Vec<DataBinding>,
    /// Optional threshold ranges for status coloring
    pub thresholds: Vec<ThresholdRange>,
    /// Domain hint for theme selection
    pub domain: Option<String>,
    /// UI configuration preset from the pushing spring
    pub ui_config: Option<UiConfig>,
    /// Last update timestamp
    pub updated_at: std::time::Instant,
}

impl VisualizationState {
    /// Create a new empty visualization state
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Handle a visualization.render request, creating or replacing a session
    pub fn handle_render(
        &mut self,
        req: VisualizationRenderRequest,
    ) -> VisualizationRenderResponse {
        let count = req.bindings.len();
        self.sessions.insert(
            req.session_id.clone(),
            RenderSession {
                title: req.title,
                bindings: req.bindings,
                thresholds: req.thresholds,
                domain: req.domain,
                ui_config: req.ui_config,
                updated_at: std::time::Instant::now(),
            },
        );
        VisualizationRenderResponse {
            session_id: req.session_id,
            bindings_accepted: count,
            status: "rendering".to_string(),
        }
    }

    /// Handle a visualization.render.stream incremental update
    pub fn handle_stream_update(&mut self, req: StreamUpdateRequest) -> StreamUpdateResponse {
        let accepted = if let Some(session) = self.sessions.get_mut(&req.session_id) {
            if let Some(binding) = session
                .bindings
                .iter_mut()
                .find(|b| binding_id(b) == req.binding_id)
            {
                apply_operation(binding, &req.operation);
                session.updated_at = std::time::Instant::now();
                true
            } else {
                false
            }
        } else {
            false
        };
        StreamUpdateResponse {
            session_id: req.session_id,
            binding_id: req.binding_id,
            accepted,
        }
    }

    /// Get all active sessions (for rendering)
    #[must_use]
    pub fn sessions(&self) -> &HashMap<String, RenderSession> {
        &self.sessions
    }

    /// Get bindings for a specific session
    #[must_use]
    pub fn session_bindings(&self, session_id: &str) -> Option<&[DataBinding]> {
        self.sessions.get(session_id).map(|s| s.bindings.as_slice())
    }

    /// Get all bindings across all sessions
    #[must_use]
    pub fn all_bindings(&self) -> Vec<&DataBinding> {
        self.sessions
            .values()
            .flat_map(|s| s.bindings.iter())
            .collect()
    }
}

impl Default for VisualizationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the `id` field from any DataBinding variant
#[must_use]
pub fn binding_id(binding: &DataBinding) -> &str {
    match binding {
        DataBinding::TimeSeries { id, .. } => id,
        DataBinding::Distribution { id, .. } => id,
        DataBinding::Bar { id, .. } => id,
        DataBinding::Gauge { id, .. } => id,
        DataBinding::Heatmap { id, .. } => id,
        DataBinding::Scatter3D { id, .. } => id,
        DataBinding::FieldMap { id, .. } => id,
        DataBinding::Spectrum { id, .. } => id,
    }
}

/// Apply a stream operation to a binding in place
pub fn apply_operation(binding: &mut DataBinding, operation: &StreamOperation) {
    match (binding, operation) {
        (
            DataBinding::TimeSeries {
                x_values, y_values, ..
            },
            StreamOperation::Append {
                x_values: new_x,
                y_values: new_y,
            },
        ) => {
            x_values.extend(new_x.iter().copied());
            y_values.extend(new_y.iter().copied());
        }
        (
            DataBinding::Spectrum {
                frequencies,
                amplitudes,
                ..
            },
            StreamOperation::Append {
                x_values: new_freq,
                y_values: new_amp,
            },
        ) => {
            frequencies.extend(new_freq.iter().copied());
            amplitudes.extend(new_amp.iter().copied());
        }
        (DataBinding::Gauge { value, .. }, StreamOperation::SetValue { value: new_val }) => {
            *value = *new_val;
        }
        (b, StreamOperation::Replace { binding: new_b }) => {
            *b = new_b.clone();
        }
        _ => {
            warn!("Mismatched stream operation: operation not applicable to this binding type");
        }
    }
}
