// SPDX-License-Identifier: AGPL-3.0-or-later
//! Interaction engine -- the game-engine-style bidirectional loop.
//!
//! Consumes [`SensorEvent`] values from input
//! adapters and IPC, translates them to semantic intents, resolves targets
//! via inverse pipelines, applies state changes, and broadcasts results.

use std::collections::{HashMap, HashSet};

use crate::sensor::SensorEvent;

use super::adapter::{InputAdapter, InteractionContext};
use super::input_adapter_impl::InputAdapterImpl;
use super::intent::{InteractionIntent, SelectionMode};
use super::inverse::InversePipeline;
use super::inverse_pipeline_impl::InversePipelineImpl;
use super::perspective::{OutputModality, Perspective, PerspectiveId};
use super::result::{InteractionClock, InteractionEvent, InteractionResult, StateChange};
use super::target::{DataObjectId, InteractionTarget};

/// Central interaction engine managing adapters, pipelines, and perspectives.
///
/// The engine does NOT own the rendering loop -- it is called each frame
/// by whatever rendering system is active (egui, ratatui, headless).
pub struct InteractionEngine {
    adapters: Vec<InputAdapterImpl>,
    inverse_pipelines: HashMap<OutputModality, InversePipelineImpl>,
    perspectives: HashMap<PerspectiveId, Perspective>,
    clock: InteractionClock,
    pending_results: Vec<InteractionResult>,
    next_perspective_id: PerspectiveId,
}

