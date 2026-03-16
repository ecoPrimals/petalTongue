// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene bridge unit tests.

use super::SceneWidget;
use super::paint::{bounding_rect, paint_primitive, paint_scene, paint_scene_tracked};
use super::paint::{primitive_origin, to_color32, to_egui_stroke};
use super::types::{FrameHitMap, PixelProvenance};
use egui::{Painter, Pos2, Sense, Vec2};
use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};
use petal_tongue_scene::primitive::{Color, Primitive, StrokeStyle};
use petal_tongue_scene::render_plan::RenderPlan;
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::scene_graph::SceneNode;
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
fn color_conversion_preserves_extremes() {
    let black = to_color32(Color::BLACK);
    assert_eq!(black.r(), 0);
    assert_eq!(black.g(), 0);
    assert_eq!(black.b(), 0);

    let white = to_color32(Color::WHITE);
    assert_eq!(white.r(), 255);
    assert_eq!(white.g(), 255);
    assert_eq!(white.b(), 255);
}

#[test]
fn color_conversion_alpha() {
    let transparent = to_color32(Color::TRANSPARENT);
    assert_eq!(transparent.a(), 0);
}

#[test]
fn stroke_conversion() {
    let s = StrokeStyle {
        color: Color::WHITE,
        width: 2.0,
        ..StrokeStyle::default()
    };
    let egui_s = to_egui_stroke(&s);
    assert!((egui_s.width - 2.0).abs() < f32::EPSILON);
}

#[test]
fn stroke_style_conversion() {
    let s = StrokeStyle {
        color: Color::from_rgba8(128, 128, 128, 255),
        width: 3.0,
        ..StrokeStyle::default()
    };
    let egui_s = to_egui_stroke(&s);
    assert!((egui_s.width - 3.0).abs() < f32::EPSILON);
}

#[test]
fn scene_bridge_handles_empty_graph() {
    let graph = SceneGraph::new();
    let flat = graph.flatten();
    assert!(flat.is_empty());
}

#[test]
fn scene_bridge_flattens_with_primitives() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
        x: 10.0,
        y: 20.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    }));
    let flat = graph.flatten();
    assert_eq!(flat.len(), 1);
}

#[test]
fn scene_widget_creation() {
    let graph = SceneGraph::new();
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let plan = RenderPlan::new(graph, grammar);

    let _widget = SceneWidget::new(&plan);
}

#[test]
fn scene_widget_desired_size_builder() {
    let graph = SceneGraph::new();
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let plan = RenderPlan::new(graph, grammar);

    let _widget = SceneWidget::new(&plan).desired_size(egui::Vec2::new(800.0, 600.0));
}

#[test]
fn paint_plan_delegates_to_paint_scene() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    }));
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let plan = RenderPlan::new(graph, grammar);
    assert!(!plan.scene.flatten().is_empty());
}

#[test]
fn paint_primitive_point_with_stroke() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
        x: 10.0,
        y: 20.0,
        radius: 5.0,
        fill: Some(Color::rgb(1.0, 0.0, 0.0)),
        stroke: Some(StrokeStyle {
            color: Color::WHITE,
            width: 2.0,
            ..StrokeStyle::default()
        }),
        data_id: None,
    }));
    let flat = graph.flatten();
    assert_eq!(flat.len(), 1);
    let (_transform, prim) = &flat[0];
    match prim {
        Primitive::Point {
            x,
            y,
            radius,
            fill,
            stroke,
            ..
        } => {
            assert!((*x - 10.0).abs() < f64::EPSILON);
            assert!((*y - 20.0).abs() < f64::EPSILON);
            assert!((*radius - 5.0).abs() < f64::EPSILON);
            assert!(fill.is_some());
            assert!(stroke.is_some());
        }
        _ => panic!("expected Point primitive"),
    }
}

#[test]
fn color_mapping_rgba() {
    let c = Color::from_rgba8(128, 64, 32, 200);
    let egui_c = egui::Color32::from_rgba_unmultiplied(128, 64, 32, 200);
    let converted = to_color32(c);
    assert_eq!(converted.r(), egui_c.r());
    assert_eq!(converted.g(), egui_c.g());
    assert_eq!(converted.b(), egui_c.b());
    assert_eq!(converted.a(), egui_c.a());
}

