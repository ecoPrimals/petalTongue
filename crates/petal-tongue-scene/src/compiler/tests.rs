// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::data_binding::DataBindingCompiler;
use crate::domain_palette::palette_for_domain;
use crate::grammar::{CoordinateSystem, FacetLayout, GeometryType, GrammarExpr};
use crate::math::Axes;
use crate::primitive::Primitive;
use crate::tufte::TufteConstraintImpl;

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
    let constraints = [TufteConstraintImpl::DataInkRatio];
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

// ──────────────────────────────────────────────────────────────────────────
// Tier 1: Math Validation Tests — Axes, Geometry, and Coordinate Ranges
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn axes_from_data_genomic_scale() {
    let points: Vec<[f64; 2]> = vec![[0.0, 0.0], [4_629_812.0, 100.0]];
    let axes = Axes::from_data(&points);
    assert!(
        axes.x_range.2 >= 200_000.0,
        "genomic x-step should be ≥200k, got {}",
        axes.x_range.2
    );
    assert!(
        axes.x_range.0 <= 0.0,
        "x_min should include 0, got {}",
        axes.x_range.0
    );
    assert!(
        axes.x_range.1 >= 4_600_000.0,
        "x_max should include ~4.6M, got {}",
        axes.x_range.1
    );
}

#[test]
fn axes_from_data_tir_scale() {
    let points: Vec<[f64; 2]> = vec![[42.0, 0.0], [72_891.0, 15.0]];
    let axes = Axes::from_data(&points);
    let x_range = axes.x_range.1 - axes.x_range.0;
    assert!(
        x_range > 0.0,
        "TIR-scale range should be non-degenerate: {x_range}"
    );
    assert!(
        axes.x_range.2 > 0.0,
        "x_step should be positive: {}",
        axes.x_range.2
    );
    assert!(
        axes.x_range.1 >= 72_891.0,
        "x_max should cover data, got {}",
        axes.x_range.1
    );
}

#[test]
fn axes_from_data_format_tick_m_suffix() {
    use crate::math::format_tick;
    let s = format_tick(2_500_000.0, 500_000.0);
    assert!(
        s.contains('M'),
        "genomic-scale ticks should use M suffix, got '{s}'"
    );
}

#[test]
fn axes_from_data_format_tick_k_suffix() {
    use crate::math::format_tick;
    let s = format_tick(50_000.0, 10_000.0);
    assert!(
        s.contains('k'),
        "kilo-scale ticks should use k suffix, got '{s}'"
    );
}

#[test]
fn axes_data_to_screen_no_nan() {
    let points: Vec<[f64; 2]> = vec![[0.0, 0.0], [1_000_000.0, 50.0], [4_500_000.0, 95.0]];
    let axes = Axes::from_data(&points);
    for &[x, y] in &points {
        let (sx, sy) = axes.data_to_screen(x, y);
        assert!(!sx.is_nan(), "screen x should not be NaN for data ({x}, {y})");
        assert!(!sy.is_nan(), "screen y should not be NaN for data ({x}, {y})");
        assert!(sx.is_finite(), "screen x should be finite for data ({x}, {y})");
        assert!(sy.is_finite(), "screen y should be finite for data ({x}, {y})");
    }
}

#[test]
fn axes_data_to_screen_within_canvas_bounds() {
    let points: Vec<[f64; 2]> = vec![[0.0, 0.0], [100.0, 50.0]];
    let axes = Axes::from_data(&points);
    let (ox, oy) = axes.origin;
    for &[x, y] in &points {
        let (sx, sy) = axes.data_to_screen(x, y);
        assert!(
            sx >= ox - 1.0,
            "screen x={sx} should be >= origin x={ox} for data ({x}, {y})"
        );
        assert!(
            sx <= ox + axes.width + 1.0,
            "screen x={sx} should be <= origin + width={} for data ({x}, {y})",
            ox + axes.width
        );
        assert!(
            sy <= oy + 1.0,
            "screen y={sy} should be <= origin y={oy} (y flipped) for data ({x}, {y})"
        );
        assert!(
            sy >= oy - axes.height - 1.0,
            "screen y={sy} should be >= origin - height={} for data ({x}, {y})",
            oy - axes.height
        );
    }
}

#[test]
fn compile_arc_polar_multi_feature_polygon_count() {
    let compiler = GrammarCompiler::new();
    let mut expr = GrammarExpr::new("data", GeometryType::Arc)
        .with_x("x")
        .with_y("y");
    expr.coordinate = CoordinateSystem::Polar;

    let data = vec![
        serde_json::json!({"x": 45.0, "y": 0.0, "value": 30.0, "label": "feat1"}),
        serde_json::json!({"x": 135.0, "y": 1.0, "value": 40.0, "label": "feat2"}),
        serde_json::json!({"x": 270.0, "y": 0.0, "value": 60.0, "label": "feat3"}),
    ];
    let graph = compiler.compile(&expr, &data);
    let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();

    let polygon_count = primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Polygon { .. }))
        .count();
    assert_eq!(
        polygon_count, 3,
        "3 polar arc features → 3 polygons, got {polygon_count}"
    );

    let arc_count = primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Arc { data_id: Some(id), .. } if id == "backbone"))
        .count();
    assert_eq!(arc_count, 1, "should have exactly 1 backbone arc");
}

