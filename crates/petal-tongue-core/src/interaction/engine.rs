// SPDX-License-Identifier: AGPL-3.0-only
//! Interaction engine -- the game-engine-style bidirectional loop.
//!
//! Consumes [`SensorEvent`] values from input
//! adapters and IPC, translates them to semantic intents, resolves targets
//! via inverse pipelines, applies state changes, and broadcasts results.

use std::collections::HashMap;

use crate::sensor::SensorEvent;

use super::adapter::{InputAdapter, InteractionContext};
use super::intent::{InteractionIntent, SelectionMode};
use super::inverse::InversePipeline;
use super::perspective::{OutputModality, Perspective, PerspectiveId};
use super::result::{InteractionClock, InteractionEvent, InteractionResult, StateChange};
use super::target::{DataObjectId, InteractionTarget};

/// Central interaction engine managing adapters, pipelines, and perspectives.
///
/// The engine does NOT own the rendering loop -- it is called each frame
/// by whatever rendering system is active (egui, ratatui, headless).
pub struct InteractionEngine {
    adapters: Vec<Box<dyn InputAdapter>>,
    inverse_pipelines: HashMap<OutputModality, Box<dyn InversePipeline>>,
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
    pub fn register_adapter(&mut self, adapter: Box<dyn InputAdapter>) {
        tracing::debug!(
            "Registered input adapter: {} ({})",
            adapter.name(),
            adapter.modality()
        );
        self.adapters.push(adapter);
    }

    /// Register an inverse pipeline for a modality.
    pub fn register_inverse(&mut self, pipeline: Box<dyn InversePipeline>) {
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

        // Step 3: Apply -- update perspective state
        let state_changes = self.apply_intent(&intent, &resolved_targets, context.perspective_id);

        Some(InteractionResult {
            intent,
            resolved_targets,
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
                if let Some(data_id) = pipeline.resolve_to_data_id(target) {
                    return vec![data_id];
                }
                // Also try resolving from the raw event
                if let Some(resolved) = pipeline.resolve(event, context)
                    && let Some(data_id) = pipeline.resolve_to_data_id(&resolved)
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
        resolved_targets: &[DataObjectId],
        perspective_id: PerspectiveId,
    ) -> Vec<StateChange> {
        let mut changes = Vec::new();

        match intent {
            InteractionIntent::Select { mode, .. } => {
                if let Some(perspective) = self.perspectives.get_mut(&perspective_id) {
                    match mode {
                        SelectionMode::Replace => {
                            perspective.selection = resolved_targets.to_vec();
                        }
                        SelectionMode::Add => {
                            for target in resolved_targets {
                                perspective.add_to_selection(target.clone());
                            }
                        }
                        SelectionMode::Remove => {
                            perspective
                                .selection
                                .retain(|s| !resolved_targets.contains(s));
                        }
                        SelectionMode::Toggle => {
                            for target in resolved_targets {
                                perspective.toggle_selection(target.clone());
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
                    perspective.focus = resolved_targets.first().cloned();
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

    /// Propagate selection changes to perspectives with SharedSelection or FullSync.
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
    pub fn adapter_count(&self) -> usize {
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
            viewport.zoom *= 1.0 + magnitude * 0.1;
            viewport.zoom = viewport.zoom.clamp(0.1, 100.0);
        }
        NavigationDirection::Out => {
            viewport.zoom /= 1.0 + magnitude * 0.1;
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
mod tests {
    use super::*;
    use crate::interaction::perspective::Perspective;

    #[test]
    fn engine_lifecycle() {
        let mut engine = InteractionEngine::new();
        assert_eq!(engine.adapter_count(), 0);
        assert_eq!(engine.inverse_pipeline_count(), 0);

        let id = engine.add_perspective(Perspective::new(0));
        assert!(engine.perspective(id).is_some());
        assert_eq!(engine.perspectives().len(), 1);
    }

    #[test]
    fn process_empty_events() {
        let mut engine = InteractionEngine::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let results = engine.process_events(&[], &ctx);
        assert!(results.is_empty());
    }

    #[test]
    fn selection_propagation() {
        use crate::interaction::perspective::PerspectiveSync;

        let mut engine = InteractionEngine::new();

        let mut p1 = Perspective::new(0);
        p1.sync_mode = PerspectiveSync::SharedSelection;
        let id1 = engine.add_perspective(p1);

        let mut p2 = Perspective::new(0);
        p2.sync_mode = PerspectiveSync::SharedSelection;
        let id2 = engine.add_perspective(p2);

        // Manually select in p1
        let obj = DataObjectId::new("test", serde_json::json!("row1"));
        engine.perspective_mut(id1).unwrap().select(obj.clone());

        // Propagate
        let mut changes = Vec::new();
        engine.propagate_selection(id1, &mut changes);

        // p2 should now have the same selection
        assert!(engine.perspective(id2).unwrap().is_selected(&obj));
    }

    #[test]
    fn navigation_zoom() {
        use super::super::intent::NavigationDirection;
        use super::super::perspective::PerspectiveViewport;

        let mut vp = PerspectiveViewport::default();
        let initial_zoom = vp.zoom;

        apply_navigation(&mut vp, &NavigationDirection::In, 1.0);
        assert!(vp.zoom > initial_zoom);

        apply_navigation(&mut vp, &NavigationDirection::Out, 1.0);
        // Should be approximately back to initial (not exact due to asymmetric scaling)
        assert!(vp.zoom < initial_zoom * 1.2);
    }

    #[test]
    fn ipc_event_injection() {
        let mut engine = InteractionEngine::new();
        let mut p = Perspective::new(0);
        p.sync_mode = super::super::perspective::PerspectiveSync::SharedSelection;
        let id = engine.add_perspective(p);

        let event = InteractionEvent {
            event_type: "select".into(),
            targets: vec![DataObjectId::new("remote", serde_json::json!("obj1"))],
            perspective_id: 99,
            grammar_id: "test".into(),
            timestamp: "2026-03-09T00:00:00Z".into(),
        };

        engine.inject_ipc_event(event);

        // Process should drain the pending result
        let ctx = InteractionContext::default_for_perspective(id);
        let results = engine.process_events(&[], &ctx);
        assert_eq!(results.len(), 1);

        // The local perspective should have the remote selection
        assert!(
            engine
                .perspective(id)
                .unwrap()
                .is_selected(&DataObjectId::new("remote", serde_json::json!("obj1")))
        );
    }
}
