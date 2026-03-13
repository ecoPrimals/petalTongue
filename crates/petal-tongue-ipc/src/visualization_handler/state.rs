// SPDX-License-Identifier: AGPL-3.0-only
//! Visualization session state and handler logic.
//!
//! Manages active visualization sessions from springs/primals, including
//! render, stream updates, grammar compilation, validation, export, and dismiss.

use petal_tongue_core::{DataBinding, ThresholdRange};
use petal_tongue_scene::DataBindingCompiler;
use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::tufte::{ChartjunkDetection, DataInkRatio, TufteConstraint, TufteReport};

use super::modality;
use super::stream::{apply_operation, binding_id};
use super::types::{
    BackpressureConfig, ConstraintResult, DashboardRenderRequest, DashboardRenderResponse,
    DismissRequest, DismissResponse, ExportRequest, ExportResponse, GrammarRenderRequest,
    GrammarRenderResponse, SessionStatusRequest, SessionStatusResponse, StreamUpdateRequest,
    StreamUpdateResponse, UiConfig, ValidateRequest, ValidateResponse, VisualizationRenderRequest,
    VisualizationRenderResponse,
};

/// Manages active visualization sessions from springs/primals
pub struct VisualizationState {
    sessions: std::collections::HashMap<String, RenderSession>,
    grammar_scenes: std::collections::HashMap<String, petal_tongue_scene::scene_graph::SceneGraph>,
    backpressure_config: BackpressureConfig,
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
    /// Total stream updates received
    pub frame_count: u64,
    /// Timestamps of recent updates for rate calculation
    recent_updates: std::collections::VecDeque<std::time::Instant>,
    /// Whether backpressure is currently active
    pub backpressure_active: bool,
    /// When backpressure cooldown ends
    cooldown_until: Option<std::time::Instant>,
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

    /// Handle a visualization.render request, creating or replacing a session
    pub fn handle_render(
        &mut self,
        req: VisualizationRenderRequest,
    ) -> VisualizationRenderResponse {
        let count = req.bindings.len();
        self.sessions.insert(
            req.session_id.clone(),
            RenderSession {
                title: req.title.clone(),
                bindings: req.bindings.clone(),
                thresholds: req.thresholds.clone(),
                domain: req.domain.clone(),
                ui_config: req.ui_config.clone(),
                updated_at: std::time::Instant::now(),
                frame_count: 0,
                recent_updates: std::collections::VecDeque::with_capacity(128),
                backpressure_active: false,
                cooldown_until: None,
            },
        );
        // Auto-compile DataBindings to scene graphs via Grammar of Graphics
        if !req.bindings.is_empty() {
            let compiler = GrammarCompiler::new();
            let domain = req.domain.as_deref();
            for binding in &req.bindings {
                let (grammar_expr, data) = DataBindingCompiler::compile(binding, domain);
                let scene = compiler.compile(&grammar_expr, &data);
                let binding_key = format!("{}:{}", req.session_id, binding_id(binding));
                self.grammar_scenes.insert(binding_key, scene);
            }
        }
        VisualizationRenderResponse {
            session_id: req.session_id,
            bindings_accepted: count,
            status: "rendering".to_string(),
        }
    }

