// SPDX-License-Identifier: AGPL-3.0-only
//! Grammar compiler: transforms GrammarExpr + data into a SceneGraph.
//!
//! The compiler reads variable bindings from the grammar expression to map
//! data fields to x/y coordinates, applies scales, and produces primitives
//! for the requested geometry type.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::domain_palette::palette_for_domain;
use crate::grammar::{FacetLayout, GeometryType, GrammarExpr, VariableRole};
use crate::math_objects::{Axes, MathObject};
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
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

            GeometryType::Tile => {
                // Tile: each data row becomes a filled rectangle at grid position
                // Expected data format: {"x": col, "y": row, "value": val}
                if points.is_empty() {
                    Vec::new()
                } else {
                    // Find value field for color mapping
                    let values: Vec<f64> = data
                        .iter()
                        .map(|obj| {
                            obj.as_object()
                                .and_then(|o| Self::get_number(o, "value"))
                                .unwrap_or(0.0)
                        })
                        .collect();
                    let val_min = values.iter().copied().fold(f64::INFINITY, f64::min);
                    let val_max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let val_range = (val_max - val_min).max(f64::EPSILON);

                    // Determine grid dimensions from unique x/y values
                    #[allow(clippy::cast_possible_truncation)]
                    let x_vals: std::collections::BTreeSet<i64> =
                        points.iter().map(|p| (p[0] * 1000.0) as i64).collect();
                    #[allow(clippy::cast_possible_truncation)]
                    let y_vals: std::collections::BTreeSet<i64> =
                        points.iter().map(|p| (p[1] * 1000.0) as i64).collect();
                    let cols = x_vals.len().max(1);
                    let rows = y_vals.len().max(1);
                    let tile_w = (axes.width / cols as f64).max(2.0);
                    let tile_h = (axes.height / rows as f64).max(2.0);

                    points
                        .iter()
                        .zip(values.iter())
                        .enumerate()
                        .map(|(i, (point, &val))| {
                            let [x, y] = *point;
                            let (sx, sy) = axes.data_to_screen(x, y);
                            // Check for threshold status field (injected by DataBindingCompiler)
                            let status = data.get(i).and_then(|d| {
                                d.as_object()
                                    .and_then(|o| o.get("status"))
                                    .and_then(|s| s.as_str())
                            });
                            let fill = if let Some(status) = status {
                                match status {
                                    "normal" => palette.normal,
                                    "warning" => palette.warning,
                                    "critical" => palette.critical,
                                    _ => {
                                        #[allow(clippy::cast_possible_truncation)]
                                        let t =
                                            ((val - val_min) / val_range).clamp(0.0, 1.0) as f32;
                                        Color::rgba(
                                            primary.r * t + palette.chart_bg.r * (1.0 - t),
                                            primary.g * t + palette.chart_bg.g * (1.0 - t),
                                            primary.b * t + palette.chart_bg.b * (1.0 - t),
                                            0.9,
                                        )
                                    }
                                }
                            } else {
                                #[allow(clippy::cast_possible_truncation)]
                                let t = ((val - val_min) / val_range).clamp(0.0, 1.0) as f32;
                                Color::rgba(
                                    primary.r * t + palette.chart_bg.r * (1.0 - t),
                                    primary.g * t + palette.chart_bg.g * (1.0 - t),
                                    primary.b * t + palette.chart_bg.b * (1.0 - t),
                                    0.9,
                                )
                            };
                            Primitive::Rect {
                                x: sx - tile_w / 2.0,
                                y: sy - tile_h / 2.0,
                                width: tile_w,
                                height: tile_h,
                                fill: Some(fill),
                                stroke: Some(StrokeStyle {
                                    color: Color::rgba(0.0, 0.0, 0.0, 0.1),
                                    width: 0.5,
                                    cap: LineCap::Butt,
                                    join: LineJoin::Miter,
                                }),
                                corner_radius: 0.0,
                                data_id: Some(format!("tile-{i}")),
                            }
                        })
                        .collect()
                }
            }

            GeometryType::Arc => {
                // Arc: semi-circular gauge
                // First data point y-value is the gauge value, x is ignored
                if let Some(&[_, value]) = points.first() {
                    let cx = axes.origin.0 + axes.width / 2.0;
                    let cy = axes.origin.1;
                    let radius = axes.width.min(axes.height) * 0.4;

                    // Background arc (full semicircle) - pie slice from center
                    let bg_color = Color::rgba(
                        palette.chart_bg.r,
                        palette.chart_bg.g,
                        palette.chart_bg.b,
                        0.5,
                    );
                    let mut prims = vec![Primitive::Arc {
                        cx,
                        cy,
                        radius,
                        start_angle: std::f64::consts::PI,
                        end_angle: 2.0 * std::f64::consts::PI,
                        fill: Some(bg_color),
                        stroke: None,
                        data_id: Some("gauge-bg".to_string()),
                    }];

                    // Filled arc proportional to value (0..1 mapped to 180..360 degrees)
                    let normalized = value.clamp(0.0, 1.0);
                    let sweep = std::f64::consts::PI * normalized;
                    prims.push(Primitive::Arc {
                        cx,
                        cy,
                        radius,
                        start_angle: std::f64::consts::PI,
                        end_angle: std::f64::consts::PI + sweep,
                        fill: Some(primary),
                        stroke: None,
                        data_id: Some("gauge-fill".to_string()),
                    });

                    // Value label
                    prims.push(Primitive::Text {
                        x: cx,
                        y: cy - radius * 0.15,
                        content: format!("{value:.1}"),
                        font_size: 18.0,
                        color: primary,
                        anchor: crate::primitive::AnchorPoint::Center,
                        bold: true,
                        italic: false,
                        data_id: None,
                    });

                    prims
                } else {
                    Vec::new()
                }
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

    /// Compile with faceting support (small multiples).
    ///
    /// When `expr.facets` is set, partitions data by the facet variable and
    /// compiles each group as a sub-scene. Groups are arranged according to
    /// the `FacetLayout` (Wrap or Grid).
    ///
    /// If no facets are configured, delegates to `compile`.
    pub fn compile_faceted(&self, expr: &GrammarExpr, data: &[Value]) -> SceneGraph {
        let Some(facet) = &expr.facets else {
            return self.compile(expr, data);
        };

        let columns = match &facet.layout {
            FacetLayout::Wrap { columns } => *columns,
            FacetLayout::Grid { .. } => 3,
        };
        let columns = columns.max(1);

        let groups = partition_by_field(data, &facet.variable);
        if groups.is_empty() {
            return self.compile(expr, data);
        }

        let mut graph = SceneGraph::new();
        let axes = Axes::default();
        let panel_w = axes.width + 40.0;
        let panel_h = axes.height + 60.0;

        for (idx, (key, group_data)) in groups.iter().enumerate() {
            let col = idx % columns;
            let row = idx / columns;
            let offset_x = col as f64 * panel_w;
            let offset_y = row as f64 * panel_h;

            let mut facet_expr = expr.clone();
            facet_expr.facets = None;
            facet_expr.title = Some(key.clone());

            let sub_scene = self.compile(&facet_expr, group_data);
            let flattened = sub_scene.flatten();

            let mut prims: Vec<Primitive> = Vec::new();
            for (_, prim) in &flattened {
                let mut p = (*prim).clone();
                offset_primitive(&mut p, offset_x, offset_y);
                prims.push(p);
            }

            let panel_node = SceneNode::new(format!("facet-{key}")).with_primitives(prims);
            graph.add_to_root(panel_node);

            let label_node =
                SceneNode::new(format!("facet-label-{key}")).with_primitive(Primitive::Text {
                    x: offset_x + panel_w / 2.0,
                    y: offset_y - 5.0,
                    content: key.clone(),
                    font_size: 12.0,
                    color: Color::rgba(0.3, 0.3, 0.3, 1.0),
                    anchor: AnchorPoint::BottomCenter,
                    bold: true,
                    italic: false,
                    data_id: None,
                });
            graph.add_to_root(label_node);
        }
        graph
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

/// Partition data rows by a JSON field value, preserving insertion order.
fn partition_by_field(data: &[Value], field: &str) -> Vec<(String, Vec<Value>)> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in data {
        let key = row.as_object().and_then(|o| o.get(field)).map_or_else(
            || "(none)".to_string(),
            |v| match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            },
        );
        groups.entry(key).or_default().push(row.clone());
    }
    groups.into_iter().collect()
}

