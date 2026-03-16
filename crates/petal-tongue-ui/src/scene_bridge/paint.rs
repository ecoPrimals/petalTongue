// SPDX-License-Identifier: AGPL-3.0-only
//! Primitive painting and conversion helpers for scene bridge.

use super::types::{FrameHitMap, PixelProvenance};
use egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, Vec2};
use petal_tongue_scene::primitive::{AnchorPoint, Color, Primitive};
use petal_tongue_scene::render_plan::RenderPlan;
use petal_tongue_scene::scene_graph::SceneGraph;
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

/// Apply accumulated opacity to a `Color32`.
pub fn apply_opacity(c: Color32, opacity: f32) -> Color32 {
    if opacity >= 1.0 {
        return c;
    }
    let a = (f32::from(c.a()) * opacity).round() as u8;
    Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), a)
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

pub fn to_color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

pub fn to_egui_stroke(s: &petal_tongue_scene::primitive::StrokeStyle) -> Stroke {
    Stroke::new(s.width, to_color32(s.color))
}

/// Render a single primitive with its world transform and accumulated opacity.
///
/// Returns the bounding `Rect` on screen for the rendered shape, or `None`
/// if the primitive was not drawn (e.g. degenerate).
#[expect(clippy::cast_possible_truncation, reason = "scene f64 to screen f32")]
pub fn paint_primitive(
    painter: &Painter,
    prim: &Primitive,
    transform: &Transform2D,
    offset: Vec2,
    opacity: f32,
) -> Option<Rect> {
    match prim {
        Primitive::Point {
            x,
            y,
            radius,
            fill,
            stroke,
            ..
        } => {
            let (tx, ty) = transform.apply(*x, *y);
            let center = Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y);
            let r = *radius as f32;
            let fill_c = apply_opacity(
                fill.map(to_color32).unwrap_or(Color32::TRANSPARENT),
                opacity,
            );
            let stroke_s = stroke.as_ref().map_or(Stroke::NONE, to_egui_stroke);
            painter.circle(center, r, fill_c, stroke_s);
            Some(Rect::from_center_size(center, egui::vec2(r * 2.0, r * 2.0)))
        }

        Primitive::Line {
            points,
            stroke: s,
            closed,
            ..
        } => {
            let egui_stroke = to_egui_stroke(s);
            let pts: Vec<Pos2> = points
                .iter()
                .map(|[x, y]| {
                    let (tx, ty) = transform.apply(*x, *y);
                    Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y)
                })
                .collect();
            if pts.len() >= 2 {
                let rect = bounding_rect(&pts);
                if *closed && pts.len() >= 3 {
                    painter.add(egui::Shape::closed_line(pts, egui_stroke));
                } else {
                    painter.add(egui::Shape::line(pts, egui_stroke));
                }
                Some(rect)
            } else {
                None
            }
        }

        Primitive::Rect {
            x,
            y,
            width,
            height,
            fill,
            stroke,
            corner_radius,
            ..
        } => {
            let (tx, ty) = transform.apply(*x, *y);
            let (tx2, ty2) = transform.apply(x + width, y + height);
            let rect = Rect::from_min_max(
                Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y),
                Pos2::new(tx2 as f32 + offset.x, ty2 as f32 + offset.y),
            );
            let fill_c = apply_opacity(
                fill.map(to_color32).unwrap_or(Color32::TRANSPARENT),
                opacity,
            );
            let stroke_s = stroke.as_ref().map_or(Stroke::NONE, to_egui_stroke);
            let rounding = Rounding::same(*corner_radius as f32);
            painter.rect(rect, rounding, fill_c, stroke_s);
            Some(rect)
        }

        Primitive::Text {
            x,
            y,
            content,
            font_size,
            color,
            anchor,
            bold: _bold,
            ..
        } => {
            let (tx, ty) = transform.apply(*x, *y);
            let pos = Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y);
            let font = egui::FontId::proportional(*font_size as f32);
            let align = anchor_to_align2(anchor);
            let text_color = apply_opacity(to_color32(*color), opacity);
            let galley = painter.text(pos, align, content, font, text_color);
            Some(galley)
        }

        Primitive::Polygon {
            points,
            fill,
            stroke,
            ..
        } => {
            let pts: Vec<Pos2> = points
                .iter()
                .map(|[x, y]| {
                    let (tx, ty) = transform.apply(*x, *y);
                    Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y)
                })
                .collect();
            let fill_c = apply_opacity(to_color32(*fill), opacity);
            let stroke_s = stroke.as_ref().map_or(Stroke::NONE, to_egui_stroke);
            if pts.len() >= 3 {
                let rect = bounding_rect(&pts);
                painter.add(egui::Shape::convex_polygon(pts, fill_c, stroke_s));
                Some(rect)
            } else {
                None
            }
        }

        Primitive::Arc {
            cx,
            cy,
            radius,
            start_angle,
            end_angle,
            fill,
            stroke,
            ..
        } => {
            let segments = 32;
            let angle_span = end_angle - start_angle;
            let pts: Vec<Pos2> = (0..=segments)
                .map(|i| {
                    let t = angle_span.mul_add(i as f64 / segments as f64, *start_angle);
                    let px = cx + radius * t.cos();
                    let py = cy + radius * t.sin();
                    let (tx, ty) = transform.apply(px, py);
                    Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y)
                })
                .collect();
            if let Some(fill_color) = fill {
                let mut fan = vec![{
                    let (tx, ty) = transform.apply(*cx, *cy);
                    Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y)
                }];
                fan.extend_from_slice(&pts);
                let fill_c = apply_opacity(to_color32(*fill_color), opacity);
                painter.add(egui::Shape::convex_polygon(fan, fill_c, Stroke::NONE));
            }
            let rect = bounding_rect(&pts);
            if let Some(s) = stroke {
                painter.add(egui::Shape::line(pts, to_egui_stroke(s)));
            }
            Some(rect)
        }

        Primitive::BezierPath {
            start,
            segments,
            stroke: s,
            fill,
            ..
        } => {
            let mut pts = Vec::new();
            let (sx, sy) = transform.apply(start[0], start[1]);
            pts.push(Pos2::new(sx as f32 + offset.x, sy as f32 + offset.y));

            let mut cur = *start;
            for seg in segments {
                let steps = 16;
                let p0 = cur;
                for i in 1..=steps {
                    let t = i as f64 / steps as f64;
                    let mt = 1.0 - t;
                    let mt2 = mt * mt;
                    let mt3 = mt2 * mt;
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let px = (3.0 * mt * t2)
                        .mul_add(seg.cp2[0], mt3 * p0[0] + 3.0 * mt2 * t * seg.cp1[0])
                        + t3 * seg.end[0];
                    let py = (3.0 * mt * t2)
                        .mul_add(seg.cp2[1], mt3 * p0[1] + 3.0 * mt2 * t * seg.cp1[1])
                        + t3 * seg.end[1];
                    let (tx, ty) = transform.apply(px, py);
                    pts.push(Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y));
                }
                cur = seg.end;
            }

            if pts.len() < 2 {
                return None;
            }

            let rect = bounding_rect(&pts);
            if let Some(fill_color) = fill {
                let fill_c = apply_opacity(to_color32(*fill_color), opacity);
                painter.add(egui::Shape::convex_polygon(
                    pts.clone(),
                    fill_c,
                    Stroke::NONE,
                ));
            }
            painter.add(egui::Shape::line(pts, to_egui_stroke(s)));
            Some(rect)
        }

        Primitive::Mesh {
            vertices, indices, ..
        } => {
            if vertices.is_empty() || indices.is_empty() {
                return None;
            }
            let mut mesh = egui::Mesh::default();
            for v in vertices {
                let (tx, ty) = transform.apply(v.position[0], v.position[1]);
                let color = Color32::from_rgba_unmultiplied(
                    (v.color.r * 255.0) as u8,
                    (v.color.g * 255.0) as u8,
                    (v.color.b * 255.0) as u8,
                    ((v.color.a * opacity) * 255.0) as u8,
                );
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y),
                    uv: Pos2::ZERO,
                    color,
                });
            }
            for idx in indices {
                mesh.indices.push(*idx);
            }
            let rect = mesh.calc_bounds();
            painter.add(egui::Shape::mesh(mesh));
            Some(rect)
        }
    }
}

