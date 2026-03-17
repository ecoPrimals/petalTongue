// SPDX-License-Identifier: AGPL-3.0-or-later
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
                    let t = angle_span.mul_add(f64::from(i) / f64::from(segments), *start_angle);
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
#[must_use]
pub fn paint_plan_tracked(painter: &Painter, plan: &RenderPlan, offset: Vec2) -> FrameHitMap {
    paint_scene_tracked(painter, &plan.scene, offset)
}

/// Legacy entry point — renders without tracking.
pub fn paint_plan(painter: &Painter, plan: &RenderPlan, offset: Vec2) {
    paint_scene(painter, &plan.scene, offset);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use egui::{Painter, Sense, Vec2};
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};
    use petal_tongue_scene::primitive::{
        AnchorPoint, BezierSegment, Color, FillRule, MeshVertex, Primitive, StrokeStyle,
    };
    use petal_tongue_scene::render_plan::RenderPlan;
    use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
    use petal_tongue_scene::transform::Transform2D;

    fn with_egui_painter(size: Vec2, f: impl FnOnce(&Painter, Vec2)) {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) = ui.allocate_painter(size, Sense::hover());
                if let Some(callback) = f.take() {
                    callback(&painter, response.rect.min.to_vec2());
                }
            });
        });
    }

    #[test]
    fn anchor_to_align2_all_variants() {
        assert_eq!(
            anchor_to_align2(&AnchorPoint::TopLeft),
            egui::Align2::LEFT_TOP
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::TopCenter),
            egui::Align2::CENTER_TOP
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::TopRight),
            egui::Align2::RIGHT_TOP
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::CenterLeft),
            egui::Align2::LEFT_CENTER
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::Center),
            egui::Align2::CENTER_CENTER
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::CenterRight),
            egui::Align2::RIGHT_CENTER
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::BottomLeft),
            egui::Align2::LEFT_BOTTOM
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::BottomCenter),
            egui::Align2::CENTER_BOTTOM
        );
        assert_eq!(
            anchor_to_align2(&AnchorPoint::BottomRight),
            egui::Align2::RIGHT_BOTTOM
        );
    }

    #[test]
    fn apply_opacity_full_returns_unchanged() {
        let c = egui::Color32::from_rgb(100, 150, 200);
        let result = apply_opacity(c, 1.0);
        assert_eq!(result.r(), c.r());
        assert_eq!(result.g(), c.g());
        assert_eq!(result.b(), c.b());
        assert_eq!(result.a(), c.a());
    }

    #[test]
    fn apply_opacity_half_reduces_alpha() {
        let c = egui::Color32::from_rgba_unmultiplied(100, 150, 200, 255);
        let result = apply_opacity(c, 0.5);
        assert_eq!(result.r(), 100);
        assert_eq!(result.g(), 150);
        assert_eq!(result.b(), 200);
        assert!(result.a() < 255);
        assert!(result.a() >= 127);
    }

    #[test]
    fn primitive_origin_point() {
        let p = Primitive::Point {
            x: 10.0,
            y: 20.0,
            radius: 5.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 10.0).abs() < f64::EPSILON);
        assert!((y - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_rect() {
        let p = Primitive::Rect {
            x: 5.0,
            y: 15.0,
            width: 100.0,
            height: 50.0,
            fill: None,
            stroke: None,
            corner_radius: 0.0,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 5.0).abs() < f64::EPSILON);
        assert!((y - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_text() {
        let p = Primitive::Text {
            x: 42.0,
            y: 84.0,
            content: "Hi".to_string(),
            font_size: 12.0,
            color: Color::BLACK,
            anchor: AnchorPoint::Center,
            bold: false,
            italic: false,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 42.0).abs() < f64::EPSILON);
        assert!((y - 84.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_arc() {
        let p = Primitive::Arc {
            cx: 50.0,
            cy: 60.0,
            radius: 20.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::PI,
            fill: None,
            stroke: None,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 50.0).abs() < f64::EPSILON);
        assert!((y - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_line_empty() {
        let p = Primitive::Line {
            points: vec![],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 0.0).abs() < f64::EPSILON);
        assert!((y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_line_with_points() {
        let p = Primitive::Line {
            points: vec![[1.0, 2.0], [3.0, 4.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 1.0).abs() < f64::EPSILON);
        assert!((y - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_polygon_empty() {
        let p = Primitive::Polygon {
            points: vec![],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 0.0).abs() < f64::EPSILON);
        assert!((y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_polygon_with_points() {
        let p = Primitive::Polygon {
            points: vec![[10.0, 20.0], [30.0, 40.0], [50.0, 60.0]],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 10.0).abs() < f64::EPSILON);
        assert!((y - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_bezier_path() {
        let p = Primitive::BezierPath {
            start: [7.0, 8.0],
            segments: vec![],
            stroke: StrokeStyle::default(),
            fill: None,
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 7.0).abs() < f64::EPSILON);
        assert!((y - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_origin_mesh() {
        let p = Primitive::Mesh {
            vertices: vec![],
            indices: vec![],
            data_id: None,
        };
        let (x, y) = primitive_origin(&p);
        assert!((x - 0.0).abs() < f64::EPSILON);
        assert!((y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bounding_rect_empty() {
        let pts: Vec<Pos2> = vec![];
        let r = bounding_rect(&pts);
        assert!(r.width() <= 0.0 || r.height() <= 0.0);
    }

    #[test]
    fn bounding_rect_single_point() {
        let pts = vec![Pos2::new(42.0, 84.0)];
        let r = bounding_rect(&pts);
        assert!((r.min.x - 42.0).abs() < f32::EPSILON);
        assert!((r.min.y - 84.0).abs() < f32::EPSILON);
        assert!((r.max.x - 42.0).abs() < f32::EPSILON);
        assert!((r.max.y - 84.0).abs() < f32::EPSILON);
    }

    #[test]
    fn paint_primitive_arc_with_fill() {
        let prim = Primitive::Arc {
            cx: 50.0,
            cy: 50.0,
            radius: 30.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::FRAC_PI_2,
            fill: Some(Color::rgb(1.0, 0.0, 0.0)),
            stroke: None,
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_some());
        });
    }

    #[test]
    fn paint_primitive_arc_with_stroke() {
        let prim = Primitive::Arc {
            cx: 50.0,
            cy: 50.0,
            radius: 30.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::PI,
            fill: None,
            stroke: Some(StrokeStyle {
                color: Color::BLACK,
                width: 2.0,
                ..StrokeStyle::default()
            }),
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_some());
        });
    }

    #[test]
    fn paint_primitive_arc_with_fill_and_stroke() {
        let prim = Primitive::Arc {
            cx: 50.0,
            cy: 50.0,
            radius: 30.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::TAU,
            fill: Some(Color::rgb(0.0, 1.0, 0.0)),
            stroke: Some(StrokeStyle::default()),
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_some());
        });
    }

    #[test]
    fn paint_primitive_bezier_path_with_segments() {
        let prim = Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![BezierSegment {
                cp1: [20.0, 0.0],
                cp2: [40.0, 40.0],
                end: [50.0, 50.0],
            }],
            stroke: StrokeStyle::default(),
            fill: Some(Color::WHITE),
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_some());
        });
    }

    #[test]
    fn paint_primitive_bezier_path_empty_segments_returns_none() {
        let prim = Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![],
            stroke: StrokeStyle::default(),
            fill: None,
            fill_rule: FillRule::NonZero,
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_none());
        });
    }

    #[test]
    fn paint_primitive_mesh_empty_returns_none() {
        let prim = Primitive::Mesh {
            vertices: vec![],
            indices: vec![],
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_none());
        });
    }

    #[test]
    fn paint_primitive_mesh_empty_indices_returns_none() {
        let prim = Primitive::Mesh {
            vertices: vec![MeshVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                color: Color::WHITE,
            }],
            indices: vec![],
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_none());
        });
    }

    #[test]
    fn paint_primitive_mesh_with_data() {
        let prim = Primitive::Mesh {
            vertices: vec![
                MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    color: Color::WHITE,
                },
                MeshVertex {
                    position: [100.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    color: Color::WHITE,
                },
                MeshVertex {
                    position: [50.0, 86.6, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    color: Color::WHITE,
                },
            ],
            indices: vec![0, 1, 2],
            data_id: Some("mesh-1".to_string()),
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 1.0);
            assert!(result.is_some());
        });
    }

    #[test]
    fn paint_primitive_arc_with_opacity() {
        let prim = Primitive::Arc {
            cx: 50.0,
            cy: 50.0,
            radius: 30.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::FRAC_PI_2,
            fill: Some(Color::rgba(1.0, 0.0, 0.0, 0.8)),
            stroke: None,
            data_id: None,
        };
        let transform = Transform2D::IDENTITY;
        with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
            let result = paint_primitive(painter, &prim, &transform, offset, 0.5);
            assert!(result.is_some());
        });
    }

    #[test]
    fn paint_plan_tracked_builds_hit_map() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("n").with_primitive(Primitive::Point {
            x: 10.0,
            y: 20.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("pt-1".to_string()),
        }));
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let plan = RenderPlan::new(graph, grammar);
        with_egui_painter(Vec2::new(400.0, 300.0), |painter, offset| {
            let hit_map = paint_plan_tracked(painter, &plan, offset);
            assert_eq!(hit_map.len(), 1);
        });
    }

    #[test]
    fn paint_plan_renders() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("n").with_primitive(Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 3.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        }));
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let plan = RenderPlan::new(graph, grammar);
        with_egui_painter(Vec2::new(400.0, 300.0), |painter, offset| {
            paint_plan(painter, &plan, offset);
        });
    }
}
