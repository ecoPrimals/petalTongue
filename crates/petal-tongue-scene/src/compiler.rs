// SPDX-License-Identifier: AGPL-3.0-only
//! Grammar compiler: transforms GrammarExpr + data into a SceneGraph.
//!
//! The compiler reads variable bindings from the grammar expression to map
//! data fields to x/y coordinates, applies scales, and produces primitives
//! for the requested geometry type.

use serde_json::Value;

use crate::domain_palette::palette_for_domain;
use crate::grammar::{GeometryType, GrammarExpr, VariableRole};
use crate::math_objects::{Axes, MathObject};
use crate::primitive::{Color, LineCap, LineJoin, Primitive, StrokeStyle};
use crate::scene_graph::{SceneGraph, SceneNode};
use crate::tufte::{TufteConstraint, TufteReport};

/// Compiles GrammarExpr and data into a SceneGraph.
#[derive(Debug, Clone, Default)]
pub struct GrammarCompiler;

impl GrammarCompiler {
    /// Create a new grammar compiler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Get the x field name from variable bindings.
    fn x_field(expr: &GrammarExpr) -> Option<&str> {
        expr.variables
            .iter()
            .find(|v| v.role == VariableRole::X)
            .map(|v| v.field.as_str())
    }

    /// Get the y field name from variable bindings.
    fn y_field(expr: &GrammarExpr) -> Option<&str> {
        expr.variables
            .iter()
            .find(|v| v.role == VariableRole::Y)
            .map(|v| v.field.as_str())
    }

    /// Extract a numeric value from JSON for a given key.
    fn get_number(obj: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
        obj.get(key).and_then(|v| match v {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        })
    }

    /// Compile grammar expression and data into a scene graph.
    #[expect(
        clippy::too_many_lines,
        reason = "compile is a single cohesive match over grammar variants"
    )]
    pub fn compile(&self, expr: &GrammarExpr, data: &[Value]) -> SceneGraph {
        let mut graph = SceneGraph::new();

        let x_field = Self::x_field(expr);
        let y_field = Self::y_field(expr);

        // Resolve domain palette (capability-based, no hardcoded primal names)
        let palette = expr
            .domain
            .as_deref()
            .map_or_else(|| palette_for_domain("measurement"), palette_for_domain);
        let primary = palette.primary;
        let secondary = palette.secondary;

        let axes = Axes::default();
        let mut points: Vec<[f64; 2]> = Vec::new();
        for (i, obj) in data.iter().enumerate() {
            let obj = obj.as_object();
            let x = obj
                .and_then(|o| x_field.and_then(|f| Self::get_number(o, f)))
                .unwrap_or(i as f64);
            let y = obj
                .and_then(|o| y_field.and_then(|f| Self::get_number(o, f)))
                .unwrap_or(0.0);
            points.push([x, y]);
        }

        let stroke = StrokeStyle {
            color: primary,
            width: 1.5,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let primitives: Vec<Primitive> = match expr.geometry {
            GeometryType::Point => points
                .iter()
                .enumerate()
                .map(|(i, &[x, y])| {
                    let (sx, sy) = axes.data_to_screen(x, y);
                    Primitive::Point {
                        x: sx,
                        y: sy,
                        radius: 4.0,
                        fill: Some(primary),
                        stroke: None,
                        data_id: Some(format!("pt-{i}")),
                    }
                })
                .collect(),

            GeometryType::Bar => {
                let bar_width = if points.is_empty() {
                    0.0
                } else {
                    (axes.width / points.len() as f64).max(2.0) * 0.8
                };
                points
                    .iter()
                    .enumerate()
                    .map(|(i, &[x, y])| {
                        let (sx, sy) = axes.data_to_screen(x, y);
                        let (_, sy_base) = axes.data_to_screen(x, 0.0);
                        let height = (sy_base - sy).abs();
                        let bar_x = sx - bar_width / 2.0;
                        let bar_y = sy.min(sy_base);
                        Primitive::Rect {
                            x: bar_x,
                            y: bar_y,
                            width: bar_width,
                            height: height.max(1.0),
                            fill: Some(crate::domain_palette::categorical_color(palette, i)),
                            stroke: None,
                            corner_radius: 0.0,
                            data_id: Some(format!("bar-{i}")),
                        }
                    })
                    .collect()
            }

            GeometryType::Line => {
                if points.len() < 2 {
                    Vec::new()
                } else {
                    let screen_points: Vec<[f64; 2]> = points
                        .iter()
                        .map(|&[x, y]| axes.data_to_screen(x, y))
                        .map(|(sx, sy)| [sx, sy])
                        .collect();
                    vec![Primitive::Line {
                        points: screen_points,
                        stroke,
                        closed: false,
                        data_id: Some("line-0".to_string()),
                    }]
                }
            }

            GeometryType::Area => {
                // Area geometry: filled region under a line (ideal for Spectrum rendering)
                if points.len() < 2 {
                    Vec::new()
                } else {
                    let mut screen_points: Vec<[f64; 2]> = points
                        .iter()
                        .map(|&[x, y]| axes.data_to_screen(x, y))
                        .map(|(sx, sy)| [sx, sy])
                        .collect();

                    // Close the polygon by adding baseline points
                    let (_, baseline_y) = axes.data_to_screen(0.0, 0.0);
                    if let Some(last) = screen_points.last() {
                        screen_points.push([last[0], baseline_y]);
                    }
                    if let Some(first_x) = points.first().map(|p| p[0]) {
                        let (sx, _) = axes.data_to_screen(first_x, 0.0);
                        screen_points.push([sx, baseline_y]);
                    }

                    let fill_color = Color::rgba(primary.r, primary.g, primary.b, 0.3);
                    let mut prims = vec![Primitive::Polygon {
                        points: screen_points,
                        fill: fill_color,
                        stroke: None,
                        fill_rule: crate::primitive::FillRule::NonZero,
                        data_id: Some("area-fill".to_string()),
                    }];

                    // Add stroke line on top
                    let line_points: Vec<[f64; 2]> = points
                        .iter()
                        .map(|&[x, y]| axes.data_to_screen(x, y))
                        .map(|(sx, sy)| [sx, sy])
                        .collect();
                    prims.push(Primitive::Line {
                        points: line_points,
                        stroke,
                        closed: false,
                        data_id: Some("area-line".to_string()),
                    });
                    prims
                }
            }

            GeometryType::Ribbon => {
                // Ribbon: filled band between ymin and ymax (confidence intervals)
                let fill_color = Color::rgba(secondary.r, secondary.g, secondary.b, 0.2);
                vec![Primitive::Text {
                    x: axes.origin.0 + axes.width / 2.0,
                    y: axes.origin.1 - axes.height / 2.0,
                    content: "Ribbon (requires ymin/ymax roles)".to_string(),
                    font_size: 12.0,
                    color: fill_color,
                    anchor: crate::primitive::AnchorPoint::Center,
                    bold: false,
                    italic: false,
                    data_id: None,
                }]
            }

            _ => {
                vec![Primitive::Text {
                    x: axes.origin.0 + axes.width / 2.0,
                    y: axes.origin.1 - axes.height / 2.0,
                    content: format!("Geometry {:?} (placeholder)", expr.geometry),
                    font_size: 12.0,
                    color: Color::BLACK,
                    anchor: crate::primitive::AnchorPoint::Center,
                    bold: false,
                    italic: false,
                    data_id: None,
                }]
            }
        };

        // Add axes primitives
        let axes_prims = axes.to_primitives();
        let mut main_prims = primitives;
        main_prims.extend(axes_prims);

        let main_node = SceneNode::new("main").with_primitives(main_prims);
        graph.add_to_root(main_node);

        // Add title node if present
        if let Some(ref title) = expr.title {
            let title_node = SceneNode::new("title").with_primitive(Primitive::Text {
                x: axes.origin.0 + axes.width / 2.0,
                y: axes.origin.1 - axes.height - 20.0,
                content: title.clone(),
                font_size: 16.0,
                color: Color::BLACK,
                anchor: crate::primitive::AnchorPoint::BottomCenter,
                bold: true,
                italic: false,
                data_id: None,
            });
            graph.add_to_root(title_node);
        }

        graph
    }

    /// Compile grammar expression and data, then evaluate Tufte constraints.
    pub fn compile_with_constraints(
        &self,
        expr: &GrammarExpr,
        data: &[Value],
        constraints: &[&dyn TufteConstraint],
    ) -> (SceneGraph, TufteReport) {
        let graph = self.compile(expr, data);
        let primitives: Vec<Primitive> = graph
            .flatten()
            .into_iter()
            .map(|(_, p)| p.clone())
            .collect();
        let report = TufteReport::evaluate_all(constraints, &primitives, expr);
        (graph, report)
    }
}

