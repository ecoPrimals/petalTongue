// SPDX-License-Identifier: AGPL-3.0-only
//! Bridge between the declarative scene engine (`petal-tongue-scene`) and egui rendering.
//!
//! Translates scene graph primitives into egui paint commands, connecting the
//! Grammar of Graphics pipeline to the live UI. Builds a `FrameHitMap` alongside
//! the paint commands so every rendered region can be traced back to its source
//! primitive, node, and data object.

use egui::{Color32, Painter, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use petal_tongue_scene::primitive::{Color, Primitive};
use petal_tongue_scene::render_plan::RenderPlan;
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::transform::Transform2D;

/// Provenance of a single rendered region — the answer to "what produced this pixel?"
#[derive(Debug, Clone)]
pub struct PixelProvenance {
    /// The scene-graph node that owns the primitive.
    pub node_id: String,
    /// Index of the primitive within the flattened scene output.
    pub primitive_index: usize,
    /// The `data_id` from the primitive, if any.
    pub data_id: Option<String>,
    /// World-space coordinates (pre-offset) of the primitive origin.
    pub world_x: f64,
    pub world_y: f64,
}

/// Per-frame spatial index mapping screen regions to their scene-graph source.
///
/// Built alongside `paint_scene_tracked` so every egui shape can be traced
/// back through the scene graph to the originating data.
#[derive(Debug, Clone, Default)]
pub struct FrameHitMap {
    entries: Vec<(Rect, PixelProvenance)>,
}

impl FrameHitMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a rendered region with its provenance.
    pub fn register(&mut self, screen_rect: Rect, provenance: PixelProvenance) {
        self.entries.push((screen_rect, provenance));
    }

    /// Query the hit map for the topmost entry at the given screen position.
    ///
    /// Returns the last (topmost) entry whose bounding rect contains the point,
    /// matching the painter's back-to-front draw order.
    #[must_use]
    pub fn query(&self, screen_x: f32, screen_y: f32) -> Option<&PixelProvenance> {
        let pos = Pos2::new(screen_x, screen_y);
        self.entries
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(pos))
            .map(|(_, prov)| prov)
    }

    /// Number of registered entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the hit map is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate all entries.
    pub fn iter(&self) -> impl Iterator<Item = &(Rect, PixelProvenance)> {
        self.entries.iter()
    }
}

/// Render a scene graph into an egui `Painter`, building a `FrameHitMap`.
pub fn paint_scene_tracked(painter: &Painter, scene: &SceneGraph, offset: Vec2) -> FrameHitMap {
    let mut hit_map = FrameHitMap::new();
    for (idx, (transform, prim, node_id)) in scene.flatten_with_ids().into_iter().enumerate() {
        let screen_rect = paint_primitive(painter, prim, &transform, offset);
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
    for (transform, prim) in scene.flatten() {
        paint_primitive(painter, prim, &transform, offset);
    }
}

/// Extract the logical origin point of a primitive (used for provenance).
fn primitive_origin(prim: &Primitive) -> (f64, f64) {
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

/// Render a single primitive with its world transform.
///
/// Returns the bounding `Rect` on screen for the rendered shape, or `None`
/// if the primitive was not drawn (e.g. degenerate or GPU-only).
#[expect(clippy::cast_possible_truncation, reason = "scene f64 to screen f32")]
pub fn paint_primitive(
    painter: &Painter,
    prim: &Primitive,
    transform: &Transform2D,
    offset: Vec2,
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
            let fill_c = fill.map(to_color32).unwrap_or(Color32::TRANSPARENT);
            let stroke_s = stroke.map(to_stroke).unwrap_or(Stroke::NONE);
            painter.circle(center, r, fill_c, stroke_s);
            Some(Rect::from_center_size(center, egui::vec2(r * 2.0, r * 2.0)))
        }

        Primitive::Line {
            points,
            stroke: s,
            closed,
            ..
        } => {
            let egui_stroke = to_stroke_style(s);
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
            let fill_c = fill.map(to_color32).unwrap_or(Color32::TRANSPARENT);
            let stroke_s = stroke.map(to_stroke).unwrap_or(Stroke::NONE);
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
            bold: _bold,
            ..
        } => {
            let (tx, ty) = transform.apply(*x, *y);
            let pos = Pos2::new(tx as f32 + offset.x, ty as f32 + offset.y);
            let font = egui::FontId::proportional(*font_size as f32);
            let galley = painter.text(
                pos,
                egui::Align2::LEFT_TOP,
                content,
                font,
                to_color32(*color),
            );
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
            let fill_c = to_color32(*fill);
            let stroke_s = stroke.map(to_stroke).unwrap_or(Stroke::NONE);
            if pts.len() >= 3 {
                let rect = bounding_rect(&pts);
                painter.add(egui::Shape::convex_polygon(pts, fill_c, stroke_s));
                Some(rect)
            } else {
                None
            }
        }

        Primitive::Arc { .. } | Primitive::BezierPath { .. } | Primitive::Mesh { .. } => None,
    }
}

