// SPDX-License-Identifier: AGPL-3.0-only
//! Determinism tests for the scene compiler.
//!
//! Compiles the same `GrammarExpr` + data multiple times and asserts
//! byte-identical output. Uses flattened primitives (depth-first order)
//! for serialization since `SceneGraph`'s `HashMap` has non-deterministic key order.

use petal_tongue_scene::{
    GrammarCompiler,
    grammar::{GeometryType, GrammarExpr},
    modality::{AudioCompiler, ModalityCompiler, ModalityOutput, SvgCompiler, TerminalCompiler},
    tufte::{DataInkRatio, LieFactor, TufteConstraint},
};
use serde_json::Value;

const NUM_RUNS: usize = 100;

/// Serialize scene graph in deterministic order (flattened depth-first).
fn graph_to_deterministic_json(graph: &petal_tongue_scene::SceneGraph) -> String {
    let flat: Vec<_> = graph
        .flatten()
        .into_iter()
        .map(|(t, p)| (t, p.clone()))
        .collect();
    serde_json::to_string(&flat).unwrap()
}

fn point_expr() -> GrammarExpr {
    GrammarExpr::new("data", GeometryType::Point)
        .with_x("x")
        .with_y("y")
}

fn line_expr() -> GrammarExpr {
    GrammarExpr::new("data", GeometryType::Line)
        .with_x("x")
        .with_y("y")
}

fn bar_expr() -> GrammarExpr {
    GrammarExpr::new("data", GeometryType::Bar)
        .with_x("x")
        .with_y("y")
}

fn area_expr() -> GrammarExpr {
    GrammarExpr::new("data", GeometryType::Area)
        .with_x("x")
        .with_y("y")
}

fn sample_data() -> Vec<Value> {
    vec![
        serde_json::json!({"x": 1.0, "y": 2.0}),
        serde_json::json!({"x": 3.0, "y": 4.0}),
        serde_json::json!({"x": 5.0, "y": 6.0}),
    ]
}

#[test]
fn determinism_point_geometry_100_runs() {
    let compiler = GrammarCompiler::new();
    let expr = point_expr();
    let data = sample_data();

    let first = graph_to_deterministic_json(&compiler.compile(&expr, &data));
    for _ in 0..(NUM_RUNS - 1) {
        let run = graph_to_deterministic_json(&compiler.compile(&expr, &data));
        assert_eq!(
            first, run,
            "Point geometry must produce byte-identical output"
        );
    }
}

#[test]
fn determinism_line_geometry_100_runs() {
    let compiler = GrammarCompiler::new();
    let expr = line_expr();
    let data = sample_data();

    let first = graph_to_deterministic_json(&compiler.compile(&expr, &data));
    for _ in 0..(NUM_RUNS - 1) {
        let run = graph_to_deterministic_json(&compiler.compile(&expr, &data));
        assert_eq!(
            first, run,
            "Line geometry must produce byte-identical output"
        );
    }
}

#[test]
fn determinism_bar_geometry_100_runs() {
    let compiler = GrammarCompiler::new();
    let expr = bar_expr();
    let data = sample_data();

    let first = graph_to_deterministic_json(&compiler.compile(&expr, &data));
    for _ in 0..(NUM_RUNS - 1) {
        let run = graph_to_deterministic_json(&compiler.compile(&expr, &data));
        assert_eq!(
            first, run,
            "Bar geometry must produce byte-identical output"
        );
    }
}

#[test]
fn determinism_area_geometry_100_runs() {
    let compiler = GrammarCompiler::new();
    let expr = area_expr();
    let data = sample_data();

    let first = graph_to_deterministic_json(&compiler.compile(&expr, &data));
    for _ in 0..(NUM_RUNS - 1) {
        let run = graph_to_deterministic_json(&compiler.compile(&expr, &data));
        assert_eq!(
            first, run,
            "Area geometry must produce byte-identical output"
        );
    }
}

#[test]
fn determinism_with_tufte_constraints_100_runs() {
    let compiler = GrammarCompiler::new();
    let expr = point_expr();
    let data = sample_data();
    let constraints: Vec<&dyn TufteConstraint> = vec![&DataInkRatio, &LieFactor];

    let (first_graph, first_report) = compiler.compile_with_constraints(&expr, &data, &constraints);
    let first_json = graph_to_deterministic_json(&first_graph);
    let first_report_json = serde_json::to_string(&first_report).unwrap();

    for _ in 0..(NUM_RUNS - 1) {
        let (graph, report) = compiler.compile_with_constraints(&expr, &data, &constraints);
        let run_json = graph_to_deterministic_json(&graph);
        let report_json = serde_json::to_string(&report).unwrap();
        assert_eq!(
            first_json, run_json,
            "SceneGraph with constraints must be deterministic"
        );
        assert_eq!(
            first_report_json, report_json,
            "TufteReport must be deterministic"
        );
    }
}

