// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::domain_palette::palette_for_domain;
use crate::grammar::{FacetLayout, GeometryType, GrammarExpr};
use crate::primitive::Primitive;
use crate::tufte::{DataInkRatio, TufteConstraint};

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
    let (_graph, report) = compiler.compile_with_constraints(&point_expr(), &data, &constraints);
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