/// Compute the axis-aligned bounding box of a set of screen points.
fn bounding_rect(pts: &[Pos2]) -> Rect {
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

/// Render a full `RenderPlan` into an egui `Painter`, returning the hit map.
pub fn paint_plan_tracked(painter: &Painter, plan: &RenderPlan, offset: Vec2) -> FrameHitMap {
    paint_scene_tracked(painter, &plan.scene, offset)
}

/// Legacy entry point — renders without tracking.
pub fn paint_plan(painter: &Painter, plan: &RenderPlan, offset: Vec2) {
    paint_scene(painter, &plan.scene, offset);
}

/// An egui widget that renders a `RenderPlan` via the scene bridge.
///
/// Drop this into any egui panel to display a grammar-compiled visualization:
/// ```ignore
/// SceneWidget::new(&plan).desired_size(vec2(400.0, 300.0)).show(ui);
/// ```
pub struct SceneWidget<'a> {
    plan: &'a RenderPlan,
    desired_size: Vec2,
    hit_map: Option<&'a mut FrameHitMap>,
}

impl<'a> SceneWidget<'a> {
    pub const fn new(plan: &'a RenderPlan) -> Self {
        Self {
            plan,
            desired_size: Vec2::new(400.0, 300.0),
            hit_map: None,
        }
    }

    #[must_use]
    pub const fn desired_size(mut self, size: Vec2) -> Self {
        self.desired_size = size;
        self
    }

    /// If provided, the widget populates this hit map during rendering.
    #[must_use]
    pub const fn with_hit_map(mut self, hit_map: &'a mut FrameHitMap) -> Self {
        self.hit_map = Some(hit_map);
        self
    }

    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(self.desired_size, Sense::hover());
        let offset = response.rect.min.to_vec2();
        if let Some(hit_map) = self.hit_map {
            *hit_map = paint_plan_tracked(&painter, self.plan, offset);
        } else {
            paint_plan(&painter, self.plan, offset);
        }
        response
    }
}

fn to_color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

fn to_stroke(s: petal_tongue_scene::primitive::StrokeStyle) -> Stroke {
    Stroke::new(s.width, to_color32(s.color))
}

fn to_stroke_style(s: &petal_tongue_scene::primitive::StrokeStyle) -> Stroke {
    Stroke::new(s.width, to_color32(s.color))
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};
    use petal_tongue_scene::primitive::StrokeStyle;
    use petal_tongue_scene::render_plan::RenderPlan;
    use petal_tongue_scene::scene_graph::SceneNode;

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
        let egui_s = to_stroke(s);
        assert!((egui_s.width - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stroke_style_conversion() {
        let s = StrokeStyle {
            color: Color::from_rgba8(128, 128, 128, 255),
            width: 3.0,
            ..StrokeStyle::default()
        };
        let egui_s = to_stroke_style(&s);
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
        // Widget created with default size 400x300
    }

    #[test]
    fn scene_widget_desired_size_builder() {
        let graph = SceneGraph::new();
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let plan = RenderPlan::new(graph, grammar);

        let _widget = SceneWidget::new(&plan).desired_size(egui::Vec2::new(800.0, 600.0));
        // Builder pattern applied
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
        // paint_plan just calls paint_scene - we verify the plan has the right structure
        assert!(!plan.scene.flatten().is_empty());
    }

    #[test]
    fn paint_primitive_point_with_stroke() {
        use petal_tongue_scene::primitive::StrokeStyle;

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
        let egui_c = Color32::from_rgba_unmultiplied(128, 64, 32, 200);
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 100.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 200.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 200.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 200.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 200.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 200.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 200.0), egui::Sense::hover());
                paint_primitive(&painter, &prim, &transform, response.rect.min.to_vec2());
            });
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

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(400.0, 300.0), egui::Sense::hover());
                paint_scene(&painter, &graph, response.rect.min.to_vec2());
            });
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
                let _response = SceneWidget::new(&plan)
                    .desired_size(egui::Vec2::new(400.0, 300.0))
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
        let rect = Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(50.0, 50.0));
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
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
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
            Rect::from_min_max(Pos2::new(20.0, 20.0), Pos2::new(80.0, 80.0)),
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
}
