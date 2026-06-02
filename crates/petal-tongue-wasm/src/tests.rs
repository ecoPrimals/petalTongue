// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for WASM rendering entry points.

use crate::{
    compile_scene, render_binding, render_binding_to_modality, render_binding_with_thresholds,
    render_bindings, render_dashboard, render_grammar, render_grammar_to_modality, render_scene,
    render_scene_to_modality, validate_grammar, version,
};

const GRAMMAR: &str = r#"{
    "data_source": "test",
    "variables": [
        {"name": "x", "field": "time", "role": "X"},
        {"name": "y", "field": "value", "role": "Y"}
    ],
    "scales": [
        {"variable": "x", "scale_type": "Linear"},
        {"variable": "y", "scale_type": "Linear"}
    ],
    "geometry": "Point",
    "coordinate": "Cartesian",
    "aesthetics": []
}"#;

const DATA: &str = r#"[
    {"time": 0.0, "value": 1.0},
    {"time": 1.0, "value": 2.0},
    {"time": 2.0, "value": 3.0}
]"#;

fn assert_svg(result: &str) {
    assert!(
        result.starts_with("<svg"),
        "expected SVG output, got: {result}"
    );
    assert!(result.contains("</svg>"));
}

fn assert_no_error(result: &str) {
    assert!(!result.starts_with("Error:"), "unexpected error: {result}");
}

// ── Grammar tests ───────────────────────────────────────────────

#[test]
fn render_grammar_produces_svg() {
    assert_svg(&render_grammar(GRAMMAR, DATA));
}

#[test]
fn render_grammar_to_modality_svg() {
    assert_svg(&render_grammar_to_modality(GRAMMAR, DATA, "svg"));
}

#[test]
fn render_grammar_to_modality_description() {
    assert_no_error(&render_grammar_to_modality(GRAMMAR, DATA, "description"));
}

#[test]
fn render_grammar_to_modality_terminal() {
    let result = render_grammar_to_modality(GRAMMAR, DATA, "terminal");
    assert_no_error(&result);
}

#[test]
fn render_grammar_invalid_json_returns_error() {
    assert!(render_grammar("{bad", "[]").starts_with("Error:"));
}

// ── Scene graph tests ───────────────────────────────────────────