impl InteractionEngine {
    /// Create a new empty engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
            inverse_pipelines: HashMap::new(),
            perspectives: HashMap::new(),
            clock: InteractionClock::new(),
            pending_results: Vec::new(),
            next_perspective_id: 1,
        }
    }

    /// Register an input adapter.
    pub fn register_adapter(&mut self, adapter: InputAdapterImpl) {
        tracing::debug!(
            "Registered input adapter: {} ({})",
            adapter.name(),
            adapter.modality()
        );
        self.adapters.push(adapter);
    }

    /// Register an inverse pipeline for a modality.
    pub fn register_inverse(&mut self, pipeline: InversePipelineImpl) {
        let modality = pipeline.modality();
        tracing::debug!("Registered inverse pipeline for {modality:?}");
        self.inverse_pipelines.insert(modality, pipeline);
    }

    /// Create and register a new perspective, returning its ID.
    pub fn add_perspective(&mut self, mut perspective: Perspective) -> PerspectiveId {
        let id = self.next_perspective_id;
        self.next_perspective_id += 1;
        perspective.id = id;
        self.perspectives.insert(id, perspective);
        id
    }

    /// Get an immutable reference to a perspective.
    #[must_use]
    pub fn perspective(&self, id: PerspectiveId) -> Option<&Perspective> {
        self.perspectives.get(&id)
    }

    /// Get a mutable reference to a perspective.
    pub fn perspective_mut(&mut self, id: PerspectiveId) -> Option<&mut Perspective> {
        self.perspectives.get_mut(&id)
    }

    /// List all active perspectives.
    #[must_use]
    pub fn perspectives(&self) -> Vec<&Perspective> {
        self.perspectives.values().collect()
    }

    /// Process a batch of sensor events through the interaction loop.
    ///
    /// This is the core tick: translate -> resolve -> apply -> collect results.
    /// Call this once per frame from the rendering loop.
    pub fn process_events(
        &mut self,
        events: &[SensorEvent],
        context: &InteractionContext,
    ) -> Vec<InteractionResult> {
        let mut results = Vec::new();

        for event in events {
            if let Some(result) = self.process_single_event(event, context) {
                results.push(result);
            }
        }

        // Drain any pending results from IPC or deferred processing
        results.append(&mut self.pending_results);

        // Send feedback to all adapters
        for result in &results {
            for adapter in &mut self.adapters {
                adapter.feedback(result);
            }
        }

        results
    }

    /// Process a single sensor event through the full pipeline.
    fn process_single_event(
        &mut self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionResult> {
        // Skip non-interaction events (heartbeats, display confirmations)
        if !event.is_user_interaction() && !matches!(event, SensorEvent::Position { .. }) {
            return None;
        }

        // Step 1: Translate -- try each adapter until one produces an intent
        let intent = self.translate_event(event, context)?;

        // Step 2: Resolve -- use inverse pipelines to map targets to data space
        let resolved_targets = self.resolve_targets(&intent, event, context);

        // Step 3: Apply -- update perspective state (clone once for result, consume in apply)
        let resolved_for_result = resolved_targets.clone();
        let state_changes = self.apply_intent(&intent, resolved_targets, context.perspective_id);

        Some(InteractionResult {
            intent,
            resolved_targets: resolved_for_result,
            state_changes,
            perspective_id: context.perspective_id,
            timestamp_ms: self.clock.elapsed_ms(),
        })
    }

    /// Try each adapter to translate a sensor event into an intent.
    fn translate_event(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        for adapter in &self.adapters {
            if let Some(intent) = adapter.translate(event, context) {
                return Some(intent);
            }
        }
        None
    }

    /// Resolve interaction targets to data-space objects.
    fn resolve_targets(
        &self,
        intent: &InteractionIntent,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Vec<DataObjectId> {
        let target = match intent {
            InteractionIntent::Select { target, .. }
            | InteractionIntent::Inspect { target, .. }
            | InteractionIntent::Manipulate { target, .. }
            | InteractionIntent::Annotate { target, .. }
            | InteractionIntent::Focus { target } => Some(target),
            InteractionIntent::Navigate { .. }
            | InteractionIntent::Command { .. }
            | InteractionIntent::Dismiss => None,
        };

        if let Some(target) = target {
            // If target is already a DataRow, extract it directly
            if let InteractionTarget::DataRow { data_id } = target {
                return vec![data_id.clone()];
            }

            // Otherwise, try each inverse pipeline
            for pipeline in self.inverse_pipelines.values() {
                if let Some(data_id) = InversePipeline::resolve_to_data_id(pipeline, target) {
                    return vec![data_id];
                }
                // Also try resolving from the raw event
                if let Some(resolved) = InversePipeline::resolve(pipeline, event, context)
                    && let Some(data_id) = InversePipeline::resolve_to_data_id(pipeline, &resolved)
                {
                    return vec![data_id];
                }
            }
        }

        Vec::new()
    }

    /// Apply an intent to perspective state and produce state changes.
    fn apply_intent(
        &mut self,
        intent: &InteractionIntent,
        resolved_targets: Vec<DataObjectId>,
        perspective_id: PerspectiveId,
    ) -> Vec<StateChange> {
        let mut changes = Vec::new();

        match intent {
            InteractionIntent::Select { mode, .. } => {
                if let Some(perspective) = self.perspectives.get_mut(&perspective_id) {
                    match mode {
                        SelectionMode::Replace => {
                            perspective.selection = resolved_targets;
                        }
                        SelectionMode::Add => {
                            for target in resolved_targets {
                                perspective.add_to_selection(target);
                            }
                        }
                        SelectionMode::Remove => {
                            let resolved_set: HashSet<_> = resolved_targets.iter().collect();
                            perspective.selection.retain(|s| !resolved_set.contains(s));
                        }
                        SelectionMode::Toggle => {
                            for target in resolved_targets {
                                perspective.toggle_selection(target);
                            }
                        }
                    }

                    changes.push(StateChange::SelectionChanged {
                        selected: perspective.selection.clone(),
                    });

                    // Propagate to synchronized perspectives
                    self.propagate_selection(perspective_id, &mut changes);
                }
            }

            InteractionIntent::Focus { .. } => {
                if let Some(perspective) = self.perspectives.get_mut(&perspective_id) {
                    perspective.focus = resolved_targets.into_iter().next();
                    changes.push(StateChange::FocusChanged {
                        focused: perspective.focus.clone(),
                    });
                }
            }

            InteractionIntent::Navigate {
                direction,
                magnitude,
            } => {
                if let Some(perspective) = self.perspectives.get_mut(&perspective_id) {
                    apply_navigation(&mut perspective.viewport, direction, *magnitude);
                    changes.push(StateChange::ViewChanged {
                        center_x: perspective.viewport.center_x,
                        center_y: perspective.viewport.center_y,
                        zoom: perspective.viewport.zoom,
                    });
                }
            }

            InteractionIntent::Dismiss => {
                if let Some(perspective) = self.perspectives.get_mut(&perspective_id) {
                    perspective.clear_selection();
                    perspective.focus = None;
                    changes.push(StateChange::SelectionChanged {
                        selected: Vec::new(),
                    });
                    changes.push(StateChange::FocusChanged { focused: None });
                }
            }

            _ => {}
        }

        changes
    }

    /// Propagate selection changes to perspectives with `SharedSelection` or `FullSync`.
    fn propagate_selection(&mut self, source_id: PerspectiveId, _changes: &mut Vec<StateChange>) {
        let source_selection = self
            .perspectives
            .get(&source_id)
            .map(|p| (p.selection.clone(), p.sync_mode))
            .unwrap_or_default();

        let (selection, sync_mode) = source_selection;

        if matches!(
            sync_mode,
            super::perspective::PerspectiveSync::SharedSelection
                | super::perspective::PerspectiveSync::FullSync
        ) {
            for (id, perspective) in &mut self.perspectives {
                if *id == source_id {
                    continue;
                }
                if matches!(
                    perspective.sync_mode,
                    super::perspective::PerspectiveSync::SharedSelection
                        | super::perspective::PerspectiveSync::FullSync
                ) {
                    perspective.selection.clone_from(&selection);
                    // Note: we don't add extra StateChange entries for
                    // synced perspectives -- the broadcasting layer handles
                    // that via InteractionEvent over IPC or EventBus.
                }
            }
        }
    }

    /// Inject an interaction result from IPC (another primal or remote user).
    pub fn inject_ipc_event(&mut self, event: InteractionEvent) {
        // Apply the remote event as if it were local
        for perspective in self.perspectives.values_mut() {
            if matches!(
                perspective.sync_mode,
                super::perspective::PerspectiveSync::SharedSelection
                    | super::perspective::PerspectiveSync::FullSync
            ) {
                if event.event_type == "select" {
                    perspective.selection.clone_from(&event.targets);
                } else if event.event_type == "focus" {
                    perspective.focus = event.targets.first().cloned();
                }
            }
        }

        self.pending_results.push(InteractionResult {
            intent: InteractionIntent::Select {
                target: InteractionTarget::Nothing,
                mode: SelectionMode::Replace,
            },
            resolved_targets: event.targets,
            state_changes: vec![],
            perspective_id: event.perspective_id,
            timestamp_ms: self.clock.elapsed_ms(),
        });
    }

    /// Get count of registered adapters.
    #[must_use]
    pub const fn adapter_count(&self) -> usize {
        self.adapters.len()
    }

    /// Get count of registered inverse pipelines.
    #[must_use]
    pub fn inverse_pipeline_count(&self) -> usize {
        self.inverse_pipelines.len()
    }
}

impl Default for InteractionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply a navigation direction to a viewport.
fn apply_navigation(
    viewport: &mut super::perspective::PerspectiveViewport,
    direction: &super::intent::NavigationDirection,
    magnitude: f64,
) {
    use super::intent::NavigationDirection;
    let step = magnitude * 10.0 / viewport.zoom;

    match direction {
        NavigationDirection::Left => viewport.center_x -= step,
        NavigationDirection::Right => viewport.center_x += step,
        NavigationDirection::Up => viewport.center_y += step,
        NavigationDirection::Down => viewport.center_y -= step,
        NavigationDirection::In => {
            viewport.zoom *= magnitude.mul_add(0.1, 1.0);
            viewport.zoom = viewport.zoom.clamp(0.1, 100.0);
        }
        NavigationDirection::Out => {
            viewport.zoom /= magnitude.mul_add(0.1, 1.0);
            viewport.zoom = viewport.zoom.clamp(0.1, 100.0);
        }
        NavigationDirection::Forward
        | NavigationDirection::Backward
        | NavigationDirection::ToData { .. } => {
            // Forward/backward ignored in 2D; data-targeted nav handled by caller.
        }
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
