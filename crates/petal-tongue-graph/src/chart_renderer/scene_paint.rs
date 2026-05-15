// SPDX-License-Identifier: AGPL-3.0-or-later
//! Minimal SceneGraph → egui painting for converged render path.
//!
//! This mirrors the painting logic in `petal-tongue-ui/src/scene_bridge/paint`
//! so that `draw_channel` can render SceneGraph primitives without depending
//! on `petal-tongue-ui` (which would create a circular dependency).

use egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, Vec2};
use petal_tongue_scene::primitive::{AnchorPoint, Color, Primitive, StrokeStyle};
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::transform::Transform2D;

fn to_color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

fn to_egui_stroke(s: &StrokeStyle) -> Stroke {
    Stroke::new(s.width, to_color32(s.color))
}

fn anchor_to_align2(anchor: &AnchorPoint) -> egui::Align2 {
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

fn world_to_screen(transform: &Transform2D, offset: Vec2, x: f64, y: f64) -> Pos2 {
    let (tx, ty) = transform.apply(x, y);
    Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y)
}

fn world_points_to_screen(
    transform: &Transform2D,
    offset: Vec2,
    points: &[[f64; 2]],
) -> Vec<Pos2> {
    points
        .iter()
        .map(|[x, y]| world_to_screen(transform, offset, *x, *y))
        .collect()
}

fn paint_primitive(painter: &Painter, prim: &Primitive, transform: &Transform2D, offset: Vec2) {
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
            let fill_c = fill.map_or(Color32::TRANSPARENT, to_color32);
            let stroke_s = stroke.as_ref().map_or(Stroke::NONE, to_egui_stroke);
            painter.circle(center, r, fill_c, stroke_s);
        }

        Primitive::Line {
            points,
            stroke: s,
            closed,
            ..
        } => {
            let pts = world_points_to_screen(transform, offset, points);
            if pts.len() >= 2 {
                let egui_stroke = to_egui_stroke(s);
                if *closed && pts.len() >= 3 {
                    painter.add(egui::Shape::closed_line(pts, egui_stroke));
                } else {
                    painter.add(egui::Shape::line(pts, egui_stroke));
                }
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
            let fill_c = fill.map_or(Color32::TRANSPARENT, to_color32);
            let stroke_s = stroke.as_ref().map_or(Stroke::NONE, to_egui_stroke);
            let rounding = Rounding::same(*corner_radius as f32);
            painter.rect(rect, rounding, fill_c, stroke_s);
        }

        Primitive::Text {
            x,
            y,
            content,
            font_size,
            color,
            anchor,
            ..
        } => {
            let pos = world_to_screen(transform, offset, *x, *y);
            let font = egui::FontId::proportional(*font_size as f32);
            let align = anchor_to_align2(anchor);
            let text_color = to_color32(*color);
            painter.text(pos, align, content, font, text_color);
        }

        Primitive::Polygon {
            points,
            fill,
            stroke,
            ..
        } => {
            let pts = world_points_to_screen(transform, offset, points);
            let fill_c = to_color32(*fill);
            let stroke_s = stroke.as_ref().map_or(Stroke::NONE, to_egui_stroke);
            if pts.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(pts, fill_c, stroke_s));
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
                let fill_c = to_color32(*fill_color);
                painter.add(egui::Shape::convex_polygon(fan, fill_c, Stroke::NONE));
            }
            if let Some(s) = stroke {
                painter.add(egui::Shape::line(pts, to_egui_stroke(s)));
            }
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
                return;
            }
            if let Some(fill_color) = fill {
                let fill_c = to_color32(*fill_color);
                painter.add(egui::Shape::convex_polygon(
                    pts.clone(),
                    fill_c,
                    Stroke::NONE,
                ));
            }
            painter.add(egui::Shape::line(pts, to_egui_stroke(s)));
        }

        Primitive::Mesh {
            vertices, indices, ..
        } => {
            if vertices.is_empty() || indices.is_empty() {
                return;
            }
            let mut mesh = egui::Mesh::default();
            for v in vertices {
                let pos = world_to_screen(transform, offset, v.position[0], v.position[1]);
                let color = Color32::from_rgba_unmultiplied(
                    (v.color.r * 255.0) as u8,
                    (v.color.g * 255.0) as u8,
                    (v.color.b * 255.0) as u8,
                    (v.color.a * 255.0) as u8,
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
            painter.add(egui::Shape::mesh(mesh));
        }

        Primitive::Texture {
            x,
            y,
            width,
            height,
            tint,
            ..
        } => {
            let min = world_to_screen(transform, offset, *x, *y);
            let max = world_to_screen(transform, offset, x + width, y + height);
            let rect = Rect::from_min_max(min, max);
            let fill_c = tint.map_or(Color32::from_gray(180), to_color32);
            painter.rect(rect, Rounding::ZERO, fill_c, Stroke::new(1.0, Color32::GRAY));
        }
    }
}

