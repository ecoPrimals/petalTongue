// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared interaction state for the SceneGraph-based visualization pipeline.
//!
//! `SceneInteractionState` tracks hover and selection across frames for a
//! single binding, persisted via `egui::data_mut`. `ViewCamera` provides
//! pan/zoom for spatial exploration.

use crate::scene_bridge::PixelProvenance;

/// Per-binding interaction state persisted across frames.
#[derive(Debug, Clone, Default)]
pub struct SceneInteractionState {
    /// `data_id` values of currently selected primitives.
    pub selected_ids: Vec<String>,
    /// `data_id` of the primitive currently under the cursor.
    pub hovered_id: Option<String>,
    /// Full provenance of the hovered primitive (coordinates, node, etc.).
    pub hovered_prov: Option<PixelProvenance>,
}

impl SceneInteractionState {
    /// Toggle selection of a `data_id`. Returns `true` if now selected.
    pub fn toggle_selection(&mut self, data_id: &str) -> bool {
        if let Some(pos) = self.selected_ids.iter().position(|id| id == data_id) {
            self.selected_ids.remove(pos);
            false
        } else {
            self.selected_ids.push(data_id.to_string());
            true
        }
    }

    pub fn is_selected(&self, data_id: &str) -> bool {
        self.selected_ids.iter().any(|id| id == data_id)
    }

    pub fn clear_selection(&mut self) {
        self.selected_ids.clear();
    }
}

/// 2D camera transform for scene pan/zoom.
#[derive(Debug, Clone)]
pub struct ViewCamera {
    /// Pan offset in screen pixels.
    pub offset: egui::Vec2,
    /// Zoom level (1.0 = fit-to-view).
    pub zoom: f32,
}

impl Default for ViewCamera {
    fn default() -> Self {
        Self {
            offset: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl ViewCamera {
    /// Apply zoom centered on a screen-space focal point.
    pub fn zoom_at(&mut self, focal: egui::Pos2, factor: f32) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.1, 50.0);
        let ratio = self.zoom / old_zoom;
        self.offset = focal.to_vec2() - (focal.to_vec2() - self.offset) * ratio;
    }

    /// Convert a screen-space point to scene-space, accounting for zoom + offset.
    pub fn screen_to_scene(&self, screen_pt: egui::Pos2, widget_origin: egui::Pos2) -> egui::Pos2 {
        let local = screen_pt - widget_origin.to_vec2();
        egui::pos2(
            (local.x - self.offset.x) / self.zoom,
            (local.y - self.offset.y) / self.zoom,
        )
    }

    /// Reset to default (fit-to-view).
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
