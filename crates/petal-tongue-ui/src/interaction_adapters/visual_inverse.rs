// SPDX-License-Identifier: AGPL-3.0-only
//! Visual inverse pipeline for egui-based rendering.
//!
//! Maps pixel coordinates back to data-space targets by inverting the
//! camera transform (zoom, pan, offset). This replaces the direct
//! `screen_to_world` calls scattered through `graph_canvas` and `visual_2d`
//! with a unified, trait-based approach.

use petal_tongue_core::interaction::{
    BoundingBox, DataObjectId, DataRow, InteractionContext, InteractionTarget, InversePipeline,
    OutputModality, PrimitiveId,
};
use petal_tongue_core::sensor::SensorEvent;
use petal_tongue_scene::render_plan::RenderPlan;

/// Inverse pipeline for egui/GUI visual rendering.
///
/// Resolves pixel-space sensor events to data-space targets by applying
/// the inverse of the camera transform (center, zoom) used during rendering.
pub struct VisualInversePipeline {
    /// Nodes registered for hit-testing, with their world-space positions
    /// and associated data IDs. Populated each frame by the renderer.
    hit_targets: Vec<HitTarget>,
    /// Half-width of a node in world-space units (for hit-test radius).
    node_half_width: f32,
    /// Half-height of a node in world-space units.
    node_half_height: f32,
}

/// A renderable object registered for hit-testing.
#[derive(Debug, Clone)]
struct HitTarget {
    /// World-space X position.
    world_x: f32,
    /// World-space Y position.
    world_y: f32,
    /// Data identity for this object.
    data_id: DataObjectId,
    /// Primitive index within the current frame.
    primitive_id: PrimitiveId,
}

impl VisualInversePipeline {
    /// Create a new visual inverse pipeline.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hit_targets: Vec::new(),
            node_half_width: 30.0,
            node_half_height: 20.0,
        }
    }

    /// Set the node dimensions used for hit-testing.
    pub const fn set_node_size(&mut self, half_width: f32, half_height: f32) {
        self.node_half_width = half_width;
        self.node_half_height = half_height;
    }

    /// Clear all hit targets (call at the start of each frame).
    pub fn clear_targets(&mut self) {
        self.hit_targets.clear();
    }

    /// Register a hit target for the current frame.
    ///
    /// Call this during rendering for each object that should be
    /// interactable. The renderer knows the world positions; the
    /// inverse pipeline uses them for hit-testing.
    pub fn register_target(
        &mut self,
        world_x: f32,
        world_y: f32,
        data_id: DataObjectId,
        primitive_id: PrimitiveId,
    ) {
        self.hit_targets.push(HitTarget {
            world_x,
            world_y,
            data_id,
            primitive_id,
        });
    }

    /// Convert screen coordinates to world coordinates.
    ///
    /// Inverts the camera transform: `world = camera_pos + (screen - center) / zoom`
    fn screen_to_world(
        &self,
        screen_x: f32,
        screen_y: f32,
        context: &InteractionContext,
    ) -> (f32, f32) {
        let center_x = context.screen_width / 2.0;
        let center_y = context.screen_height / 2.0;
        let zoom = context.zoom as f32;

        let world_x = context.viewport_center_x as f32 + (screen_x - center_x) / zoom;
        let world_y = context.viewport_center_y as f32 + (screen_y - center_y) / zoom;
        (world_x, world_y)
    }

    /// Find the nearest hit target to a world-space position.
    fn hit_test(&self, world_x: f32, world_y: f32, zoom: f32) -> Option<&HitTarget> {
        let half_w = self.node_half_width / zoom;
        let half_h = self.node_half_height / zoom;

        self.hit_targets
            .iter()
            .find(|t| (t.world_x - world_x).abs() < half_w && (t.world_y - world_y).abs() < half_h)
    }
}

impl VisualInversePipeline {
    /// Resolve a screen position to data-space coordinates using a `RenderPlan`.
    ///
    /// 1. Screen → world via camera inverse
    /// 2. World → panel hit test (which panel contains this point?)
    /// 3. Panel pixel → `AxisMeta::inverse()` → data value per axis
    ///
    /// Returns `None` if the position falls outside all panels.
    #[must_use]
    pub fn resolve_to_data_coords(
        &self,
        screen_x: f32,
        screen_y: f32,
        context: &InteractionContext,
        plan: &RenderPlan,
    ) -> Option<DataRow> {
        let (world_x, world_y) = self.screen_to_world(screen_x, screen_y, context);

        for panel in &plan.panels {
            if !panel
                .bounds
                .contains(f64::from(world_x), f64::from(world_y))
            {
                continue;
            }

            let mut row = DataRow::new();
            for axis in &panel.axes {
                let visual_val = match axis.variable.as_str() {
                    "x" => f64::from(world_x),
                    "y" => f64::from(world_y),
                    _ => continue,
                };
                let data_val = axis.inverse(visual_val);
                row.insert(axis.variable.clone(), serde_json::Value::from(data_val));
            }
            return Some(row);
        }
        None
    }
}

impl Default for VisualInversePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl InversePipeline for VisualInversePipeline {
    fn modality(&self) -> OutputModality {
        OutputModality::Gui
    }

