// SPDX-License-Identifier: AGPL-3.0-or-later
//! Stream update handler with backpressure.
//!
//! Enforces server-side rate limiting so springs can throttle when
//! a session exceeds the configured update rate.

use super::super::stream::{apply_operation, binding_id};
use super::super::types::{StreamUpdateRequest, StreamUpdateResponse};
use super::types::{CompiledBinding, VisualizationState};

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::data_binding::DataBindingCompiler;

impl VisualizationState {
    /// Recompile a single binding into `grammar_scenes` so the SceneViewer
    /// reflects the latest stream update.
    fn recompile_binding(
        grammar_scenes: &mut std::collections::HashMap<String, CompiledBinding>,
        session_id: &str,
        bid: &str,
        binding: &petal_tongue_core::DataBinding,
        domain: Option<&str>,
    ) {
        let (grammar_expr, data) = DataBindingCompiler::compile(binding, domain);
        let compiler = GrammarCompiler::new();
        let scene = compiler.compile(&grammar_expr, &data);
        let key = format!("{session_id}:{bid}");
        let prev_scene = grammar_scenes.get(&key).map(|cb| cb.scene.clone());
        grammar_scenes.insert(
            key,
            CompiledBinding {
                scene,
                grammar: grammar_expr,
                prev_scene,
                source_binding: Some(binding.clone()),
            },
        );
    }

    /// Handle a visualization.render.stream incremental update.
    ///
    /// Enforces server-side backpressure: when a session exceeds the configured
    /// update rate, `backpressure_active` is set in the response so springs can
    /// throttle. Updates are still accepted during backpressure to avoid data loss.
    ///
    /// After each accepted update, the affected binding is recompiled into
    /// `grammar_scenes` so the SceneViewer displays the latest data.
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
                        Self::recompile_binding(
                            &mut self.grammar_scenes,
                            &req.session_id,
                            &req.binding_id,
                            binding,
                            session.domain.as_deref(),
                        );
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
            #[expect(
                clippy::cast_possible_truncation,
                reason = "rate count bounded by u32 for backpressure"
            )]
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
                Self::recompile_binding(
                    &mut self.grammar_scenes,
                    &req.session_id,
                    &req.binding_id,
                    binding,
                    session.domain.as_deref(),
                );
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
}