#[test]
fn compile_scene_returns_json() {
    let json = compile_scene(GRAMMAR, DATA);
    assert_no_error(&json);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn render_scene_roundtrip() {
    let scene_json = compile_scene(GRAMMAR, DATA);
    assert_no_error(&scene_json);
    let svg = render_scene(&scene_json);
    assert_svg(&svg);
}

#[test]
fn render_scene_to_modality_description() {
    let scene_json = compile_scene(GRAMMAR, DATA);
    let desc = render_scene_to_modality(&scene_json, "description");
    assert_no_error(&desc);
}

#[test]
fn render_scene_invalid_json() {
    assert!(render_scene("{bad}").starts_with("Error:"));
}

// ── Tufte validation ────────────────────────────────────────────

#[test]
fn validate_grammar_returns_report() {
    let report = validate_grammar(GRAMMAR, DATA);
    assert_no_error(&report);
    let parsed: serde_json::Value = serde_json::from_str(&report).expect("valid JSON");
    assert!(parsed.get("results").is_some() || parsed.get("overall_score").is_some());
}

// ── DataBinding tests (all variants) ────────────────────────────

#[test]
fn render_binding_timeseries() {
    let binding = r#"{
        "channel_type": "timeseries",
        "id": "ts1", "label": "Series",
        "x_label": "Time", "y_label": "Value", "unit": "u",
        "x_values": [0.0, 1.0, 2.0],
        "y_values": [1.0, 4.0, 2.0]
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_distribution() {
    let binding = r#"{
        "channel_type": "distribution",
        "id": "dist1", "label": "Heights", "unit": "cm",
        "values": [170.0, 175.0, 180.0, 165.0, 172.0],
        "mean": 172.4, "std": 5.5, "comparison_value": 178.0
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_bar() {
    let binding = r#"{
        "channel_type": "bar",
        "id": "bar1", "label": "Sales", "unit": "$",
        "categories": ["A", "B", "C"],
        "values": [100.0, 200.0, 150.0]
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_gauge() {
    let binding = r#"{
        "channel_type": "gauge",
        "id": "g1", "label": "CPU", "unit": "%",
        "value": 72.5, "min": 0.0, "max": 100.0,
        "normal_range": [0.0, 70.0],
        "warning_range": [70.0, 90.0]
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_spectrum() {
    let binding = r#"{
        "channel_type": "spectrum",
        "id": "sp1", "label": "Audio", "unit": "dB",
        "frequencies": [100.0, 200.0, 400.0, 800.0],
        "amplitudes": [0.5, 0.8, 0.3, 0.1]
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_heatmap() {
    let binding = r#"{
        "channel_type": "heatmap",
        "id": "hm1", "label": "Temp", "unit": "C",
        "x_labels": ["A", "B"],
        "y_labels": ["1", "2"],
        "values": [1.0, 2.0, 3.0, 4.0]
    }"#;
    assert_svg(&render_binding(binding, "health"));
}

#[test]
fn render_binding_scatter() {
    let binding = r#"{
        "channel_type": "scatter",
        "id": "sc1", "label": "Points", "unit": "",
        "x": [1.0, 2.0, 3.0],
        "y": [4.0, 5.0, 6.0]
    }"#;
    assert_svg(&render_binding(binding, "ecology"));
}

#[test]
fn render_binding_scatter3d() {
    let binding = r#"{
        "channel_type": "scatter3d",
        "id": "s3d1", "label": "3D", "unit": "",
        "x": [1.0, 2.0],
        "y": [3.0, 4.0],
        "z": [5.0, 6.0]
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_fieldmap() {
    let binding = r#"{
        "channel_type": "fieldmap",
        "id": "fm1", "label": "Field", "unit": "",
        "grid_x": [0.0, 1.0],
        "grid_y": [0.0, 1.0],
        "values": [0.5, 0.8, 0.2, 0.9]
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_game_scene() {
    let binding = r#"{
        "channel_type": "game_scene",
        "id": "gs1", "label": "Level 1",
        "scene": {"width": 10, "height": 10, "tiles": []}
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_soundscape() {
    let binding = r#"{
        "channel_type": "soundscape",
        "id": "ss1", "label": "Forest",
        "definition": {"layers": [], "duration": 60.0}
    }"#;
    assert_svg(&render_binding(binding, ""));
}

#[test]
fn render_binding_genome_track() {
    let binding = r#"{
        "channel_type": "genome_track",
        "id": "gt1", "label": "E. coli",
        "sequence_length": 4629812.0,
        "tracks": ["SNP", "IS insertion"],
        "segments": [
            {"track": "SNP", "start": 2450, "end": 2451, "name": "thrA"}
        ],
        "unit": "count"
    }"#;
    assert_svg(&render_binding(binding, "genomics"));
}

#[test]
fn render_binding_circular_map() {
    let binding = r#"{
        "channel_type": "circular_map",
        "id": "cm1", "label": "Plasmid",
        "sequence_length": 2686.0,
        "rings": ["Forward", "Reverse"],
        "arcs": [
            {"ring": 0, "start_angle": 53.0, "end_angle": 60.5, "category": "CDS"}
        ],
        "unit": "degrees"
    }"#;
    assert_svg(&render_binding(binding, "genomics"));
}

#[test]
fn render_binding_to_modality_description() {
    let binding = r#"{
        "channel_type": "timeseries",
        "id": "ts2", "label": "Test",
        "x_label": "T", "y_label": "V", "unit": "",
        "x_values": [0.0, 1.0], "y_values": [1.0, 2.0]
    }"#;
    let desc = render_binding_to_modality(binding, "", "description");
    assert_no_error(&desc);
}

// ── Dashboard / batch tests ─────────────────────────────────────

#[test]
fn render_dashboard_basic() {
    let bindings = r#"[
        {"channel_type": "timeseries", "id": "ts", "label": "Series",
         "x_label": "T", "y_label": "V", "unit": "",
         "x_values": [0.0, 1.0], "y_values": [1.0, 2.0]},
        {"channel_type": "bar", "id": "bar", "label": "Bars",
         "x_label": "Cat", "y_label": "Val", "unit": "",
         "categories": ["A", "B"], "values": [10.0, 20.0]}
    ]"#;
    assert_svg(&render_dashboard(bindings, ""));
}

#[test]
fn render_dashboard_with_config() {
    let bindings = r#"[
        {"channel_type": "gauge", "id": "g", "label": "CPU", "unit": "%",
         "value": 65.0, "min": 0.0, "max": 100.0,
         "normal_range": [0.0, 70.0], "warning_range": [70.0, 90.0]}
    ]"#;
    let config = r#"{"layout": "vertical", "title": "System", "domain": "measurement"}"#;
    assert_svg(&render_dashboard(bindings, config));
}

#[test]
fn render_dashboard_empty_returns_error() {
    let result = render_dashboard("[]", "");
    assert!(result.starts_with("Error:"));
}

#[test]
fn render_bindings_batch() {
    let bindings = r#"[
        {"channel_type": "timeseries", "id": "ts", "label": "Series",
         "x_label": "T", "y_label": "V", "unit": "",
         "x_values": [0.0, 1.0], "y_values": [1.0, 2.0]},
        {"channel_type": "scatter", "id": "sc", "label": "Points",
         "unit": "", "x": [1.0, 2.0], "y": [3.0, 4.0]}
    ]"#;
    let result = render_bindings(bindings, "");
    assert_no_error(&result);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&result).expect("valid JSON array");
    assert_eq!(arr.len(), 2);
    assert!(arr[0]["svg"].as_str().unwrap().starts_with("<svg"));
    assert_eq!(arr[0]["id"], "ts");
    assert_eq!(arr[1]["id"], "sc");
}

// ── Threshold test ──────────────────────────────────────────────

#[test]
fn render_binding_with_thresholds_heatmap() {
    let binding = r#"{
        "channel_type": "heatmap",
        "id": "hm", "label": "Status", "unit": "",
        "x_labels": ["A", "B"],
        "y_labels": ["1", "2"],
        "values": [25.0, 75.0, 50.0, 90.0]
    }"#;
    let thresholds = r#"[
        {"label": "normal", "min": 0.0, "max": 50.0, "status": "normal"},
        {"label": "warning", "min": 50.0, "max": 80.0, "status": "warning"},
        {"label": "critical", "min": 80.0, "max": 100.0, "status": "critical"}
    ]"#;
    assert_svg(&render_binding_with_thresholds(
        binding, "health", thresholds,
    ));
}

// ── Meta ────────────────────────────────────────────────────────

#[test]
fn version_matches_cargo() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}
