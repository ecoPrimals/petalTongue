// SPDX-License-Identifier: AGPL-3.0-only
//! Bridge between the declarative scene engine (`petal-tongue-scene`) and egui rendering.
//!
//! Translates scene graph primitives into egui paint commands, connecting the
//! Grammar of Graphics pipeline to the live UI. Builds a `FrameHitMap` alongside
//! the paint commands so every rendered region can be traced back to its source
//! primitive, node, and data object.

mod paint;
mod types;

use egui::{Sense, Ui, Vec2};
use petal_tongue_scene::render_plan::RenderPlan;

#[cfg(test)]
mod tests;

pub use paint::{
    paint_plan, paint_plan_tracked, paint_primitive, paint_scene, paint_scene_tracked,
};
pub use types::{FrameHitMap, PixelProvenance};

/// An egui widget that renders a `RenderPlan` via the scene bridge.
///
/// Drop this into any egui panel to display a grammar-compiled visualization:
/// ```ignore
/// SceneWidget::new(&plan).desired_size(vec2(400.0, 300.0)).show(ui);
/// ```
pub struct SceneWidget<'a> {
    plan: &'a RenderPlan,
    desired_size: Vec2,
    hit_map: Option<&'a mut FrameHitMap>,
}

impl<'a> SceneWidget<'a> {
    pub const fn new(plan: &'a RenderPlan) -> Self {
        Self {
            plan,
            desired_size: Vec2::new(400.0, 300.0),
            hit_map: None,
        }
    }

    #[must_use]
    pub const fn desired_size(mut self, size: Vec2) -> Self {
        self.desired_size = size;
        self
    }

    /// If provided, the widget populates this hit map during rendering.
    #[must_use]
    pub const fn with_hit_map(mut self, hit_map: &'a mut FrameHitMap) -> Self {
        self.hit_map = Some(hit_map);
        self
    }

    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(self.desired_size, Sense::hover());
        let offset = response.rect.min.to_vec2();
        if let Some(hit_map) = self.hit_map {
            *hit_map = paint_plan_tracked(&painter, self.plan, offset);
        } else {
            paint_plan(&painter, self.plan, offset);
        }
        response
    }
}
