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
    ConstraintResult, DashboardRenderRequest, DashboardRenderResponse, DismissRequest,
    DismissResponse, ExportRequest, ExportResponse, GrammarRenderRequest, GrammarRenderResponse,
    StreamUpdateRequest, StreamUpdateResponse, UiConfig, ValidateRequest, ValidateResponse,
    VisualizationRenderRequest, VisualizationRenderResponse,
};

/// Manages active visualization sessions from springs/primals
pub struct VisualizationState {
    sessions: std::collections::HashMap<String, RenderSession>,
    grammar_scenes: std::collections::HashMap<String, petal_tongue_scene::scene_graph::SceneGraph>,
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
            sessions: std::collections::HashMap::new(),
            grammar_scenes: std::collections::HashMap::new(),
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
                title: req.title.clone(),
                bindings: req.bindings.clone(),
                thresholds: req.thresholds.clone(),
                domain: req.domain.clone(),
                ui_config: req.ui_config.clone(),
                updated_at: std::time::Instant::now(),
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
    pub fn sessions(&self) -> &std::collections::HashMap<String, RenderSession> {
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
            let report = TufteReport::evaluate_all(&constraints, &primitives, &req.grammar);
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
        let report = TufteReport::evaluate_all(&constraints, &primitives, &req.grammar);

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
