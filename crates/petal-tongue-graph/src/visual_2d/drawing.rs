// SPDX-License-Identifier: AGPL-3.0-only
//! Edge and arrow drawing for 2D graph visualization.

use egui::{Color32, Pos2, Stroke, Vec2};

/// Draw an arrow head at the target end of an edge.
pub fn draw_arrow_head(painter: &egui::Painter, from: Pos2, to: Pos2, zoom: f32) {
    let arrow_size = 10.0 * zoom;
    let direction = (to - from).normalized();
    let perpendicular = Vec2::new(-direction.y, direction.x);

    let arrow_tip = to - direction * 20.0 * zoom;
    let arrow_left = arrow_tip - direction * arrow_size + perpendicular * arrow_size * 0.5;
    let arrow_right = arrow_tip - direction * arrow_size - perpendicular * arrow_size * 0.5;

    painter.add(egui::Shape::convex_polygon(
        vec![arrow_tip, arrow_left, arrow_right],
        Color32::GRAY,
        Stroke::NONE,
    ));
}
