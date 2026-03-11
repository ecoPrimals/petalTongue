// SPDX-License-Identifier: AGPL-3.0-only
//! Grammar compiler: transforms `GrammarExpr` + data into a `SceneGraph`.
//!
//! The compiler reads variable bindings from the grammar expression to map
//! data fields to x/y coordinates, applies scales, and produces primitives
//! for the requested geometry type.

mod geometry;
mod utils;

use serde_json::Value;

use crate::domain_palette::palette_for_domain;
use crate::grammar::{FacetLayout, GrammarExpr, ScaleType};
use crate::math::{Axes, MathObject};
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
use crate::render_plan::{AxisMeta, PanelBounds, PanelMeta, RenderPlan};
use crate::scene_graph::{SceneGraph, SceneNode};
use crate::tufte::{TufteConstraint, TufteReport};

use geometry::compile_geometry;
use utils::{offset_primitive, partition_by_field, x_field, y_field};

/// Compiles `GrammarExpr` and data into a `SceneGraph`.
#[derive(Debug, Clone, Default)]
pub struct GrammarCompiler;

impl GrammarCompiler {
    /// Create a new grammar compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Compile grammar expression and data into a scene graph.
    pub fn compile(&self, expr: &GrammarExpr, data: &[Value]) -> SceneGraph {
        let mut graph = SceneGraph::new();

        let x_field = x_field(expr);
        let y_field = y_field(expr);

        let palette = expr
            .domain
            .as_deref()
            .map_or_else(|| palette_for_domain("measurement"), palette_for_domain);

        let axes = Axes::default();
        let mut points: Vec<[f64; 2]> = Vec::new();
        for (i, obj) in data.iter().enumerate() {
            let obj = obj.as_object();
            let x = obj
                .and_then(|o| x_field.and_then(|f| utils::get_number(o, f)))
                .unwrap_or(i as f64);
            let y = obj
                .and_then(|o| y_field.and_then(|f| utils::get_number(o, f)))
                .unwrap_or(0.0);
            points.push([x, y]);
        }

        let stroke = StrokeStyle {
            color: palette.primary,
            width: 1.5,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let primitives = compile_geometry(expr, data, &points, &axes, palette, &stroke);

        let axes_prims = axes.to_primitives();
        let mut main_prims = primitives;
        main_prims.extend(axes_prims);

        let main_node = SceneNode::new("main").with_primitives(main_prims);
        graph.add_to_root(main_node);

        if let Some(ref title) = expr.title {
            let title_node = SceneNode::new("title").with_primitive(Primitive::Text {
                x: axes.origin.0 + axes.width / 2.0,
                y: axes.origin.1 - axes.height - 20.0,
                content: title.clone(),
                font_size: 16.0,
                color: Color::BLACK,
                anchor: AnchorPoint::BottomCenter,
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
        let report = TufteReport::evaluate_all(constraints, &primitives, expr, Some(data));
        (graph, report)
    }

    /// Compile grammar expression and data into a `RenderPlan`.
    ///
    /// The render plan wraps the scene graph with panel bounds, axis metadata,
    /// and optional Tufte constraint results. Modality compilers and the
    /// interaction engine consume the plan for inverse transforms.
    pub fn compile_plan(
        &self,
        expr: &GrammarExpr,
        data: &[Value],
        constraints: &[&dyn TufteConstraint],
    ) -> RenderPlan {
        let graph = self.compile(expr, data);
        let axes = Axes::default();

        let x_scale = expr
            .scales
            .iter()
            .find(|s| s.variable == "x")
            .map_or(ScaleType::Linear, |s| s.scale_type);
        let y_scale = expr
            .scales
            .iter()
            .find(|s| s.variable == "y")
            .map_or(ScaleType::Linear, |s| s.scale_type);

        let (x_min, x_max) = Self::data_extent(data, x_field(expr));
        let (y_min, y_max) = Self::data_extent(data, y_field(expr));

        let panel = PanelMeta::new(
            "main",
            PanelBounds::new(
                axes.origin.0,
                axes.origin.1 - axes.height,
                axes.width,
                axes.height,
            ),
        )
        .with_axis(
            AxisMeta::new("x", x_scale)
                .with_domain(x_min, x_max)
                .with_range(axes.origin.0, axes.origin.0 + axes.width),
        )
        .with_axis(
            AxisMeta::new("y", y_scale)
                .with_domain(y_min, y_max)
                .with_range(axes.origin.1, axes.origin.1 - axes.height),
        );

        let mut plan = RenderPlan::new(graph, expr.clone()).with_panel(panel);

        if !constraints.is_empty() {
            let primitives: Vec<Primitive> = plan
                .scene
                .flatten()
                .into_iter()
                .map(|(_, p)| p.clone())
                .collect();
            let report = TufteReport::evaluate_all(constraints, &primitives, expr, Some(data));
            plan = plan.with_constraints_report(report);
        }

        plan
    }

    fn data_extent(data: &[Value], field: Option<&str>) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for (i, obj) in data.iter().enumerate() {
            let v = obj
                .as_object()
                .and_then(|o| field.and_then(|f| utils::get_number(o, f)))
                .unwrap_or(i as f64);
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        if min.is_infinite() {
            (0.0, 1.0)
        } else {
            (min, max)
        }
    }

    /// Compile with faceting support (small multiples).
    ///
    /// When `expr.facets` is set, partitions data by the facet variable and
    /// compiles each group as a sub-scene. Groups are arranged according to
    /// the `FacetLayout` (Wrap or Grid).
    ///
    /// If no facets are configured, delegates to `compile`.
    #[must_use]
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

            let key_str = key.as_ref();
            let mut facet_expr = expr.clone();
            facet_expr.facets = None;
            facet_expr.title = Some(key_str.to_string());

            let sub_scene = self.compile(&facet_expr, group_data);
            let flattened = sub_scene.flatten();

            let mut prims: Vec<Primitive> = Vec::new();
            for (_, prim) in &flattened {
                let mut p = (*prim).clone();
                offset_primitive(&mut p, offset_x, offset_y);
                prims.push(p);
            }

            let panel_node = SceneNode::new(format!("facet-{key_str}")).with_primitives(prims);
            graph.add_to_root(panel_node);

            let label_node =
                SceneNode::new(format!("facet-label-{key_str}")).with_primitive(Primitive::Text {
                    x: offset_x + panel_w / 2.0,
                    y: offset_y - 5.0,
                    content: key_str.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{FacetLayout, GeometryType, GrammarExpr};
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
            .with_facet("group", FacetLayout::Wrap { columns: 2 });
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
        let health = palette_for_domain("health");
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

    #[test]
    fn compile_line_with_one_point_produces_no_data_line() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Line)
            .with_x("x")
            .with_y("y");
        let data = vec![serde_json::json!({"x": 1.0, "y": 2.0})];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        // Data line has data_id "line-0"; axes produce many Line primitives without that id
        let data_line_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Line { data_id: Some(id), .. } if id == "line-0"))
            .count();
        assert_eq!(
            data_line_count, 0,
            "Line with 1 point should produce no data Line"
        );
    }

    #[test]
    fn compile_line_with_two_points_produces_data_line() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Line)
            .with_x("x")
            .with_y("y");
        let data = vec![
            serde_json::json!({"x": 1.0, "y": 2.0}),
            serde_json::json!({"x": 3.0, "y": 4.0}),
        ];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let data_line_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Line { data_id: Some(id), .. } if id == "line-0"))
            .count();
        assert_eq!(data_line_count, 1);
    }

    #[test]
    fn compile_area_geometry_creates_polygon_and_line() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Area)
            .with_x("x")
            .with_y("y");
        let data = vec![
            serde_json::json!({"x": 0.0, "y": 10.0}),
            serde_json::json!({"x": 1.0, "y": 20.0}),
            serde_json::json!({"x": 2.0, "y": 15.0}),
        ];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let polygon_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Polygon { .. }))
            .count();
        let line_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Line { .. }))
            .count();
        assert!(polygon_count >= 1);
        assert!(line_count >= 1);
    }

    #[test]
    fn compile_ribbon_geometry_produces_placeholder() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Ribbon)
            .with_x("x")
            .with_y("y");
        let data = vec![
            serde_json::json!({"x": 0.0, "y": 10.0}),
            serde_json::json!({"x": 1.0, "y": 20.0}),
        ];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let text_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Text { content, .. } if content.contains("Ribbon")))
            .count();
        assert!(text_count >= 1);
    }

    #[test]
    fn compile_uses_index_when_x_field_missing() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Point).with_y("y");
        let data = vec![serde_json::json!({"y": 1.0}), serde_json::json!({"y": 2.0})];
        let graph = compiler.compile(&expr, &data);
        let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let point_count = primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Point { .. }))
            .count();
        assert_eq!(point_count, 2);
    }

    #[test]
    fn compile_faceted_grid_layout_uses_columns() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y")
            .with_facet(
                "group",
                FacetLayout::Grid {
                    rows: "r".to_string(),
                    cols: "c".to_string(),
                },
            );
        let data = vec![
            serde_json::json!({"x": 1.0, "y": 2.0, "group": "A"}),
            serde_json::json!({"x": 3.0, "y": 4.0, "group": "B"}),
        ];
        let graph = compiler.compile_faceted(&expr, &data);
        let all: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
        let point_count = all
            .iter()
            .filter(|p| matches!(p, Primitive::Point { .. }))
            .count();
        assert!(point_count >= 2);
    }

    #[test]
    fn compile_faceted_empty_groups_delegates_to_compile() {
        let compiler = GrammarCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y")
            .with_facet("group", FacetLayout::Wrap { columns: 2 });
        let data: Vec<serde_json::Value> = vec![];
        let graph = compiler.compile_faceted(&expr, &data);
        assert!(graph.total_primitives() > 0);
    }
}