/// Paint a `SceneGraph` into an egui `Painter` at the given screen offset.
#[expect(dead_code, reason = "superseded by scene_bridge::paint::paint_scene which adds hit-map tracking")]
pub fn paint_scene(painter: &Painter, scene: &SceneGraph, offset: Vec2) {
    for (transform, prim, _node_id, _opacity) in scene.flatten_with_opacity() {
        paint_primitive(painter, prim, &transform, offset);
    }
}

/// Compile a `DataBinding` through the full primal pipeline and paint the
/// resulting `SceneGraph` into an egui `Ui`.
///
/// Returns `true` if rendering succeeded, `false` if compilation produced
/// an empty scene.
pub fn draw_binding_via_scene(
    ui: &mut egui::Ui,
    binding: &petal_tongue_core::DataBinding,
    domain: Option<&str>,
) -> bool {
    use petal_tongue_scene::compiler::GrammarCompiler;
    use petal_tongue_scene::data_binding::DataBindingCompiler;

    let (grammar_expr, data) = DataBindingCompiler::compile(binding, domain);
    let title = grammar_expr
        .title
        .clone()
        .unwrap_or_default();
    let compiler = GrammarCompiler::new();
    let scene = compiler.compile(&grammar_expr, &data);

    let flat = scene.flatten_with_opacity();
    if flat.is_empty() {
        return false;
    }

    if !title.is_empty() {
        ui.label(
            egui::RichText::new(&title)
                .strong()
                .color(egui::Color32::LIGHT_GRAY),
        );
    }

    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (transform, prim, _, _) in &flat {
        let (ox, oy) = prim_origin(prim);
        let (tx, ty) = transform.apply(ox, oy);
        min_x = min_x.min(tx);
        min_y = min_y.min(ty);
        max_x = max_x.max(tx);
        max_y = max_y.max(ty);
    }
    let scene_w = (max_x - min_x).max(1.0);
    let scene_h = (max_y - min_y).max(1.0);

    let avail_w = ui.available_width().min(500.0);
    let scale = avail_w as f64 / scene_w;
    let display_h = (scene_h * scale) as f32;
    let display_h = display_h.clamp(60.0, 400.0);

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(avail_w, display_h),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);

    let scale_x = f64::from(avail_w) / scene_w;
    let scale_y = f64::from(display_h) / scene_h;
    let uniform_scale = scale_x.min(scale_y);

    let view_transform = Transform2D {
        a: uniform_scale,
        b: 0.0,
        tx: -min_x * uniform_scale,
        c: 0.0,
        d: uniform_scale,
        ty: -min_y * uniform_scale,
    };

    let offset = rect.min.to_vec2();

    for (node_transform, prim, _node_id, _opacity) in &flat {
        let composed = view_transform.then(*node_transform);
        paint_primitive(&painter, prim, &composed, offset);
    }

    true
}

fn prim_origin(prim: &Primitive) -> (f64, f64) {
    match prim {
        Primitive::Point { x, y, .. } => (*x, *y),
        Primitive::Rect { x, y, .. } => (*x, *y),
        Primitive::Text { x, y, .. } => (*x, *y),
        Primitive::Arc { cx, cy, .. } => (*cx, *cy),
        Primitive::Line { points, .. } => points.first().map_or((0.0, 0.0), |p| (p[0], p[1])),
        Primitive::Polygon { points, .. } => points.first().map_or((0.0, 0.0), |p| (p[0], p[1])),
        Primitive::BezierPath { start, .. } => (start[0], start[1]),
        Primitive::Mesh { .. } | Primitive::Texture { .. } => (0.0, 0.0),
    }
}
