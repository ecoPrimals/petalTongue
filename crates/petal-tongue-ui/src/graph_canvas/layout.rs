// SPDX-License-Identifier: AGPL-3.0-only
//! Graph canvas layout - coordinate conversion between world and screen space.

use egui::{Pos2, Rect};
use petal_tongue_core::graph_builder::Vec2;

/// Convert world coordinates to screen coordinates
pub(super) fn world_to_screen(
    world_pos: Vec2,
    canvas_rect: Rect,
    position: &Vec2,
    zoom: f32,
) -> Pos2 {
    let screen_x =
        canvas_rect.center().x + (world_pos.x - position.x) * zoom;
    let screen_y =
        canvas_rect.center().y + (world_pos.y - position.y) * zoom;
    Pos2::new(screen_x, screen_y)
}

/// Convert screen coordinates to world coordinates
pub(super) fn screen_to_world(
    screen_pos: Pos2,
    canvas_rect: Rect,
    position: &Vec2,
    zoom: f32,
) -> Vec2 {
    let world_x =
        position.x + (screen_pos.x - canvas_rect.center().x) / zoom;
    let world_y =
        position.y + (screen_pos.y - canvas_rect.center().y) / zoom;
    Vec2::new(world_x, world_y)
}
