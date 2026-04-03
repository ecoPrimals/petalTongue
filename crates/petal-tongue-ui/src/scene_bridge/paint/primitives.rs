// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-primitive-type painting into an egui `Painter`.

use super::color::{apply_opacity, to_color32, to_egui_stroke};
use super::geometry::{anchor_to_align2, bounding_rect, world_points_to_screen, world_to_screen};
use egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, Vec2};
use petal_tongue_scene::primitive::Primitive;
use petal_tongue_scene::transform::Transform2D;

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
            let center = world_to_screen(transform, offset, *x, *y);
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
            let pts = world_points_to_screen(transform, offset, points);
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
            let min = world_to_screen(transform, offset, *x, *y);
            let max = world_to_screen(transform, offset, x + width, y + height);
            let rect = Rect::from_min_max(min, max);
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
            let pos = world_to_screen(transform, offset, *x, *y);
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
            let pts = world_points_to_screen(transform, offset, points);
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
                    let t = angle_span.mul_add(f64::from(i) / f64::from(segments), *start_angle);
                    let px = cx + radius * t.cos();
                    let py = cy + radius * t.sin();
                    world_to_screen(transform, offset, px, py)
                })
                .collect();
            if let Some(fill_color) = fill {
                let mut fan = vec![world_to_screen(transform, offset, *cx, *cy)];
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
            let mut pts = vec![world_to_screen(transform, offset, start[0], start[1])];

            let mut cur = *start;
            for seg in segments {
                let steps = 16;
                let p0 = cur;
                for i in 1..=steps {
                    let t = f64::from(i) / f64::from(steps);
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
                    pts.push(world_to_screen(transform, offset, px, py));
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
                let pos = world_to_screen(transform, offset, v.position[0], v.position[1]);
                let color = Color32::from_rgba_unmultiplied(
                    (v.color.r * 255.0) as u8,
                    (v.color.g * 255.0) as u8,
                    (v.color.b * 255.0) as u8,
                    ((v.color.a * opacity) * 255.0) as u8,
                );
                mesh.vertices.push(egui::epaint::Vertex {
                    pos,
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
