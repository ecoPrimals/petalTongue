// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bridge between egui events and the `InteractionEngine`.
//!
//! Converts egui's `Response` events into `SensorEvent` values and
//! feeds them through the interaction engine pipeline. This replaces
//! scattered direct egui event handling with the unified, modality-agnostic
//! interaction system.
//!
//! ## Usage
//!
//! ```ignore
//! let bridge = EguiInteractionBridge::new();
//! // Each frame:
//! let events = bridge.collect_events(ui, &response);
//! let results = bridge.engine.process_events(&events, &context);
//! ```

use std::time::Instant;

use petal_tongue_core::interaction::{
    InputAdapterImpl, InteractionContext, InteractionEngine, InteractionResult,
    InversePipelineImpl, Perspective, PerspectiveId,
};
use petal_tongue_core::sensor::{MouseButton, SensorEvent};

use crate::interaction_adapters::{KeyboardAdapter, PointerAdapter, VisualInversePipeline};

/// Bridges egui events to the `InteractionEngine`.
pub struct EguiInteractionBridge {
    pub engine: InteractionEngine,
    pub perspective_id: PerspectiveId,
    inverse_pipeline: VisualInversePipeline,
}

impl EguiInteractionBridge {
    /// Create a new bridge with default pointer and keyboard adapters.
    #[must_use]
    pub fn new() -> Self {
        let mut engine = InteractionEngine::new();

        engine.register_adapter(InputAdapterImpl::Pointer(PointerAdapter::new()));
        engine.register_adapter(InputAdapterImpl::Keyboard(KeyboardAdapter::new()));

        let inverse = VisualInversePipeline::new();
        engine.register_inverse(InversePipelineImpl::Visual(VisualInversePipeline::new()));

        let perspective = Perspective::new(0);
        let perspective_id = engine.add_perspective(perspective);

        Self {
            engine,
            perspective_id,
            inverse_pipeline: inverse,
        }
    }

    /// Access the inverse pipeline for hit-target registration.
    pub const fn inverse_pipeline_mut(&mut self) -> &mut VisualInversePipeline {
        &mut self.inverse_pipeline
    }

    /// Collect `SensorEvent`s from an egui `Response`.
    #[must_use]
    pub fn collect_events(&self, response: &egui::Response) -> Vec<SensorEvent> {
        let mut events = Vec::new();
        let now = Instant::now();

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            events.push(SensorEvent::Click {
                x: pos.x,
                y: pos.y,
                button: MouseButton::Left,
                timestamp: now,
            });
        }

        if response.secondary_clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            events.push(SensorEvent::Click {
                x: pos.x,
                y: pos.y,
                button: MouseButton::Right,
                timestamp: now,
            });
        }

        if response.hovered()
            && let Some(pos) = response.hover_pos()
        {
            events.push(SensorEvent::Position {
                x: pos.x,
                y: pos.y,
                timestamp: now,
            });
        }

        let scroll = response.ctx.input(|i| i.smooth_scroll_delta);
        if scroll.y.abs() > 0.1 {
            events.push(SensorEvent::Scroll {
                delta_x: scroll.x,
                delta_y: scroll.y,
                timestamp: now,
            });
        }

        events
    }

    /// Build the `InteractionContext` for the current frame.
    #[must_use]
    pub const fn build_context(
        &self,
        screen_width: f32,
        screen_height: f32,
        camera_x: f64,
        camera_y: f64,
        zoom: f64,
    ) -> InteractionContext {
        let mut ctx = InteractionContext::default_for_perspective(self.perspective_id);
        ctx.screen_width = screen_width;
        ctx.screen_height = screen_height;
        ctx.viewport_center_x = camera_x;
        ctx.viewport_center_y = camera_y;
        ctx.zoom = zoom;
        ctx
    }

    /// Run the full interaction loop for this frame.
    pub fn process_frame(
        &mut self,
        response: &egui::Response,
        screen_width: f32,
        screen_height: f32,
        camera_x: f64,
        camera_y: f64,
        zoom: f64,
    ) -> Vec<InteractionResult> {
        let events = self.collect_events(response);
        let context = self.build_context(screen_width, screen_height, camera_x, camera_y, zoom);
        self.engine.process_events(&events, &context)
    }

    /// Get the currently selected data object IDs.
    #[must_use]
    pub fn selected_data_ids(&self) -> Vec<petal_tongue_core::interaction::DataObjectId> {
        self.engine
            .perspective(self.perspective_id)
            .map(|p| p.selection.clone())
            .unwrap_or_default()
    }

    /// Get the currently focused data object ID.
    #[must_use]
    pub fn focused_data_id(&self) -> Option<petal_tongue_core::interaction::DataObjectId> {
        self.engine
            .perspective(self.perspective_id)
            .and_then(|p| p.focus.clone())
    }
}

