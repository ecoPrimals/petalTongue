// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.render` and UI configuration from springs.

use petal_tongue_core::{DataBinding, ThresholdRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// UI configuration preset pushed from springs alongside data.
///
/// Allows springs to drive petalTongue panel visibility, mode, and zoom
/// without compile-time coupling (healthSpring V9 SAME DAVE motor channel).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    /// Panel visibility (e.g., "`left_sidebar`", "`audio_panel`", "`trust_dashboard`")
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

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::DataBinding;

    #[test]
    fn ui_config_default() {
        let config = UiConfig::default();
        assert!(config.show_panels.is_empty());
        assert!(config.mode.is_none());
        assert!(config.initial_zoom.is_none());
        assert!(config.awakening_enabled.is_none());
        assert!(config.theme.is_none());
    }

    #[test]
    fn ui_config_serialization() {
        let mut config = UiConfig::default();
        config.show_panels.insert("left_sidebar".into(), true);
        config.mode = Some("clinical".into());
        let json = serde_json::to_string(&config).expect("serialize");
        let restored: UiConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.show_panels.get("left_sidebar"), Some(&true));
        assert_eq!(restored.mode.as_deref(), Some("clinical"));
    }

    #[test]
    fn visualization_render_request_roundtrip() {
        let req = VisualizationRenderRequest {
            session_id: "s1".into(),
            title: "Test".into(),
            bindings: vec![DataBinding::TimeSeries {
                id: "ts1".into(),
                label: "Series".into(),
                x_label: "t".into(),
                y_label: "v".into(),
                unit: String::new(),
                x_values: vec![1.0, 2.0],
                y_values: vec![10.0, 20.0],
            }],
            thresholds: vec![],
            domain: Some("health".into()),
            ui_config: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: VisualizationRenderRequest =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.session_id, "s1");
        assert_eq!(restored.bindings.len(), 1);
    }

    #[test]
    fn visualization_render_response_roundtrip() {
        let resp = VisualizationRenderResponse {
            session_id: "s1".into(),
            bindings_accepted: 3,
            status: "rendering".into(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: VisualizationRenderResponse =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.bindings_accepted, 3);
    }
}