#[test]
fn paint_primitive_line_open() {
    let prim = Primitive::Line {
        points: vec![[0.0, 0.0], [100.0, 50.0]],
        stroke: StrokeStyle {
            color: Color::BLACK,
            width: 2.0,
            ..StrokeStyle::default()
        },
        closed: false,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 100.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_primitive_line_closed() {
    let prim = Primitive::Line {
        points: vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]],
        stroke: StrokeStyle {
            color: Color::WHITE,
            width: 1.0,
            ..StrokeStyle::default()
        },
        closed: true,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_primitive_rect() {
    let prim = Primitive::Rect {
        x: 10.0,
        y: 20.0,
        width: 80.0,
        height: 40.0,
        fill: Some(Color::rgb(1.0, 0.0, 0.0)),
        stroke: Some(StrokeStyle {
            color: Color::BLACK,
            width: 1.0,
            ..StrokeStyle::default()
        }),
        corner_radius: 4.0,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_primitive_text() {
    use petal_tongue_scene::primitive::AnchorPoint;

    let prim = Primitive::Text {
        x: 50.0,
        y: 50.0,
        content: "Test Label".to_string(),
        font_size: 14.0,
        color: Color::BLACK,
        anchor: AnchorPoint::TopLeft,
        bold: false,
        italic: false,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_primitive_polygon() {
    use petal_tongue_scene::primitive::FillRule;

    let prim = Primitive::Polygon {
        points: vec![[0.0, 0.0], [100.0, 0.0], [50.0, 86.6]],
        fill: Color::rgb(0.0, 0.5, 1.0),
        stroke: None,
        fill_rule: FillRule::NonZero,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_primitive_line_single_point_no_draw() {
    let prim = Primitive::Line {
        points: vec![[50.0, 50.0]],
        stroke: StrokeStyle::default(),
        closed: false,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_primitive_polygon_insufficient_points_no_draw() {
    use petal_tongue_scene::primitive::FillRule;

    let prim = Primitive::Polygon {
        points: vec![[0.0, 0.0], [100.0, 0.0]],
        fill: Color::BLACK,
        stroke: None,
        fill_rule: FillRule::NonZero,
        data_id: None,
    };
    let transform = Transform2D::IDENTITY;
    with_egui_painter(Vec2::new(200.0, 200.0), |painter, offset| {
        paint_primitive(painter, &prim, &transform, offset, 1.0);
    });
}

#[test]
fn paint_scene_with_offset() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    }));
    with_egui_painter(Vec2::new(400.0, 300.0), |painter, offset| {
        paint_scene(painter, &graph, offset);
    });
}

#[test]
fn scene_widget_show_headless() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
        x: 10.0,
        y: 20.0,
        radius: 3.0,
        fill: Some(Color::rgb(0.0, 1.0, 0.0)),
        stroke: None,
        data_id: None,
    }));
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let plan = RenderPlan::new(graph, grammar);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            SceneWidget::new(&plan)
                .desired_size(Vec2::new(400.0, 300.0))
                .show(ui);
        });
    });
}

#[test]
fn frame_hit_map_empty() {
    let hit_map = FrameHitMap::new();
    assert!(hit_map.is_empty());
    assert_eq!(hit_map.len(), 0);
    assert!(hit_map.query(100.0, 100.0).is_none());
}

#[test]
fn frame_hit_map_register_and_query() {
    let mut hit_map = FrameHitMap::new();
    let rect = egui::Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(50.0, 50.0));
    hit_map.register(
        rect,
        PixelProvenance {
            node_id: "node_1".to_string(),
            primitive_index: 0,
            data_id: Some("sensor_42".to_string()),
            world_x: 30.0,
            world_y: 30.0,
        },
    );
    assert_eq!(hit_map.len(), 1);

    let hit = hit_map.query(25.0, 25.0);
    assert!(hit.is_some());
    let prov = hit.unwrap();
    assert_eq!(prov.node_id, "node_1");
    assert_eq!(prov.data_id.as_deref(), Some("sensor_42"));

    assert!(hit_map.query(0.0, 0.0).is_none());
}

#[test]
fn frame_hit_map_topmost_wins() {
    let mut hit_map = FrameHitMap::new();
    let rect = egui::Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
    hit_map.register(
        rect,
        PixelProvenance {
            node_id: "bg".to_string(),
            primitive_index: 0,
            data_id: None,
            world_x: 0.0,
            world_y: 0.0,
        },
    );
    hit_map.register(
        egui::Rect::from_min_max(Pos2::new(20.0, 20.0), Pos2::new(80.0, 80.0)),
        PixelProvenance {
            node_id: "fg".to_string(),
            primitive_index: 1,
            data_id: Some("top_data".to_string()),
            world_x: 50.0,
            world_y: 50.0,
        },
    );
    let hit = hit_map.query(50.0, 50.0).unwrap();
    assert_eq!(hit.node_id, "fg");
}

#[test]
fn paint_scene_tracked_builds_hit_map() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("r").with_primitive(Primitive::Rect {
        x: 10.0,
        y: 20.0,
        width: 80.0,
        height: 40.0,
        fill: Some(Color::rgb(1.0, 0.0, 0.0)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("rect_data".to_string()),
    }));
    graph.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
        x: 50.0,
        y: 50.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: Some("point_data".to_string()),
    }));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::Vec2::new(400.0, 300.0), egui::Sense::hover());
            let hit_map = paint_scene_tracked(&painter, &graph, response.rect.min.to_vec2());
            assert_eq!(hit_map.len(), 2);
        });
    });
}

#[test]
fn primitive_origin_extracts_xy() {
    let p = Primitive::Point {
        x: 10.0,
        y: 20.0,
        radius: 5.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    let (ox, oy) = primitive_origin(&p);
    assert!((ox - 10.0).abs() < f64::EPSILON);
    assert!((oy - 20.0).abs() < f64::EPSILON);
}

#[test]
fn bounding_rect_from_points() {
    let pts = vec![
        Pos2::new(10.0, 20.0),
        Pos2::new(50.0, 80.0),
        Pos2::new(30.0, 50.0),
    ];
    let r = bounding_rect(&pts);
    assert!((r.min.x - 10.0).abs() < f32::EPSILON);
    assert!((r.min.y - 20.0).abs() < f32::EPSILON);
    assert!((r.max.x - 50.0).abs() < f32::EPSILON);
    assert!((r.max.y - 80.0).abs() < f32::EPSILON);
}

#[test]
fn flatten_with_ids_returns_node_ids() {
    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("my_node").with_primitive(Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: Some("d".to_string()),
    }));
    let flat = graph.flatten_with_ids();
    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].2.as_str(), "my_node");
}
