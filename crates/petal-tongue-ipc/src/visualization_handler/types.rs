// SPDX-License-Identifier: AGPL-3.0-only
//! Request and response types for visualization IPC methods.
//!
//! These DTOs define the JSON-RPC contract for visualization.render,
//! visualization.render.stream, visualization.render.grammar, visualization.validate,
//! visualization.export, visualization.dismiss, and visualization.interact.*.

use petal_tongue_core::{DataBinding, ThresholdRange};
use petal_tongue_scene::grammar::GrammarExpr;
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
    /// Append new data points to a `TimeSeries` or Spectrum
    #[serde(rename = "append")]
    Append {
        /// X-axis values (timestamps for `TimeSeries`, frequencies for Spectrum)
        x_values: Vec<f64>,
        /// Y-axis values (measurements for `TimeSeries`, amplitudes for Spectrum)
        y_values: Vec<f64>,
    },
    /// Replace the current value of a Gauge
    #[serde(rename = "set_value")]
    SetValue {
        /// New gauge value
        value: f64,
    },
    /// Replace the full binding (for Heatmap, `FieldMap`, etc.)
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
    /// Whether the server is experiencing backpressure (springs should throttle)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub backpressure_active: bool,
}

/// Server-side backpressure configuration for stream rate limiting.
///
/// Matches the client-side `BackpressureConfig` from wetSpring/healthSpring.
/// When a session receives updates faster than the budget allows, the server
/// signals `backpressure_active: true` so springs can throttle.
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum updates per second per session before entering backpressure.
    pub max_updates_per_sec: u32,
    /// Cooldown duration after entering backpressure state.
    pub cooldown: std::time::Duration,
    /// Consecutive fast updates before activating backpressure.
    pub burst_tolerance: u32,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_updates_per_sec: 120,
            cooldown: std::time::Duration::from_millis(200),
            burst_tolerance: 10,
        }
    }
}

/// Request for `visualization.session.status`: query session health metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusRequest {
    /// Session ID to query.
    pub session_id: String,
}

/// Response for `visualization.session.status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Whether the session exists.
    pub exists: bool,
    /// Total stream updates received by this session.
    pub frame_count: u64,
    /// Seconds since last update.
    pub last_update_secs: f64,
    /// Whether backpressure is currently active.
    pub backpressure_active: bool,
    /// Number of bindings in the session.
    pub binding_count: usize,
    /// Domain hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// Request payload for `visualization.render.grammar` (declarative scene engine path).
///
/// Springs send a `GrammarExpr` (data source, variable bindings, geometry type) plus
/// raw data. petalTongue compiles this through the scene engine with Tufte validation
/// and returns SVG (or another modality) output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRenderRequest {
    /// Unique session identifier.
    pub session_id: String,
    /// Grammar expression describing the visualization.
    pub grammar: GrammarExpr,
    /// Raw data rows (each row is a JSON object with field values).
    pub data: Vec<serde_json::Value>,
    /// Requested output modality: "svg" (default), "audio", "description".
    #[serde(default = "default_modality")]
    pub modality: String,
    /// Whether to run Tufte constraint validation.
    #[serde(default = "default_true")]
    pub validate_tufte: bool,
    /// Domain hint (e.g. "health", "physics") for constraint tuning.
    #[serde(default)]
    pub domain: Option<String>,
}

fn default_modality() -> String {
    "svg".into()
}
const fn default_true() -> bool {
    true
}

/// Response for `visualization.render.grammar`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRenderResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// The compiled output (SVG string, audio params JSON, or description text).
    pub output: serde_json::Value,
    /// Output modality that was used.
    pub modality: String,
    /// Number of scene graph nodes.
    pub scene_nodes: usize,
    /// Total rendering primitives.
    pub total_primitives: usize,
    /// Tufte constraint report (if validation was requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tufte_report: Option<serde_json::Value>,
}

/// Request for `visualization.validate`: validate grammar + data against Tufte constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    /// Grammar expression to validate.
    pub grammar: GrammarExpr,
    /// Raw data rows for the grammar.
    pub data: Vec<serde_json::Value>,
}

/// Response for `visualization.validate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    /// Overall Tufte score (0.0 to 1.0).
    pub score: f64,
    /// Whether the visualization passed validation.
    pub passed: bool,
    /// Per-constraint results.
    pub constraints: Vec<ConstraintResult>,
}

