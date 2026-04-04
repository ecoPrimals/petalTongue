// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primitive painting and conversion helpers for scene bridge.

mod color;
mod geometry;
mod primitives;

#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "re-exported from private `color` for scene_bridge callers; not all symbols referenced in this module"
    )
)]
pub use color::{apply_opacity, to_color32, to_egui_stroke};
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "re-exported from private `geometry` for tests and callers; not all symbols used in non-test lib builds"
    )
)]
pub use geometry::{anchor_to_align2, bounding_rect, primitive_origin};
pub use primitives::paint_primitive;

use super::types::{FrameHitMap, PixelProvenance};
use egui::{Painter, Vec2};
use petal_tongue_scene::render_plan::RenderPlan;
use petal_tongue_scene::scene_graph::SceneGraph;

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
        let pts: Vec<egui::Pos2> = vec![];
        let r = bounding_rect(&pts);
        assert!(r.width() <= 0.0 || r.height() <= 0.0);
    }

    #[test]
    fn bounding_rect_single_point() {
        let pts = vec![egui::Pos2::new(42.0, 84.0)];
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
