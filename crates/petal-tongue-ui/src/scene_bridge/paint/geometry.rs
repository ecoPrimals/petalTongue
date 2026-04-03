// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinate mapping, anchors, and bounds for scene painting.

use egui::{Pos2, Rect, Vec2};
use petal_tongue_scene::primitive::{AnchorPoint, Primitive};
use petal_tongue_scene::transform::Transform2D;

/// Map `AnchorPoint` to `egui::Align2`.
pub const fn anchor_to_align2(anchor: &AnchorPoint) -> egui::Align2 {
    match anchor {
        AnchorPoint::TopLeft => egui::Align2::LEFT_TOP,
        AnchorPoint::TopCenter => egui::Align2::CENTER_TOP,
        AnchorPoint::TopRight => egui::Align2::RIGHT_TOP,
        AnchorPoint::CenterLeft => egui::Align2::LEFT_CENTER,
        AnchorPoint::Center => egui::Align2::CENTER_CENTER,
        AnchorPoint::CenterRight => egui::Align2::RIGHT_CENTER,
        AnchorPoint::BottomLeft => egui::Align2::LEFT_BOTTOM,
        AnchorPoint::BottomCenter => egui::Align2::CENTER_BOTTOM,
        AnchorPoint::BottomRight => egui::Align2::RIGHT_BOTTOM,
    }
}

/// Extract the logical origin point of a primitive (used for provenance).
pub fn primitive_origin(prim: &Primitive) -> (f64, f64) {
    match prim {
        Primitive::Point { x, y, .. } => (*x, *y),
        Primitive::Rect { x, y, .. } => (*x, *y),
        Primitive::Text { x, y, .. } => (*x, *y),
        Primitive::Arc { cx, cy, .. } => (*cx, *cy),
        Primitive::Line { points, .. } => points.first().map_or((0.0, 0.0), |p| (p[0], p[1])),
        Primitive::Polygon { points, .. } => points.first().map_or((0.0, 0.0), |p| (p[0], p[1])),
        Primitive::BezierPath { start, .. } => (start[0], start[1]),
        Primitive::Mesh { .. } => (0.0, 0.0),
    }
}

/// Compute the axis-aligned bounding box of a set of screen points.
pub fn bounding_rect(pts: &[Pos2]) -> Rect {
    let mut min = Pos2::new(f32::MAX, f32::MAX);
    let mut max = Pos2::new(f32::MIN, f32::MIN);
    for p in pts {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    Rect::from_min_max(min, max)
}

/// Transform a world-space (f64, f64) point through `Transform2D` and apply screen offset.
#[expect(clippy::cast_possible_truncation, reason = "scene f64 to screen f32")]
pub fn world_to_screen(transform: &Transform2D, offset: Vec2, x: f64, y: f64) -> Pos2 {
    let (tx, ty) = transform.apply(x, y);
    Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y)
}

/// Transform a slice of world-space [f64; 2] points to screen-space `Pos2`.
pub fn world_points_to_screen(
    transform: &Transform2D,
    offset: Vec2,
    points: &[[f64; 2]],
) -> Vec<Pos2> {
    points
        .iter()
        .map(|[x, y]| world_to_screen(transform, offset, *x, *y))
        .collect()
}