#[test]
fn determinism_without_tufte_constraints_100_runs() {
    let compiler = GrammarCompiler::new();
    let expr = bar_expr()
        .with_title("Test Chart")
        .with_domain("measurement");
    let data = sample_data();

    let first = graph_to_deterministic_json(&compiler.compile(&expr, &data));
    for _ in 0..(NUM_RUNS - 1) {
        let run = graph_to_deterministic_json(&compiler.compile(&expr, &data));
        assert_eq!(
            first, run,
            "Compile without constraints must be deterministic"
        );
    }
}

// -----------------------------------------------------------------------------
// Modality compiler round-trip determinism tests
// -----------------------------------------------------------------------------

#[test]
fn modality_grammar_to_scene_to_svg_determinism() {
    let grammar_compiler = GrammarCompiler::new();
    let svg_compiler = SvgCompiler::new();
    let expr = point_expr();
    let data = sample_data();

    let scene = grammar_compiler.compile(&expr, &data);
    let ModalityOutput::Svg(first) = svg_compiler.compile(&scene) else {
        panic!("expected Svg");
    };

    for _ in 0..(NUM_RUNS - 1) {
        let scene = grammar_compiler.compile(&expr, &data);
        let ModalityOutput::Svg(run) = svg_compiler.compile(&scene) else {
            panic!("expected Svg");
        };
        assert_eq!(first, run, "Grammar -> Scene -> SVG must be deterministic");
    }
}

#[test]
fn modality_svg_structure_has_expected_elements() {
    let grammar_compiler = GrammarCompiler::new();
    let svg_compiler = SvgCompiler::new();
    let expr = line_expr();
    let data = sample_data();

    let scene = grammar_compiler.compile(&expr, &data);
    let ModalityOutput::Svg(b) = svg_compiler.compile(&scene) else {
        panic!("expected Svg");
    };
    let svg = std::str::from_utf8(b.as_ref()).unwrap();

    assert!(svg.contains("<svg"), "SVG must have root element");
    assert!(svg.contains("</svg>"), "SVG must close root");
    assert!(
        svg.contains("polyline") || svg.contains("circle") || svg.contains("rect"),
        "SVG must contain geometry elements"
    );
}

#[test]
fn modality_grammar_to_scene_to_audio_params_determinism() {
    let grammar_compiler = GrammarCompiler::new();
    let audio_compiler = AudioCompiler::new();
    let expr = point_expr();
    let data = sample_data();

    let scene = grammar_compiler.compile(&expr, &data);
    let ModalityOutput::AudioParams(p) = audio_compiler.compile(&scene) else {
        panic!("expected AudioParams");
    };
    let first = serde_json::to_string(&p).unwrap();

    for _ in 0..(NUM_RUNS - 1) {
        let scene = grammar_compiler.compile(&expr, &data);
        let ModalityOutput::AudioParams(p) = audio_compiler.compile(&scene) else {
            panic!("expected AudioParams");
        };
        let run = serde_json::to_string(&p).unwrap();
        assert_eq!(
            first, run,
            "Grammar -> Scene -> AudioParams must be deterministic"
        );
    }
}

#[test]
fn modality_grammar_to_scene_to_terminal_grid_determinism() {
    let grammar_compiler = GrammarCompiler::new();
    let terminal_compiler = TerminalCompiler::new(80, 24);
    let expr = point_expr();
    let data = sample_data();

    let scene = grammar_compiler.compile(&expr, &data);
    let ModalityOutput::TerminalCells(g) = terminal_compiler.compile(&scene) else {
        panic!("expected TerminalCells");
    };
    let first = g
        .iter()
        .map(|row| row.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    for _ in 0..(NUM_RUNS - 1) {
        let scene = grammar_compiler.compile(&expr, &data);
        let ModalityOutput::TerminalCells(g) = terminal_compiler.compile(&scene) else {
            panic!("expected TerminalCells");
        };
        let run = g
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            first, run,
            "Grammar -> Scene -> TerminalCells must be deterministic"
        );
    }
}