impl Default for EguiInteractionBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use petal_tongue_core::interaction::DataObjectId;

    use super::*;

    #[test]
    fn bridge_creation() {
        let bridge = EguiInteractionBridge::new();
        assert_eq!(bridge.engine.adapter_count(), 2);
        assert_eq!(bridge.engine.inverse_pipeline_count(), 1);
        assert!(bridge.engine.perspective(bridge.perspective_id).is_some());
    }

    #[test]
    fn bridge_default() {
        let bridge = EguiInteractionBridge::default();
        assert_eq!(bridge.engine.adapter_count(), 2);
    }

    #[test]
    fn build_context_sets_fields() {
        let bridge = EguiInteractionBridge::new();
        let ctx = bridge.build_context(800.0, 600.0, 100.0, 200.0, 2.0);
        assert!((ctx.screen_width - 800.0).abs() < f32::EPSILON);
        assert!((ctx.screen_height - 600.0).abs() < f32::EPSILON);
        assert!((ctx.viewport_center_x - 100.0).abs() < f64::EPSILON);
        assert!((ctx.viewport_center_y - 200.0).abs() < f64::EPSILON);
        assert!((ctx.zoom - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn build_context_various_window_sizes() {
        let bridge = EguiInteractionBridge::new();

        let ctx = bridge.build_context(1920.0, 1080.0, 0.0, 0.0, 1.0);
        assert!((ctx.screen_width - 1920.0).abs() < f32::EPSILON);
        assert!((ctx.screen_height - 1080.0).abs() < f32::EPSILON);
        assert!((ctx.zoom - 1.0).abs() < f64::EPSILON);

        let ctx = bridge.build_context(320.0, 240.0, -50.5, 100.25, 0.5);
        assert!((ctx.screen_width - 320.0).abs() < f32::EPSILON);
        assert!((ctx.screen_height - 240.0).abs() < f32::EPSILON);
        assert!((ctx.viewport_center_x - (-50.5)).abs() < f64::EPSILON);
        assert!((ctx.viewport_center_y - 100.25).abs() < f64::EPSILON);
        assert!((ctx.zoom - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn build_context_zero_dimensions() {
        let bridge = EguiInteractionBridge::new();
        let ctx = bridge.build_context(0.0, 0.0, 0.0, 0.0, 1.0);
        assert_eq!(ctx.screen_width, 0.0);
        assert_eq!(ctx.screen_height, 0.0);
    }

    #[test]
    fn selected_data_ids_empty_initially() {
        let bridge = EguiInteractionBridge::new();
        assert!(bridge.selected_data_ids().is_empty());
    }

    #[test]
    fn selected_data_ids_with_selection() {
        let mut bridge = EguiInteractionBridge::new();
        let id_a = DataObjectId::new("test", serde_json::json!("a"));
        let id_b = DataObjectId::new("test", serde_json::json!("b"));

        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.select(id_a.clone());
        }
        let ids = bridge.selected_data_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], id_a);

        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.add_to_selection(id_b.clone());
        }
        let ids = bridge.selected_data_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id_a));
        assert!(ids.contains(&id_b));
    }

    #[test]
    fn selected_data_ids_after_clear() {
        let mut bridge = EguiInteractionBridge::new();
        let id = DataObjectId::new("test", serde_json::json!("x"));
        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.select(id);
        }
        assert_eq!(bridge.selected_data_ids().len(), 1);

        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.clear_selection();
        }
        assert!(bridge.selected_data_ids().is_empty());
    }

    #[test]
    fn focused_data_id_none_initially() {
        let bridge = EguiInteractionBridge::new();
        assert!(bridge.focused_data_id().is_none());
    }

    #[test]
    fn focused_data_id_with_focus() {
        let mut bridge = EguiInteractionBridge::new();
        let id = DataObjectId::new("metrics", serde_json::json!("node-1"));

        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.focus = Some(id.clone());
        }
        assert_eq!(bridge.focused_data_id(), Some(id));
    }

    #[test]
    fn focused_data_id_cleared() {
        let mut bridge = EguiInteractionBridge::new();
        let id = DataObjectId::new("test", serde_json::json!("f"));
        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.focus = Some(id);
        }
        assert!(bridge.focused_data_id().is_some());

        if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
            p.focus = None;
        }
        assert!(bridge.focused_data_id().is_none());
    }

    #[test]
    fn inverse_pipeline_mut_accessible() {
        let mut bridge = EguiInteractionBridge::new();
        let _ = bridge.inverse_pipeline_mut();
    }
}