    /// Handle a visualization.render.stream incremental update.
    ///
    /// Enforces server-side backpressure: when a session exceeds the configured
    /// update rate, `backpressure_active` is set in the response so springs can
    /// throttle. Updates are still accepted during backpressure to avoid data loss.
    pub fn handle_stream_update(&mut self, req: StreamUpdateRequest) -> StreamUpdateResponse {
        let max_rate = self.backpressure_config.max_updates_per_sec;
        let cooldown = self.backpressure_config.cooldown;
        let burst_tolerance = self.backpressure_config.burst_tolerance;

        let (accepted, bp_active) = if let Some(session) = self.sessions.get_mut(&req.session_id) {
            let now = std::time::Instant::now();

            // Check cooldown
            if let Some(until) = session.cooldown_until {
                if now < until {
                    // Still in cooldown — accept but signal backpressure
                    if let Some(binding) = session
                        .bindings
                        .iter_mut()
                        .find(|b| binding_id(b) == req.binding_id)
                    {
                        apply_operation(binding, &req.operation);
                        session.updated_at = now;
                        session.frame_count += 1;
                        return StreamUpdateResponse {
                            session_id: req.session_id,
                            binding_id: req.binding_id,
                            accepted: true,
                            backpressure_active: true,
                        };
                    }
                    return StreamUpdateResponse {
                        session_id: req.session_id,
                        binding_id: req.binding_id,
                        accepted: false,
                        backpressure_active: true,
                    };
                }
                session.cooldown_until = None;
                session.backpressure_active = false;
            }

            // Sliding window: remove entries older than 1 second
            let one_sec_ago = now
                .checked_sub(std::time::Duration::from_secs(1))
                .unwrap_or(now);
            while session
                .recent_updates
                .front()
                .is_some_and(|&t| t < one_sec_ago)
            {
                session.recent_updates.pop_front();
            }

            // Check rate
            let rate_exceeded =
                session.recent_updates.len() as u32 >= max_rate.saturating_add(burst_tolerance);
            if rate_exceeded && !session.backpressure_active {
                session.backpressure_active = true;
                session.cooldown_until = Some(now + cooldown);
            }

            session.recent_updates.push_back(now);

            if let Some(binding) = session
                .bindings
                .iter_mut()
                .find(|b| binding_id(b) == req.binding_id)
            {
                apply_operation(binding, &req.operation);
                session.updated_at = now;
                session.frame_count += 1;
                (true, session.backpressure_active)
            } else {
                (false, session.backpressure_active)
            }
        } else {
            (false, false)
        };
        StreamUpdateResponse {
            session_id: req.session_id,
            binding_id: req.binding_id,
            accepted,
            backpressure_active: bp_active,
        }
    }

    /// Handle `visualization.session.status`: return session health metrics.
    #[must_use]
    pub fn handle_session_status(&self, req: &SessionStatusRequest) -> SessionStatusResponse {
        self.sessions.get(&req.session_id).map_or_else(
            || SessionStatusResponse {
                session_id: req.session_id.clone(),
                exists: false,
                frame_count: 0,
                last_update_secs: 0.0,
                backpressure_active: false,
                binding_count: 0,
                domain: None,
            },
            |session| SessionStatusResponse {
                session_id: req.session_id.clone(),
                exists: true,
                frame_count: session.frame_count,
                last_update_secs: session.updated_at.elapsed().as_secs_f64(),
                backpressure_active: session.backpressure_active,
                binding_count: session.bindings.len(),
                domain: session.domain.clone(),
            },
        )
    }

