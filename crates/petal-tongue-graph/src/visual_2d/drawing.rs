// SPDX-License-Identifier: AGPL-3.0-only
//! Edge and arrow drawing for 2D graph visualization.

use egui::{Color32, Pos2, Stroke, Vec2};

/// Returns the three vertices (tip, left, right) of an arrow head for a line from `from` to `to`.
#[must_use]
pub fn arrow_head_vertices(from: Pos2, to: Pos2, zoom: f32) -> (Pos2, Pos2, Pos2) {
    let arrow_size = 10.0 * zoom;
    let direction = (to - from).normalized();
    let perpendicular = Vec2::new(-direction.y, direction.x);

    let arrow_tip = to - direction * 20.0 * zoom;
    let arrow_left = arrow_tip - direction * arrow_size + perpendicular * arrow_size * 0.5;
    let arrow_right = arrow_tip - direction * arrow_size - perpendicular * arrow_size * 0.5;

    (arrow_tip, arrow_left, arrow_right)
}

/// Draw an arrow head at the target end of an edge.
pub fn draw_arrow_head(painter: &egui::Painter, from: Pos2, to: Pos2, zoom: f32) {
    let (arrow_tip, arrow_left, arrow_right) = arrow_head_vertices(from, to, zoom);

    painter.add(egui::Shape::convex_polygon(
        vec![arrow_tip, arrow_left, arrow_right],
        Color32::GRAY,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_head_vertices_horizontal() {
        let from = Pos2::new(0.0, 0.0);
        let to = Pos2::new(100.0, 0.0);
        let (tip, left, right) = arrow_head_vertices(from, to, 1.0);
        assert!(tip.x < to.x);
        assert!((tip.y - to.y).abs() < 0.001);
        assert!((left.y - right.y).abs() > 0.1);
        assert!((left.x - right.x).abs() < 0.001);
    }

    #[test]
    fn arrow_head_vertices_zoom_scales() {
        let from = Pos2::new(0.0, 0.0);
        let to = Pos2::new(100.0, 0.0);
        let (_tip1, left1, right1) = arrow_head_vertices(from, to, 1.0);
        let (_tip2, left2, right2) = arrow_head_vertices(from, to, 2.0);
        let dist1 = (left1 - right1).length();
        let dist2 = (left2 - right2).length();
        assert!(2.0f32.mul_add(-dist1, dist2).abs() < 0.1);
    }

    #[test]
    fn arrow_head_vertices_vertical() {
        let from = Pos2::new(50.0, 100.0);
        let to = Pos2::new(50.0, 0.0);
        let (tip, left, right) = arrow_head_vertices(from, to, 1.0);
        assert!((tip.x - to.x).abs() < 0.001);
        assert!(tip.y > to.y);
        assert!((left.x - right.x).abs() > 0.1);
        assert!((left.y - right.y).abs() < 0.001);
    }

    #[test]
    fn arrow_head_vertices_diagonal() {
        let from = Pos2::new(0.0, 0.0);
        let to = Pos2::new(100.0, 100.0);
        let (tip, left, right) = arrow_head_vertices(from, to, 1.0);
        assert!(tip.x < to.x);
        assert!(tip.y < to.y);
        assert!((tip.x - tip.y).abs() < 1.0);
        let tip_to_left = (left - tip).length();
        let tip_to_right = (right - tip).length();
        assert!(tip_to_left > 0.1);
        assert!(tip_to_right > 0.1);
    }

    #[test]
    fn arrow_head_vertices_zero_zoom() {
        let from = Pos2::new(0.0, 0.0);
        let to = Pos2::new(100.0, 0.0);
        let (tip, _left, _right) = arrow_head_vertices(from, to, 0.1);
        assert!(tip.x < to.x);
    }
}