/// Offset a primitive's position by (dx, dy).
fn offset_primitive(p: &mut Primitive, dx: f64, dy: f64) {
    match p {
        Primitive::Point { x, y, .. }
        | Primitive::Text { x, y, .. }
        | Primitive::Rect { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Primitive::Arc { cx, cy, .. } => {
            *cx += dx;
            *cy += dy;
        }
        Primitive::Line { points, .. } | Primitive::Polygon { points, .. } => {
            for pt in points {
                pt[0] += dx;
                pt[1] += dy;
            }
        }
        Primitive::BezierPath {
            start, segments, ..
        } => {
            start[0] += dx;
            start[1] += dy;
            for seg in segments {
                seg.cp1[0] += dx;
                seg.cp1[1] += dy;
                seg.cp2[0] += dx;
                seg.cp2[1] += dy;
                seg.end[0] += dx;
                seg.end[1] += dy;
            }
        }
        Primitive::Mesh { .. } => {}
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

    #[test]
    fn compile_tile_geometry_creates_rects() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Tile)
            .with_x("x")
            .with_y("y");
        let data = vec![
            serde_json::json!({"x": 0, "y": 0, "value": 1.0}),
            serde_json::json!({"x": 1, "y": 0, "value": 2.0}),
            serde_json::json!({"x": 0, "y": 1, "value": 3.0}),
            serde_json::json!({"x": 1, "y": 1, "value": 4.0}),
        ];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let rect_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Rect { .. }))
            .count();
        assert!(
            rect_count >= 4,
            "expected at least 4 tile rects, got {rect_count}"
        );
    }

    #[test]
    fn compile_faceted_wrap_creates_multiple_panels() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y")
            .with_facet("group", crate::grammar::FacetLayout::Wrap { columns: 2 });
        let data = vec![
            serde_json::json!({"x": 1.0, "y": 2.0, "group": "A"}),
            serde_json::json!({"x": 3.0, "y": 4.0, "group": "A"}),
            serde_json::json!({"x": 5.0, "y": 6.0, "group": "B"}),
            serde_json::json!({"x": 7.0, "y": 8.0, "group": "B"}),
            serde_json::json!({"x": 9.0, "y": 10.0, "group": "C"}),
        ];
        let graph = compiler.compile_faceted(&expr, &data);
        let all: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let point_count = all
            .iter()
            .filter(|p| matches!(p, Primitive::Point { .. }))
            .count();
        assert!(
            point_count >= 5,
            "all 5 data points should render across facets"
        );
        let label_count = all
            .iter()
            .filter(|p| {
                matches!(p, Primitive::Text { content, font_size, .. }
                if (*font_size - 12.0).abs() < 0.1 && ["A", "B", "C"].contains(&content.as_str()))
            })
            .count();
        assert!(
            label_count >= 3,
            "expected 3 facet labels, got {label_count}"
        );
    }

    #[test]
    fn compile_faceted_no_facets_delegates_to_compile() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y");
        let data = vec![serde_json::json!({"x": 1.0, "y": 2.0})];
        let g1 = compiler.compile(&expr, &data);
        let g2 = compiler.compile_faceted(&expr, &data);
        assert_eq!(g1.total_primitives(), g2.total_primitives());
    }

    #[test]
    fn tile_with_status_uses_palette_colors() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Tile)
            .with_x("x")
            .with_y("y")
            .with_domain("health");
        let data = vec![
            serde_json::json!({"x": 0, "y": 0, "value": 1.0, "status": "normal"}),
            serde_json::json!({"x": 1, "y": 0, "value": 2.0, "status": "warning"}),
            serde_json::json!({"x": 0, "y": 1, "value": 3.0, "status": "critical"}),
            serde_json::json!({"x": 1, "y": 1, "value": 4.0, "status": "unknown"}),
        ];
        let graph = compiler.compile(&expr, &data);
        let rects: Vec<_> = graph
            .flatten()
            .into_iter()
            .filter_map(|(_, p)| match p {
                Primitive::Rect { fill, .. } => *fill,
                _ => None,
            })
            .collect();
        assert!(rects.len() >= 4);
        let health = crate::domain_palette::palette_for_domain("health");
        assert!((rects[0].r - health.normal.r).abs() < 0.01);
        assert!((rects[1].r - health.warning.r).abs() < 0.01);
        assert!((rects[2].r - health.critical.r).abs() < 0.01);
    }

    #[test]
    fn compile_arc_geometry_creates_arcs() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Arc)
            .with_x("x")
            .with_y("y");
        let data = vec![serde_json::json!({"x": 0, "y": 0.7})];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let arc_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Arc { .. }))
            .count();
        assert!(
            arc_count >= 2,
            "expected background + fill arcs, got {arc_count}"
        );
    }
}