    /// Get all active sessions (for rendering)
    #[must_use]
    pub const fn sessions(&self) -> &std::collections::HashMap<String, RenderSession> {
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

    /// Handle `visualization.render.grammar`: compile a grammar expression through
    /// the scene engine with Tufte validation and return the requested modality output.
    pub fn handle_grammar_render(&mut self, req: GrammarRenderRequest) -> GrammarRenderResponse {
        let compiler = GrammarCompiler;
        let scene = compiler.compile(&req.grammar, &req.data);

        let tufte_report = if req.validate_tufte {
            let primitives: Vec<_> = scene
                .flatten()
                .into_iter()
                .map(|(_, p)| p.clone())
                .collect();
            let data_ink = DataInkRatio;
            let constraints: Vec<&dyn TufteConstraint> = vec![&data_ink];
            let report =
                TufteReport::evaluate_all(&constraints, &primitives, &req.grammar, Some(&req.data));
            serde_json::to_value(&report).ok()
        } else {
            None
        };

        let node_count = scene.node_count();
        let prim_count = scene.total_primitives();

        let (output, modality) = modality::compile_modality(&scene, &req.modality);

        // Store scene metadata in a grammar session for future streaming/updates
        self.grammar_scenes.insert(req.session_id.clone(), scene);

        GrammarRenderResponse {
            session_id: req.session_id,
            output,
            modality,
            scene_nodes: node_count,
            total_primitives: prim_count,
            tufte_report,
        }
    }

    /// Get a compiled scene graph by session ID (for downstream rendering).
    #[must_use]
    pub fn grammar_scene(
        &self,
        session_id: &str,
    ) -> Option<&petal_tongue_scene::scene_graph::SceneGraph> {
        self.grammar_scenes.get(session_id)
    }

    /// Handle `visualization.render.dashboard`: compile all bindings into a multi-panel
    /// layout and return the composed scene as SVG (or another modality).
    pub fn handle_dashboard_render(
        &mut self,
        req: DashboardRenderRequest,
    ) -> DashboardRenderResponse {
        let config = petal_tongue_scene::DashboardConfig {
            layout: petal_tongue_scene::DashboardLayout::Grid {
                max_columns: req.max_columns,
            },
            title: Some(req.title),
            domain: req.domain.clone(),
            ..petal_tongue_scene::DashboardConfig::default()
        };

        let dashboard = petal_tongue_scene::build_dashboard(&req.bindings, &config);
        let node_count = dashboard.scene.node_count();
        let prim_count = dashboard.scene.total_primitives();

        let (output, modality_used) = modality::compile_modality(&dashboard.scene, &req.modality);

        self.grammar_scenes
            .insert(req.session_id.clone(), dashboard.scene);

        DashboardRenderResponse {
            session_id: req.session_id,
            output,
            modality: modality_used,
            panel_count: dashboard.panel_count,
            columns: dashboard.columns,
            rows: dashboard.rows,
            scene_nodes: node_count,
            total_primitives: prim_count,
        }
    }

    /// Handle `visualization.validate`: validate grammar + data against Tufte constraints.
    #[must_use]
    pub fn handle_validate(&self, req: &ValidateRequest) -> ValidateResponse {
        let compiler = GrammarCompiler;
        let scene = compiler.compile(&req.grammar, &req.data);
        let primitives: Vec<_> = scene
            .flatten()
            .into_iter()
            .map(|(_, p)| p.clone())
            .collect();

        let data_ink = DataInkRatio;
        let chartjunk = ChartjunkDetection;
        let constraints: Vec<&dyn TufteConstraint> = vec![&data_ink, &chartjunk];
        let report =
            TufteReport::evaluate_all(&constraints, &primitives, &req.grammar, Some(&req.data));

        let constraints_out: Vec<ConstraintResult> = report
            .results
            .iter()
            .map(|(name, r)| ConstraintResult {
                name: name.clone(),
                score: r.score,
                passed: r.passed,
                details: r.message.clone(),
            })
            .collect();

        ValidateResponse {
            score: report.overall_score,
            passed: report.overall_score >= 0.5,
            constraints: constraints_out,
        }
    }

    /// Handle `visualization.export`: export a session to the requested format.
    #[must_use]
    pub fn handle_export(&self, req: ExportRequest) -> ExportResponse {
        if let Some(scene) = self.grammar_scenes.get(&req.session_id) {
            let (output, format) = modality::compile_modality(scene, &req.format);
            ExportResponse {
                session_id: req.session_id,
                format,
                content: match output {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                },
            }
        } else {
            ExportResponse {
                session_id: req.session_id,
                format: req.format,
                content: String::new(),
            }
        }
    }

    /// Handle `visualization.dismiss`: remove a session.
    pub fn handle_dismiss(&mut self, req: DismissRequest) -> DismissResponse {
        let removed_session = self.sessions.remove(&req.session_id).is_some();
        let removed_scene = self.grammar_scenes.remove(&req.session_id).is_some();
        // Remove DataBinding-compiled scenes (keys: "session_id:binding_id")
        let prefix = format!("{}:", req.session_id);
        let binding_scenes: Vec<String> = self
            .grammar_scenes
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        let removed_binding_scenes = !binding_scenes.is_empty();
        for k in binding_scenes {
            self.grammar_scenes.remove(&k);
        }
        DismissResponse {
            session_id: req.session_id,
            dismissed: removed_session || removed_scene || removed_binding_scenes,
        }
    }
}

impl Default for VisualizationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization_handler::types::{BackpressureConfig, StreamOperation};
    use petal_tongue_core::DataBinding;
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};

    fn make_timeseries(id: &str) -> DataBinding {
        DataBinding::TimeSeries {
            id: id.to_string(),
            label: "Test".to_string(),
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            unit: String::new(),
            x_values: vec![0.0, 1.0, 2.0],
            y_values: vec![10.0, 20.0, 30.0],
        }
    }

    fn make_gauge(id: &str, value: f64) -> DataBinding {
        DataBinding::Gauge {
            id: id.to_string(),
            label: "Gauge".to_string(),
            value,
            min: 0.0,
            max: 100.0,
            unit: "%".to_string(),
            normal_range: [20.0, 80.0],
            warning_range: [10.0, 90.0],
        }
    }

    #[test]
    fn test_visualization_state_new_and_default() {
        let state = VisualizationState::new();
        assert!(state.sessions().is_empty());
        assert!(state.all_bindings().is_empty());

        let default = VisualizationState::default();
        assert!(default.sessions().is_empty());
    }

    #[test]
    fn test_handle_render_empty_bindings() {
        let mut state = VisualizationState::new();
        let req = VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "Empty".to_string(),
            bindings: vec![],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        };
        let resp = state.handle_render(req);
        assert_eq!(resp.session_id, "s1");
        assert_eq!(resp.bindings_accepted, 0);
        assert_eq!(resp.status, "rendering");
        assert_eq!(state.sessions().len(), 1);
        assert!(state.grammar_scene("s1").is_none());
    }

    #[test]
    fn test_handle_render_with_bindings() {
        let mut state = VisualizationState::new();
        let bindings = vec![make_timeseries("b1"), make_gauge("b2", 50.0)];
        let req = VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "Dashboard".to_string(),
            bindings,
            thresholds: vec![],
            domain: Some("health".to_string()),
            ui_config: None,
        };
        let resp = state.handle_render(req);
        assert_eq!(resp.bindings_accepted, 2);
        let session = state.sessions().get("s1").expect("session exists");
        assert_eq!(session.bindings.len(), 2);
        assert_eq!(session.domain.as_deref(), Some("health"));
        assert!(state.grammar_scene("s1").is_none());
        assert!(state.grammar_scene("s1:b1").is_some());
        assert!(state.grammar_scene("s1:b2").is_some());
    }

    #[test]
    fn test_session_bindings_and_all_bindings() {
        let mut state = VisualizationState::new();
        let req = VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        };
        state.handle_render(req);
        assert_eq!(state.session_bindings("s1").map(<[_]>::len), Some(1));
        assert!(state.session_bindings("nonexistent").is_none());
        assert_eq!(state.all_bindings().len(), 1);
    }

    #[test]
    fn test_handle_stream_update_session_not_found() {
        let mut state = VisualizationState::new();
        let req = StreamUpdateRequest {
            session_id: "nonexistent".to_string(),
            binding_id: "b1".to_string(),
            operation: StreamOperation::Append {
                x_values: vec![3.0],
                y_values: vec![40.0],
            },
        };
        let resp = state.handle_stream_update(req);
        assert!(!resp.accepted);
    }

    #[test]
    fn test_handle_stream_update_binding_not_found() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let req = StreamUpdateRequest {
            session_id: "s1".to_string(),
            binding_id: "b99".to_string(),
            operation: StreamOperation::Append {
                x_values: vec![3.0],
                y_values: vec![40.0],
            },
        };
        let resp = state.handle_stream_update(req);
        assert!(!resp.accepted);
    }

    #[test]
    fn test_handle_stream_update_append_timeseries() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let req = StreamUpdateRequest {
            session_id: "s1".to_string(),
            binding_id: "b1".to_string(),
            operation: StreamOperation::Append {
                x_values: vec![3.0],
                y_values: vec![40.0],
            },
        };
        let resp = state.handle_stream_update(req);
        assert!(resp.accepted);
        let bindings = state.session_bindings("s1").expect("session exists");
        let b = &bindings[0];
        if let DataBinding::TimeSeries {
            x_values, y_values, ..
        } = b
        {
            assert_eq!(x_values.len(), 4);
            assert_eq!(y_values.len(), 4);
            assert!((y_values[3] - 40.0).abs() < f64::EPSILON);
        } else {
            panic!("expected TimeSeries");
        }
    }

    #[test]
    fn test_handle_stream_update_set_value_gauge() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_gauge("g1", 50.0)],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let req = StreamUpdateRequest {
            session_id: "s1".to_string(),
            binding_id: "g1".to_string(),
            operation: StreamOperation::SetValue { value: 75.0 },
        };
        let resp = state.handle_stream_update(req);
        assert!(resp.accepted);
        let bindings = state.session_bindings("s1").expect("session exists");
        if let DataBinding::Gauge { value, .. } = &bindings[0] {
            assert!((*value - 75.0).abs() < f64::EPSILON);
        } else {
            panic!("expected Gauge");
        }
    }

    #[test]
    fn test_handle_grammar_render() {
        let mut state = VisualizationState::new();
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let data = vec![
            serde_json::json!({"x": 1.0, "y": 2.0}),
            serde_json::json!({"x": 3.0, "y": 4.0}),
        ];
        let req = GrammarRenderRequest {
            session_id: "gram-s1".to_string(),
            grammar,
            data,
            modality: "svg".to_string(),
            validate_tufte: true,
            domain: None,
        };
        let resp = state.handle_grammar_render(req);
        assert_eq!(resp.session_id, "gram-s1");
        assert_eq!(resp.modality, "svg");
        assert!(resp.tufte_report.is_some());
        assert!(state.grammar_scene("gram-s1").is_some());
    }

    #[test]
    fn test_handle_grammar_render_no_tufte() {
        let mut state = VisualizationState::new();
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let req = GrammarRenderRequest {
            session_id: "g2".to_string(),
            grammar,
            data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
            modality: "description".to_string(),
            validate_tufte: false,
            domain: None,
        };
        let resp = state.handle_grammar_render(req);
        assert!(resp.tufte_report.is_none());
        assert_eq!(resp.modality, "description");
    }

    #[test]
    fn test_handle_dashboard_render() {
        let mut state = VisualizationState::new();
        let req = DashboardRenderRequest {
            session_id: "dash1".to_string(),
            title: "Dashboard".to_string(),
            bindings: vec![make_timeseries("b1"), make_gauge("b2", 60.0)],
            domain: None,
            modality: "svg".to_string(),
            max_columns: 2,
        };
        let resp = state.handle_dashboard_render(req);
        assert_eq!(resp.session_id, "dash1");
        assert_eq!(resp.panel_count, 2);
        assert!(state.grammar_scene("dash1").is_some());
    }

    #[test]
    fn test_handle_validate() {
        let state = VisualizationState::new();
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let req = ValidateRequest {
            grammar,
            data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
        };
        let resp = state.handle_validate(&req);
        assert!(!resp.constraints.is_empty());
        assert!(resp.score >= 0.0 && resp.score <= 1.0);
    }

    #[test]
    fn test_handle_export_session_not_found() {
        let state = VisualizationState::new();
        let req = ExportRequest {
            session_id: "missing".to_string(),
            format: "svg".to_string(),
        };
        let resp = state.handle_export(req);
        assert_eq!(resp.session_id, "missing");
        assert!(resp.content.is_empty());
    }

    #[test]
    fn test_handle_export_session_found() {
        let mut state = VisualizationState::new();
        state.handle_grammar_render(GrammarRenderRequest {
            session_id: "ex1".to_string(),
            grammar: GrammarExpr::new("data", GeometryType::Point),
            data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
            modality: "svg".to_string(),
            validate_tufte: false,
            domain: None,
        });
        let req = ExportRequest {
            session_id: "ex1".to_string(),
            format: "svg".to_string(),
        };
        let resp = state.handle_export(req);
        assert!(!resp.content.is_empty());
        assert!(resp.content.contains("svg") || resp.content.contains('<'));
    }

    #[test]
    fn test_handle_dismiss_session_exists() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "d1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let req = DismissRequest {
            session_id: "d1".to_string(),
        };
        let resp = state.handle_dismiss(req);
        assert!(resp.dismissed);
        assert!(state.sessions().get("d1").is_none());
    }

    #[test]
    fn test_handle_dismiss_session_not_found() {
        let mut state = VisualizationState::new();
        let req = DismissRequest {
            session_id: "nonexistent".to_string(),
        };
        let resp = state.handle_dismiss(req);
        assert!(!resp.dismissed);
    }

    #[test]
    fn test_handle_render_replaces_existing_session() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "First".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let resp = state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "Replaced".to_string(),
            bindings: vec![make_gauge("g1", 25.0)],
            thresholds: vec![],
            domain: Some("physics".to_string()),
            ui_config: None,
        });
        assert_eq!(resp.bindings_accepted, 1);
        let session = state.sessions().get("s1").expect("session exists");
        assert_eq!(session.title, "Replaced");
        assert_eq!(session.bindings.len(), 1);
        assert_eq!(session.domain.as_deref(), Some("physics"));
    }

    #[test]
    fn test_handle_stream_update_replace_operation() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_gauge("g1", 50.0)],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let new_gauge = make_gauge("g1", 99.0);
        let req = StreamUpdateRequest {
            session_id: "s1".to_string(),
            binding_id: "g1".to_string(),
            operation: StreamOperation::Replace { binding: new_gauge },
        };
        let resp = state.handle_stream_update(req);
        assert!(resp.accepted);
        let bindings = state.session_bindings("s1").expect("session exists");
        if let DataBinding::Gauge { value, .. } = &bindings[0] {
            assert!((*value - 99.0).abs() < f64::EPSILON);
        } else {
            panic!("expected Gauge");
        }
    }

    #[test]
    fn test_handle_export_json_format() {
        let mut state = VisualizationState::new();
        state.handle_grammar_render(GrammarRenderRequest {
            session_id: "ex2".to_string(),
            grammar: GrammarExpr::new("data", GeometryType::Point),
            data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
            modality: "json".to_string(),
            validate_tufte: false,
            domain: None,
        });
        let req = ExportRequest {
            session_id: "ex2".to_string(),
            format: "json".to_string(),
        };
        let resp = state.handle_export(req);
        assert_eq!(resp.session_id, "ex2");
        assert!(!resp.content.is_empty());
    }

    #[test]
    fn test_handle_dismiss_grammar_scene_only() {
        let mut state = VisualizationState::new();
        state.handle_grammar_render(GrammarRenderRequest {
            session_id: "gram-only".to_string(),
            grammar: GrammarExpr::new("data", GeometryType::Point),
            data: vec![serde_json::json!({"x": 1.0})],
            modality: "svg".to_string(),
            validate_tufte: false,
            domain: None,
        });
        let req = DismissRequest {
            session_id: "gram-only".to_string(),
        };
        let resp = state.handle_dismiss(req);
        assert!(resp.dismissed, "dismiss should remove grammar scene");
        assert!(state.grammar_scene("gram-only").is_none());
    }

    #[test]
    fn test_handle_validate_passed_threshold() {
        let state = VisualizationState::new();
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let req = ValidateRequest {
            grammar,
            data: vec![
                serde_json::json!({"x": 1.0, "y": 2.0}),
                serde_json::json!({"x": 3.0, "y": 4.0}),
            ],
        };
        let resp = state.handle_validate(&req);
        assert!(resp.score >= 0.0 && resp.score <= 1.0);
        assert_eq!(resp.passed, resp.score >= 0.5);
        for c in &resp.constraints {
            assert!(!c.name.is_empty());
        }
    }

    #[test]
    fn test_render_session_has_updated_at() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        let session = state.sessions().get("s1").expect("session exists");
        let _ = session.updated_at;
    }

    #[test]
    fn test_handle_render_with_ui_config() {
        let mut config = std::collections::HashMap::new();
        config.insert("left_sidebar".to_string(), true);
        let mut state = VisualizationState::new();
        let req = VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "With Config".to_string(),
            bindings: vec![],
            thresholds: vec![],
            domain: None,
            ui_config: Some(crate::visualization_handler::types::UiConfig {
                show_panels: config,
                mode: Some("clinical".to_string()),
                initial_zoom: Some("1.0".to_string()),
                awakening_enabled: Some(true),
                theme: Some("clinical-dark".to_string()),
            }),
        };
        let resp = state.handle_render(req);
        assert_eq!(resp.session_id, "s1");
        let session = state.sessions().get("s1").expect("session exists");
        assert_eq!(
            session.ui_config.as_ref().expect("config").mode.as_deref(),
            Some("clinical")
        );
    }

    #[test]
    fn test_handle_session_status_nonexistent() {
        let state = VisualizationState::new();
        let req = SessionStatusRequest {
            session_id: "nonexistent".to_string(),
        };
        let resp = state.handle_session_status(&req);
        assert!(!resp.exists);
        assert_eq!(resp.session_id, "nonexistent");
        assert_eq!(resp.frame_count, 0);
        assert_eq!(resp.binding_count, 0);
        assert!(!resp.backpressure_active);
    }

    #[test]
    fn test_handle_session_status_exists() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "s1".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: Some("health".to_string()),
            ui_config: None,
        });
        let req = SessionStatusRequest {
            session_id: "s1".to_string(),
        };
        let resp = state.handle_session_status(&req);
        assert!(resp.exists);
        assert_eq!(resp.session_id, "s1");
        assert_eq!(resp.binding_count, 1);
        assert_eq!(resp.domain.as_deref(), Some("health"));
    }

    #[test]
    fn test_with_backpressure_config() {
        let config = BackpressureConfig {
            max_updates_per_sec: 10,
            cooldown: std::time::Duration::from_millis(100),
            burst_tolerance: 2,
        };
        let mut state = VisualizationState::new().with_backpressure(config);
        state.handle_render(VisualizationRenderRequest {
            session_id: "bp".to_string(),
            title: "BP".to_string(),
            bindings: vec![make_timeseries("b1")],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        for _ in 0..15 {
            let _ = state.handle_stream_update(StreamUpdateRequest {
                session_id: "bp".to_string(),
                binding_id: "b1".to_string(),
                operation: StreamOperation::Append {
                    x_values: vec![1.0],
                    y_values: vec![1.0],
                },
            });
        }
        let req = SessionStatusRequest {
            session_id: "bp".to_string(),
        };
        let resp = state.handle_session_status(&req);
        assert!(resp.exists);
    }

    #[test]
    fn test_handle_dismiss_removes_binding_scenes() {
        let mut state = VisualizationState::new();
        state.handle_render(VisualizationRenderRequest {
            session_id: "ds".to_string(),
            title: "T".to_string(),
            bindings: vec![make_timeseries("b1"), make_gauge("b2", 50.0)],
            thresholds: vec![],
            domain: None,
            ui_config: None,
        });
        assert!(state.grammar_scene("ds:b1").is_some());
        assert!(state.grammar_scene("ds:b2").is_some());
        let req = DismissRequest {
            session_id: "ds".to_string(),
        };
        let resp = state.handle_dismiss(req);
        assert!(resp.dismissed);
        assert!(state.grammar_scene("ds:b1").is_none());
        assert!(state.grammar_scene("ds:b2").is_none());
        assert!(state.sessions().get("ds").is_none());
    }
}
