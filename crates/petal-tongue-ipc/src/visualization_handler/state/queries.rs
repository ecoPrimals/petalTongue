// SPDX-License-Identifier: AGPL-3.0-only
//! Read-only queries on visualization state.
//!
//! Session status, bindings access, and scene lookups.

use petal_tongue_core::DataBinding;

use super::super::types::{SessionStatusRequest, SessionStatusResponse};
use super::types::{RenderSession, VisualizationState};

impl VisualizationState {
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

    /// Get a compiled scene graph by session ID (for downstream rendering).
    #[must_use]
    pub fn grammar_scene(
        &self,
        session_id: &str,
    ) -> Option<&petal_tongue_scene::scene_graph::SceneGraph> {
        self.grammar_scenes.get(session_id)
    }
}