/// Result of evaluating a single Tufte constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Constraint name (e.g. "`DataInkRatio`", "`ChartjunkDetection`").
    pub name: String,
    /// Numeric score (0.0 to 1.0).
    pub score: f64,
    /// Whether the constraint passed.
    pub passed: bool,
    /// Human-readable details.
    pub details: String,
}

/// Request for `visualization.export`: export a session to a format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Session ID to export.
    pub session_id: String,
    /// Output format: "svg", "json", "description".
    pub format: String,
}

/// Response for `visualization.export`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Format that was used.
    pub format: String,
    /// Exported content (SVG string, JSON, or description text).
    pub content: String,
}

/// Request for `visualization.render.dashboard`: compile all bindings into a multi-panel layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRenderRequest {
    /// Unique session identifier.
    pub session_id: String,
    /// Dashboard title.
    pub title: String,
    /// Data bindings — each becomes a panel.
    pub bindings: Vec<DataBinding>,
    /// Domain hint (e.g. "health", "physics") for theming.
    #[serde(default)]
    pub domain: Option<String>,
    /// Requested output modality: "svg" (default), "description".
    #[serde(default = "default_modality")]
    pub modality: String,
    /// Maximum columns in grid layout.
    #[serde(default = "default_dashboard_columns")]
    pub max_columns: usize,
}

const fn default_dashboard_columns() -> usize {
    3
}

/// Response for `visualization.render.dashboard`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRenderResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// The compiled output (SVG string or description text).
    pub output: serde_json::Value,
    /// Output modality that was used.
    pub modality: String,
    /// Number of panels in the dashboard.
    pub panel_count: usize,
    /// Grid columns.
    pub columns: usize,
    /// Grid rows.
    pub rows: usize,
    /// Total scene graph nodes.
    pub scene_nodes: usize,
    /// Total rendering primitives.
    pub total_primitives: usize,
}

/// Request for `visualization.dismiss`: remove a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissRequest {
    /// Session ID to dismiss.
    pub session_id: String,
}

/// Response for `visualization.dismiss`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Whether the session was removed.
    pub dismissed: bool,
}

/// Request for `visualization.interact.apply`: apply an interaction intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionApplyRequest {
    /// Intent: "select", "focus", "inspect", "filter", "navigate".
    pub intent: String,
    /// Target identifiers to apply the intent to.
    pub targets: Vec<String>,
    /// Optional grammar ID for scoped interactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grammar_id: Option<String>,
}

/// Response for `visualization.interact.apply`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionApplyResponse {
    /// Whether the interaction was accepted.
    pub accepted: bool,
    /// Number of targets resolved.
    pub targets_resolved: usize,
}