    fn resolve(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionTarget> {
        let (screen_x, screen_y) = match event {
            SensorEvent::Click { x, y, .. } | SensorEvent::Position { x, y, .. } => (*x, *y),
            _ => return None,
        };

        let (world_x, world_y) = self.screen_to_world(screen_x, screen_y, context);
        let zoom = context.zoom as f32;

        self.hit_test(world_x, world_y, zoom).map_or_else(
            || {
                Some(InteractionTarget::Region {
                    bounds: BoundingBox::from_corners(
                        f64::from(world_x),
                        f64::from(world_y),
                        f64::from(world_x),
                        f64::from(world_y),
                    ),
                })
            },
            |target| {
                Some(InteractionTarget::DataRow {
                    data_id: target.data_id.clone(),
                })
            },
        )
    }

    fn nearest_primitive(&self, target: &InteractionTarget) -> Option<PrimitiveId> {
        match target {
            InteractionTarget::DataRow { data_id } => self
                .hit_targets
                .iter()
                .find(|t| t.data_id == *data_id)
                .map(|t| t.primitive_id),
            _ => None,
        }
    }

    fn data_at(&self, target: &InteractionTarget) -> Option<DataRow> {
        match target {
            InteractionTarget::DataRow { data_id } => {
                let mut row = DataRow::new();
                row.insert(
                    "source".into(),
                    serde_json::Value::String(data_id.source.clone()),
                );
                row.insert("row_key".into(), data_id.row_key.clone());
                Some(row)
            }
            _ => None,
        }
    }

    fn resolve_to_data_id(&self, target: &InteractionTarget) -> Option<DataObjectId> {
        match target {
            InteractionTarget::DataRow { data_id } => Some(data_id.clone()),
            InteractionTarget::Primitive { primitive_id } => self
                .hit_targets
                .iter()
                .find(|t| t.primitive_id == *primitive_id)
                .map(|t| t.data_id.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn make_pipeline_with_targets() -> VisualInversePipeline {
        let mut pipeline = VisualInversePipeline::new();
        pipeline.set_node_size(30.0, 20.0);

        pipeline.register_target(
            0.0,
            0.0,
            DataObjectId::new("health", serde_json::json!({"id": "alpha"})),
            1,
        );
        pipeline.register_target(
            100.0,
            100.0,
            DataObjectId::new("health", serde_json::json!({"id": "beta"})),
            2,
        );

        pipeline
    }

    fn make_centered_context() -> InteractionContext {
        let mut ctx = InteractionContext::default_for_perspective(1);
        ctx.screen_width = 800.0;
        ctx.screen_height = 600.0;
        ctx.viewport_center_x = 0.0;
        ctx.viewport_center_y = 0.0;
        ctx.zoom = 1.0;
        ctx
    }

    #[test]
    fn click_on_target_resolves_to_data() {
        let pipeline = make_pipeline_with_targets();
        let ctx = make_centered_context();

        let event = SensorEvent::Click {
            x: 400.0,
            y: 300.0,
            button: petal_tongue_core::sensor::MouseButton::Left,
            timestamp: Instant::now(),
        };

        let target = pipeline.resolve(&event, &ctx);
        assert!(target.is_some());

        if let Some(InteractionTarget::DataRow { data_id }) = target {
            assert_eq!(data_id.source, "health");
        }
    }

    #[test]
    fn click_on_empty_resolves_to_region() {
        let pipeline = make_pipeline_with_targets();
        let ctx = make_centered_context();

        let event = SensorEvent::Click {
            x: 10.0,
            y: 10.0,
            button: petal_tongue_core::sensor::MouseButton::Left,
            timestamp: Instant::now(),
        };

        let target = pipeline.resolve(&event, &ctx);
        assert!(matches!(target, Some(InteractionTarget::Region { .. })));
    }

    #[test]
    fn nearest_primitive_lookup() {
        let pipeline = make_pipeline_with_targets();

        let data_id = DataObjectId::new("health", serde_json::json!({"id": "beta"}));
        let target = InteractionTarget::DataRow { data_id };

        let prim = pipeline.nearest_primitive(&target);
        assert_eq!(prim, Some(2));
    }

    #[test]
    fn resolve_to_data_id_from_primitive() {
        let pipeline = make_pipeline_with_targets();

        let target = InteractionTarget::Primitive { primitive_id: 1 };
        let data_id = pipeline.resolve_to_data_id(&target);

        assert!(data_id.is_some());
        assert_eq!(data_id.unwrap().source, "health");
    }

    #[test]
    fn clear_targets_resets_state() {
        let mut pipeline = make_pipeline_with_targets();
        assert!(!pipeline.hit_targets.is_empty());

        pipeline.clear_targets();
        assert!(pipeline.hit_targets.is_empty());
    }

    #[test]
    fn heartbeat_returns_none() {
        let pipeline = make_pipeline_with_targets();
        let ctx = make_centered_context();

        let event = SensorEvent::Heartbeat {
            latency: std::time::Duration::from_millis(10),
            timestamp: Instant::now(),
        };

        assert!(pipeline.resolve(&event, &ctx).is_none());
    }
}