/// Render a scene graph into an egui `Painter`, building a `FrameHitMap`.
///
/// Applies accumulated node opacity to each primitive's colors.
pub fn paint_scene_tracked(painter: &Painter, scene: &SceneGraph, offset: Vec2) -> FrameHitMap {
    let mut hit_map = FrameHitMap::new();
    for (idx, (transform, prim, node_id, opacity)) in
        scene.flatten_with_opacity().into_iter().enumerate()
    {
        let screen_rect = paint_primitive(painter, prim, &transform, offset, opacity);
        if let Some(rect) = screen_rect {
            let (wx, wy) = primitive_origin(prim);
            let (wx, wy) = transform.apply(wx, wy);
            hit_map.register(
                rect,
                PixelProvenance {
                    node_id: node_id.to_string(),
                    primitive_index: idx,
                    data_id: prim.data_id().map(String::from),
                    world_x: wx,
                    world_y: wy,
                },
            );
        }
    }
    hit_map
}

/// Legacy entry point — renders without tracking.
pub fn paint_scene(painter: &Painter, scene: &SceneGraph, offset: Vec2) {
    for (transform, prim, _node_id, opacity) in scene.flatten_with_opacity() {
        paint_primitive(painter, prim, &transform, offset, opacity);
    }
}

/// Render a full `RenderPlan` into an egui `Painter`, returning the hit map.
pub fn paint_plan_tracked(painter: &Painter, plan: &RenderPlan, offset: Vec2) -> FrameHitMap {
    paint_scene_tracked(painter, &plan.scene, offset)
}

/// Legacy entry point — renders without tracking.
pub fn paint_plan(painter: &Painter, plan: &RenderPlan, offset: Vec2) {
    paint_scene(painter, &plan.scene, offset);
}