/// A visualization perspective (view configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    /// Perspective identifier.
    pub id: String,
    /// Modalities active in this perspective.
    pub modalities: Vec<String>,
    /// Current selection.
    pub selection: Vec<String>,
    /// Sync mode (e.g. "`shared_selection`").
    pub sync_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::DataBinding;
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};
    use serde_json::json;

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

    #[test]
    fn stream_operation_append_roundtrip() {
        let op = StreamOperation::Append {
            x_values: vec![1.0, 2.0],
            y_values: vec![10.0, 20.0],
        };
        let json = serde_json::to_string(&op).expect("serialize");
        let restored: StreamOperation = serde_json::from_str(&json).expect("deserialize");
        match restored {
            StreamOperation::Append { x_values, y_values } => {
                assert_eq!(x_values, vec![1.0, 2.0]);
                assert_eq!(y_values, vec![10.0, 20.0]);
            }
            _ => panic!("expected Append"),
        }
    }

    #[test]
    fn stream_operation_set_value_roundtrip() {
        let op = StreamOperation::SetValue { value: 42.5 };
        let json = serde_json::to_string(&op).expect("serialize");
        let restored: StreamOperation = serde_json::from_str(&json).expect("deserialize");
        match restored {
            StreamOperation::SetValue { value } => assert_eq!(value, 42.5),
            _ => panic!("expected SetValue"),
        }
    }

    #[test]
    fn stream_update_request_roundtrip() {
        let req = StreamUpdateRequest {
            session_id: "s1".into(),
            binding_id: "b1".into(),
            operation: StreamOperation::SetValue { value: 1.0 },
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: StreamUpdateRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.session_id, "s1");
    }

    #[test]
    fn stream_update_response_backpressure_serialization() {
        let resp = StreamUpdateResponse {
            session_id: "s1".into(),
            binding_id: "b1".into(),
            accepted: true,
            backpressure_active: true,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("backpressure_active"));
    }

    #[test]
    fn backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.max_updates_per_sec, 120);
        assert_eq!(config.cooldown, std::time::Duration::from_millis(200));
        assert_eq!(config.burst_tolerance, 10);
    }

    #[test]
    fn session_status_request_response_roundtrip() {
        let req = SessionStatusRequest {
            session_id: "s1".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: SessionStatusRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = SessionStatusResponse {
            session_id: "s1".into(),
            exists: true,
            frame_count: 100,
            last_update_secs: 0.5,
            backpressure_active: false,
            binding_count: 3,
            domain: Some("health".into()),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: SessionStatusResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.frame_count, 100);
    }

    #[test]
    fn grammar_render_request_defaults() {
        let req = GrammarRenderRequest {
            session_id: "s1".into(),
            grammar: GrammarExpr::new("data", GeometryType::Point),
            data: vec![json!({"x": 1, "y": 2})],
            modality: "svg".into(),
            validate_tufte: true,
            domain: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: GrammarRenderRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.modality, "svg");
        assert!(restored.validate_tufte);
    }

    #[test]
    fn grammar_render_response_roundtrip() {
        let resp = GrammarRenderResponse {
            session_id: "s1".into(),
            output: json!("<svg></svg>"),
            modality: "svg".into(),
            scene_nodes: 5,
            total_primitives: 10,
            tufte_report: Some(json!({"score": 0.9})),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: GrammarRenderResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.scene_nodes, 5);
    }

    #[test]
    fn validate_request_response_roundtrip() {
        let req = ValidateRequest {
            grammar: GrammarExpr::new("data", GeometryType::Line),
            data: vec![],
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: ValidateRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = ValidateResponse {
            score: 0.85,
            passed: true,
            constraints: vec![ConstraintResult {
                name: "DataInkRatio".into(),
                score: 0.9,
                passed: true,
                details: "Good".into(),
            }],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: ValidateResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.constraints.len(), 1);
    }

    #[test]
    fn export_request_response_roundtrip() {
        let req = ExportRequest {
            session_id: "s1".into(),
            format: "svg".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: ExportRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = ExportResponse {
            session_id: "s1".into(),
            format: "svg".into(),
            content: "<svg></svg>".into(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: ExportResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.content, "<svg></svg>");
    }

    #[test]
    fn dashboard_render_request_default_columns() {
        let req = DashboardRenderRequest {
            session_id: "s1".into(),
            title: "Dashboard".into(),
            bindings: vec![],
            domain: None,
            modality: "svg".into(),
            max_columns: 3,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: DashboardRenderRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.max_columns, 3);
    }

    #[test]
    fn dashboard_render_response_roundtrip() {
        let resp = DashboardRenderResponse {
            session_id: "s1".into(),
            output: json!("<svg></svg>"),
            modality: "svg".into(),
            panel_count: 2,
            columns: 2,
            rows: 1,
            scene_nodes: 4,
            total_primitives: 8,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: DashboardRenderResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.columns, 2);
    }

    #[test]
    fn dismiss_request_response_roundtrip() {
        let req = DismissRequest {
            session_id: "s1".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: DismissRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = DismissResponse {
            session_id: "s1".into(),
            dismissed: true,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: DismissResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.dismissed);
    }

    #[test]
    fn interaction_apply_request_response_roundtrip() {
        let req = InteractionApplyRequest {
            intent: "select".into(),
            targets: vec!["t1".into()],
            grammar_id: Some("g1".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: InteractionApplyRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.grammar_id, Some("g1".into()));

        let resp = InteractionApplyResponse {
            accepted: true,
            targets_resolved: 1,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: InteractionApplyResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.targets_resolved, 1);
    }

    #[test]
    fn perspective_roundtrip() {
        let p = Perspective {
            id: "p1".into(),
            modalities: vec!["svg".into()],
            selection: vec![],
            sync_mode: "shared_selection".into(),
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let restored: Perspective = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.id, "p1");
    }
}