#[test]
fn compile_tile_genome_track_rect_count() {
    let compiler = GrammarCompiler::new();
    let expr = GrammarExpr::new("data", GeometryType::Tile)
        .with_x("x")
        .with_y("y")
        .with_domain("genomics");
    let data = vec![
        serde_json::json!({"x": 0, "y": 0, "value": 1.0, "label": "IS1"}),
        serde_json::json!({"x": 1, "y": 0, "value": 0.8, "label": "IS2"}),
        serde_json::json!({"x": 2, "y": 1, "value": 0.6, "label": "SNP1"}),
        serde_json::json!({"x": 3, "y": 1, "value": 0.4, "label": "SNP2"}),
        serde_json::json!({"x": 4, "y": 2, "value": 0.9, "label": "DEL1"}),
    ];
    let graph = compiler.compile(&expr, &data);
    let primitives: Vec<_> = graph.flatten().into_iter().map(|(_, p)| p).collect();
    let rect_count = primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Rect { .. }))
        .count();
    assert!(
        rect_count >= 5,
        "5 tile segments → ≥5 rects, got {rect_count}"
    );
}

#[test]
fn genome_track_binding_compiles_to_tile() {
    use petal_tongue_types::DataBinding;
    let binding = DataBinding::GenomeTrack {
        id: "gt1".into(),
        label: "Test Track".into(),
        sequence_length: 4_629_812.0,
        tracks: vec!["SNP".into(), "IS".into()],
        segments: vec![
            serde_json::json!({"track": "SNP", "start": 100.0, "end": 200.0, "label": "snp1"}),
            serde_json::json!({"track": "IS", "start": 1000.0, "end": 2000.0, "label": "is1"}),
        ],
        unit: "bp".into(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, Some("genomics"));
    assert_eq!(expr.geometry, GeometryType::Tile);
    assert!(
        !data.is_empty(),
        "GenomeTrack should produce data rows"
    );

    let compiler = GrammarCompiler::new();
    let scene = compiler.compile(&expr, &data);
    assert!(
        scene.total_primitives() > 0,
        "GenomeTrack scene should have primitives"
    );
}

#[test]
fn circular_map_binding_compiles_to_arc_polar() {
    use petal_tongue_types::DataBinding;
    let binding = DataBinding::CircularMap {
        id: "cm1".into(),
        label: "pTest".into(),
        sequence_length: 8000.0,
        rings: vec!["features".into()],
        arcs: vec![
            serde_json::json!({"start_angle": 0.0, "end_angle": 90.0, "ring": 0, "label": "ori"}),
            serde_json::json!({"start_angle": 120.0, "end_angle": 200.0, "ring": 0, "label": "amp"}),
        ],
        unit: "bp".into(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, Some("genomics"));
    assert_eq!(expr.geometry, GeometryType::Arc);
    assert_eq!(expr.coordinate, CoordinateSystem::Polar);
    assert!(
        !data.is_empty(),
        "CircularMap should produce data rows"
    );

    let compiler = GrammarCompiler::new();
    let scene = compiler.compile(&expr, &data);
    let primitives: Vec<_> = scene.flatten().into_iter().map(|(_, p)| p).collect();
    let polygon_count = primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Polygon { .. }))
        .count();
    assert_eq!(
        polygon_count, 2,
        "2 arcs → 2 polygons, got {polygon_count}"
    );
}

#[test]
fn all_databinding_variants_produce_nonempty_scenes() {
    use petal_tongue_types::DataBinding;
    let bindings: Vec<DataBinding> = vec![
        DataBinding::TimeSeries {
            id: "ts".into(), label: "TS".into(),
            x_label: "t".into(), y_label: "v".into(), unit: "u".into(),
            x_values: vec![0.0, 1.0], y_values: vec![10.0, 20.0],
        },
        DataBinding::Bar {
            id: "b".into(), label: "Bar".into(),
            categories: vec!["A".into(), "B".into()],
            values: vec![10.0, 20.0], unit: "u".into(),
        },
        DataBinding::Gauge {
            id: "g".into(), label: "G".into(),
            value: 0.7, min: 0.0, max: 1.0, unit: "u".into(),
            normal_range: [0.2, 0.8], warning_range: [0.1, 0.9],
        },
        DataBinding::Heatmap {
            id: "h".into(), label: "H".into(),
            x_labels: vec!["a".into(), "b".into()],
            y_labels: vec!["r1".into()],
            values: vec![1.0, 2.0], unit: "u".into(),
        },
        DataBinding::Scatter {
            id: "sc".into(), label: "Sc".into(),
            x: vec![1.0, 2.0], y: vec![3.0, 4.0],
            point_labels: vec!["p1".into(), "p2".into()],
            x_label: "x".into(), y_label: "y".into(), unit: "u".into(),
        },
        DataBinding::Spectrum {
            id: "sp".into(), label: "Sp".into(),
            frequencies: vec![100.0, 200.0],
            amplitudes: vec![0.5, 0.8], unit: "dB".into(),
        },
        DataBinding::GenomeTrack {
            id: "gt".into(), label: "GT".into(),
            sequence_length: 5000.0,
            tracks: vec!["t1".into()],
            segments: vec![serde_json::json!({"track": "t1", "start": 0.0, "end": 1000.0})],
            unit: "bp".into(),
        },
        DataBinding::CircularMap {
            id: "cm".into(), label: "CM".into(),
            sequence_length: 3000.0,
            rings: vec!["r1".into()],
            arcs: vec![serde_json::json!({"start_angle": 0.0, "end_angle": 180.0, "ring": 0, "label": "feat"})],
            unit: "bp".into(),
        },
    ];

    let compiler = GrammarCompiler::new();
    for binding in &bindings {
        let (expr, data) = DataBindingCompiler::compile(binding, None);
        let scene = compiler.compile(&expr, &data);
        assert!(
            scene.total_primitives() > 0,
            "binding {:?} should produce a non-empty scene (got 0 primitives)",
            expr.data_source,
        );
    }
}
