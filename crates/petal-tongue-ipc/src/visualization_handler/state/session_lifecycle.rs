// SPDX-License-Identifier: AGPL-3.0-only
//! Session lifecycle: create and dismiss visualization sessions.
//!
//! Handles `visualization.render` (create/replace) and `visualization.dismiss` (remove).

use petal_tongue_scene::DataBindingCompiler;
use petal_tongue_scene::compiler::GrammarCompiler;

use super::super::stream::binding_id;
use super::super::types::{
    DismissRequest, DismissResponse, VisualizationRenderRequest, VisualizationRenderResponse,
};
use super::types::{RenderSession, VisualizationState};

impl VisualizationState {
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
