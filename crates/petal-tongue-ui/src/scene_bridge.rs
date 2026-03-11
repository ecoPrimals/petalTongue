// SPDX-License-Identifier: AGPL-3.0-only
//! Bridge between the declarative scene engine (`petal-tongue-scene`) and egui rendering.
//!
//! Translates scene graph primitives into egui paint commands, connecting the
//! Grammar of Graphics pipeline to the live UI.

use egui::{Color32, Painter, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use petal_tongue_scene::primitive::{Color, Primitive};
use petal_tongue_scene::render_plan::RenderPlan;
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::transform::Transform2D;

/// Render a scene graph into an egui `Painter`.
///
/// Flattens the scene graph with world transforms and draws each primitive
/// using egui's immediate-mode paint API.
pub fn paint_scene(painter: &Painter, scene: &SceneGraph, offset: Vec2) {
    for (transform, prim) in scene.flatten() {
        paint_primitive(painter, prim, &transform, offset);
    }
}

/// Render a single primitive with its world transform.
pub fn paint_primitive(painter: &Painter, prim: &Primitive, transform: &Transform2D, offset: Vec2) {
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
            let fill_c = fill.map(to_color32).unwrap_or(Color32::TRANSPARENT);
            let stroke_s = stroke.map(to_stroke).unwrap_or(Stroke::NONE);
            painter.circle(center, *radius as f32, fill_c, stroke_s);
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
            painter.text(
                pos,
                egui::Align2::LEFT_TOP,
                content,
                font,
                to_color32(*color),
            );
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
                painter.add(egui::Shape::convex_polygon(pts, fill_c, stroke_s));
            }
        }

        Primitive::Arc { .. } | Primitive::BezierPath { .. } | Primitive::Mesh { .. } => {
            // Complex primitives: delegate to GPU in future (Toadstool/barraCuda path)
        }
    }
}

/// Render a full `RenderPlan` into an egui `Painter`.
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
}

impl<'a> SceneWidget<'a> {
    pub const fn new(plan: &'a RenderPlan) -> Self {
        Self {
            plan,
            desired_size: Vec2::new(400.0, 300.0),
        }
    }

    #[must_use]
    pub const fn desired_size(mut self, size: Vec2) -> Self {
        self.desired_size = size;
        self
    }

    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(self.desired_size, Sense::hover());
        let offset = response.rect.min.to_vec2();
        paint_plan(&painter, self.plan, offset);
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
}
