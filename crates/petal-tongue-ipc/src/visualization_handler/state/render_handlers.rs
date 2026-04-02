// SPDX-License-Identifier: AGPL-3.0-or-later
//! Render handlers: grammar, dashboard, validate, export.
//!
//! Compiles scene graphs, runs Tufte validation, and exports to modalities.

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::tufte::{ChartjunkDetection, DataInkRatio, TufteConstraint, TufteReport};

use super::super::modality;
use super::super::types::{
    ConstraintResult, DashboardRenderRequest, DashboardRenderResponse, ExportRequest,
    ExportResponse, GrammarRenderRequest, GrammarRenderResponse, ValidateRequest, ValidateResponse,
};
use super::types::VisualizationState;

impl VisualizationState {
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
    ///
    /// For `GameScene`/`Soundscape` bindings with description, audio, or haptic
    /// modalities, produces rich semantic output directly from the binding data
    /// rather than the generic scene graph.
    #[must_use]
    pub fn handle_export(&self, req: ExportRequest) -> ExportResponse {
        // Try binding-aware modality compilation first (richer for GameScene/Soundscape)
        if let Some(binding) = self
            .sessions
            .get(&req.session_id)
            .and_then(|s| s.bindings.first())
        {
            let scene_key = format!(
                "{}:{}",
                req.session_id,
                super::super::stream::binding_id(binding)
            );
            if let Some(scene) = self.grammar_scenes.get(&scene_key) {
                let (output, format) =
                    modality::compile_binding_modality(binding, scene, &req.format);
                return ExportResponse {
                    session_id: req.session_id,
                    format,
                    content: match output {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    },
                };
            }
        }
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
}