impl SceneNode {
    /// Builder: add multiple primitives.
    fn with_primitives(mut self, primitives: Vec<Primitive>) -> Self {
        for p in primitives {
            self.primitives.push(p);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::GrammarExpr;
    use crate::tufte::DataInkRatio;

    fn point_expr() -> GrammarExpr {
        GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y")
    }

    #[test]
    fn compile_point_geometry_three_data_points() {
        let compiler = GrammarCompiler::new();
        let data = vec![
            serde_json::json!({"x": 1.0, "y": 2.0}),
            serde_json::json!({"x": 3.0, "y": 4.0}),
            serde_json::json!({"x": 5.0, "y": 6.0}),
        ];
        let graph = compiler.compile(&point_expr(), &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let point_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Point { .. }))
            .count();
        assert_eq!(point_count, 3);
    }

    #[test]
    fn compile_bar_geometry_rect_primitives() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Bar)
            .with_x("x")
            .with_y("y");
        let data = vec![
            serde_json::json!({"x": 0.0, "y": 10.0}),
            serde_json::json!({"x": 1.0, "y": 20.0}),
        ];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let rect_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Rect { .. }))
            .count();
        assert!(rect_count >= 2);
    }

    #[test]
    fn compile_with_title_adds_title_text_node() {
        let compiler = GrammarCompiler::new();
        let expr = point_expr().with_title("My Chart");
        let data = vec![serde_json::json!({"x": 0.0, "y": 0.0})];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let has_title = primitives
            .iter()
            .any(|p| matches!(p, Primitive::Text { content, .. } if content == "My Chart"));
        assert!(has_title);
    }

    #[test]
    fn compile_with_constraints_returns_report() {
        let compiler = GrammarCompiler::new();
        let data = vec![
            serde_json::json!({"x": 1.0, "y": 2.0}),
            serde_json::json!({"x": 3.0, "y": 4.0}),
        ];
        let constraints: Vec<&dyn TufteConstraint> = vec![&DataInkRatio];
        let (_graph, report) =
            compiler.compile_with_constraints(&point_expr(), &data, &constraints);
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].0, "DataInkRatio");
    }
}
