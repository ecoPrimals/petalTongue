// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Client-side WASM rendering module for petalTongue (WS-4).
//!
//! Compiles the Grammar of Graphics pipeline to `wasm32-unknown-unknown` so
//! browsers can render DataBindings, grammars, dashboards, and scene graphs
//! **without round-tripping to a petalTongue server**.  This is the foundation
//! for offline sporePrint content and lithoSpore deployments.
//!
//! # Architecture
//!
//! ```text
//! Browser JS ──► render_grammar(grammar, data)           ──► SVG string
//!            ──► render_binding(binding, domain)          ──► SVG string
//!            ──► render_bindings(bindings, domain)        ──► SVG string (dashboard)
//!            ──► render_dashboard(bindings, config)       ──► SVG string
//!            ──► render_scene(scene_json)                 ──► SVG string
//!            ──► compile_scene(grammar, data)             ──► SceneGraph JSON
//!            ──► validate_grammar(grammar, data)          ──► Tufte report JSON
//!            ──► render_binding_to_modality(…, modality)  ──► rendered string
//! ```
//!
//! # Usage from JavaScript
//!
//! ```js
//! import init, { render_grammar, render_binding, render_dashboard, version } from './petal_tongue_wasm.js';
//!
//! await init();
//! console.log(version());
//!
//! const svg = render_grammar(grammarJson, dataJson);
//! document.getElementById('viz').innerHTML = svg;
//!
//! const dashboard = render_dashboard(bindingsArrayJson, '{"domain":"health"}');
//! document.getElementById('dashboard').innerHTML = dashboard;
//! ```
//!
//! # Dependency chain (all wasm32-safe)
//!
//! `petal-tongue-types` → `petal-tongue-scene` → `petal-tongue-wasm`

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;

/// Initialize the WASM module with better panic messages.
///
/// Call this once from JavaScript before using any other functions.
/// Wires `console.error` as the panic handler so Rust panics produce
/// readable stack traces in the browser devtools.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
use petal_tongue_scene::dashboard::{DashboardConfig, DashboardLayout};
use petal_tongue_scene::data_binding::DataBindingCompiler;
use petal_tongue_scene::grammar::GrammarExpr;
use petal_tongue_scene::modality::svg::SvgCompiler;
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput};
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::tufte::TufteConstraintImpl;
use petal_tongue_types::DataBinding;

// ── Grammar rendering ───────────────────────────────────────────────────

/// Render a grammar expression and data array to SVG.
///
/// Accepts JSON strings for both the grammar expression and the data array.
/// Returns an SVG string, or an error message prefixed with "Error:".
#[wasm_bindgen]
pub fn render_grammar(grammar_json: &str, data_json: &str) -> String {
    let expr: GrammarExpr = match serde_json::from_str(grammar_json) {
        Ok(e) => e,
        Err(e) => return format!("Error: invalid grammar: {e}"),
    };

    let data: Vec<serde_json::Value> = match serde_json::from_str(data_json) {
        Ok(d) => d,
        Err(e) => return format!("Error: invalid data: {e}"),
    };

    let compiler = GrammarCompiler::new();
    let scene = if expr.facets.is_some() {
        compiler.compile_faceted(&expr, &data)
    } else {
        compiler.compile(&expr, &data)
    };

    scene_to_svg(&scene)
}

/// Render a grammar expression to a specified output modality.
///
/// Supported modalities: `"svg"` (default), `"description"`, `"terminal"`.
#[wasm_bindgen]
pub fn render_grammar_to_modality(grammar_json: &str, data_json: &str, modality: &str) -> String {
    let expr: GrammarExpr = match serde_json::from_str(grammar_json) {
        Ok(e) => e,
        Err(e) => return format!("Error: invalid grammar: {e}"),
    };

    let data: Vec<serde_json::Value> = match serde_json::from_str(data_json) {
        Ok(d) => d,
        Err(e) => return format!("Error: invalid data: {e}"),
    };

    let compiler = GrammarCompiler::new();
    let scene = if expr.facets.is_some() {
        compiler.compile_faceted(&expr, &data)
    } else {
        compiler.compile(&expr, &data)
    };

    compile_scene_to_modality(&scene, modality)
}

// ── DataBinding rendering ───────────────────────────────────────────────

/// Render a single data binding to SVG.
///
/// Springs push `DataBinding` values via `visualization.render`. This function
/// compiles one binding through the grammar pipeline and returns SVG.
///
/// # Arguments
///
/// * `binding_json` - JSON string of a single `DataBinding`
/// * `domain` - Optional domain hint (e.g. `"ecology"`, `"health"`).
///   Pass empty string for default.
#[wasm_bindgen]
pub fn render_binding(binding_json: &str, domain: &str) -> String {
    let binding: DataBinding = match serde_json::from_str(binding_json) {
        Ok(b) => b,
        Err(e) => return format!("Error: invalid binding: {e}"),
    };

    let domain_opt = if domain.is_empty() {
        None
    } else {
        Some(domain)
    };

    let (expr, data) = DataBindingCompiler::compile(&binding, domain_opt);
    let grammar = GrammarCompiler::new();
    let scene = grammar.compile(&expr, &data);

    scene_to_svg(&scene)
}

/// Render a single data binding to a specified modality.
///
/// Supported modalities: `"svg"` (default), `"description"`, `"terminal"`.
#[wasm_bindgen]
pub fn render_binding_to_modality(binding_json: &str, domain: &str, modality: &str) -> String {
    let binding: DataBinding = match serde_json::from_str(binding_json) {
        Ok(b) => b,
        Err(e) => return format!("Error: invalid binding: {e}"),
    };

    let domain_opt = if domain.is_empty() {
        None
    } else {
        Some(domain)
    };

    let (expr, data) = DataBindingCompiler::compile(&binding, domain_opt);
    let grammar = GrammarCompiler::new();
    let scene = grammar.compile(&expr, &data);

    compile_scene_to_modality(&scene, modality)
}

/// Render a single data binding with threshold coloring to SVG.
///
/// Threshold ranges color Heatmap/FieldMap cells by status
/// (normal/warning/critical) instead of continuous intensity.
///
/// `thresholds_json` is a JSON array of `ThresholdRange` objects:
/// `[{"label":"normal","min":0.0,"max":50.0,"status":"normal"}, ...]`
#[wasm_bindgen]
pub fn render_binding_with_thresholds(
    binding_json: &str,
    domain: &str,
    thresholds_json: &str,
) -> String {
    let binding: DataBinding = match serde_json::from_str(binding_json) {
        Ok(b) => b,
        Err(e) => return format!("Error: invalid binding: {e}"),
    };

    let thresholds: Vec<petal_tongue_types::ThresholdRange> =
        match serde_json::from_str(thresholds_json) {
            Ok(t) => t,
            Err(e) => return format!("Error: invalid thresholds: {e}"),
        };

    let domain_opt = if domain.is_empty() {
        None
    } else {
        Some(domain)
    };

    let (expr, data) =
        DataBindingCompiler::compile_with_thresholds(&binding, domain_opt, &thresholds);
    let grammar = GrammarCompiler::new();
    let scene = grammar.compile(&expr, &data);

    scene_to_svg(&scene)
}

// ── Batch / dashboard rendering ─────────────────────────────────────────

/// Render multiple data bindings as a dashboard grid to SVG.
///
/// `bindings_json` is a JSON array of `DataBinding` objects.
/// `config_json` is an optional JSON object with dashboard layout options:
///
/// ```json
/// {
///   "layout": "grid",       // "grid" (default), "vertical", "horizontal"
///   "max_columns": 3,       // for grid layout
///   "panel_width": 400.0,
///   "panel_height": 300.0,
///   "spacing": 20.0,
///   "title": "My Dashboard",
///   "domain": "health"
/// }
/// ```
///
/// Pass empty string for `config_json` to use defaults.
#[wasm_bindgen]
pub fn render_dashboard(bindings_json: &str, config_json: &str) -> String {
    let bindings: Vec<DataBinding> = match serde_json::from_str(bindings_json) {
        Ok(b) => b,
        Err(e) => return format!("Error: invalid bindings array: {e}"),
    };

    if bindings.is_empty() {
        return "Error: empty bindings array".to_string();
    }

    let config = if config_json.is_empty() {
        DashboardConfig::default()
    } else {
        parse_dashboard_config(config_json)
    };

    let dashboard = petal_tongue_scene::dashboard::build_dashboard(&bindings, &config);

    scene_to_svg(&dashboard.scene)
}

/// Render multiple bindings as individual SVGs, returned as a JSON array.
///
/// Each element is `{"id": "...", "svg": "...", "label": "..."}`.
/// Useful when the caller wants to position panels with CSS rather than
/// using the built-in dashboard grid.
#[wasm_bindgen]
pub fn render_bindings(bindings_json: &str, domain: &str) -> String {
    let bindings: Vec<DataBinding> = match serde_json::from_str(bindings_json) {
        Ok(b) => b,
        Err(e) => return format!("Error: invalid bindings array: {e}"),
    };

    let domain_opt = if domain.is_empty() {
        None
    } else {
        Some(domain)
    };

    let results: Vec<serde_json::Value> = bindings
        .iter()
        .map(|binding| {
            let (expr, data) = DataBindingCompiler::compile(binding, domain_opt);
            let compiler = GrammarCompiler::new();
            let scene = compiler.compile(&expr, &data);
            let svg = scene_to_svg(&scene);
            serde_json::json!({
                "id": binding_id(binding),
                "label": binding_label(binding),
                "svg": svg,
            })
        })
        .collect();

    serde_json::to_string(&results).unwrap_or_else(|e| format!("Error: serialization: {e}"))
}

// ── Scene graph operations ──────────────────────────────────────────────

/// Compile a grammar expression and data to a SceneGraph, returned as JSON.
///
/// Useful for springs that want to inspect or transform the scene graph
/// before rendering.
#[wasm_bindgen]
pub fn compile_scene(grammar_json: &str, data_json: &str) -> String {
    let expr: GrammarExpr = match serde_json::from_str(grammar_json) {
        Ok(e) => e,
        Err(e) => return format!("Error: invalid grammar: {e}"),
    };

    let data: Vec<serde_json::Value> = match serde_json::from_str(data_json) {
        Ok(d) => d,
        Err(e) => return format!("Error: invalid data: {e}"),
    };

    let compiler = GrammarCompiler::new();
    let scene = if expr.facets.is_some() {
        compiler.compile_faceted(&expr, &data)
    } else {
        compiler.compile(&expr, &data)
    };

    match serde_json::to_string(&scene) {
        Ok(json) => json,
        Err(e) => format!("Error: scene serialization failed: {e}"),
    }
}

/// Render a pre-built SceneGraph (as JSON) to SVG.
///
/// Accepts the JSON output of `compile_scene` or any valid `SceneGraph`
/// JSON, and renders it to SVG without recompilation.
#[wasm_bindgen]
pub fn render_scene(scene_json: &str) -> String {
    let scene: SceneGraph = match serde_json::from_str(scene_json) {
        Ok(s) => s,
        Err(e) => return format!("Error: invalid scene graph: {e}"),
    };

    scene_to_svg(&scene)
}

/// Render a pre-built SceneGraph to a specified modality.
#[wasm_bindgen]
pub fn render_scene_to_modality(scene_json: &str, modality: &str) -> String {
    let scene: SceneGraph = match serde_json::from_str(scene_json) {
        Ok(s) => s,
        Err(e) => return format!("Error: invalid scene graph: {e}"),
    };

    compile_scene_to_modality(&scene, modality)
}

// ── Tufte validation ────────────────────────────────────────────────────

/// Validate a grammar expression against Tufte principles.
///
/// Returns a JSON report with constraint results:
/// ```json
/// {
///   "valid": true,
///   "score": 0.95,
///   "results": [
///     {"constraint": "data-ink-ratio", "passed": true, "value": 0.92, "threshold": 0.5},
///     ...
///   ]
/// }
/// ```
#[wasm_bindgen]
pub fn validate_grammar(grammar_json: &str, data_json: &str) -> String {
    let expr: GrammarExpr = match serde_json::from_str(grammar_json) {
        Ok(e) => e,
        Err(e) => return format!("Error: invalid grammar: {e}"),
    };

    let data: Vec<serde_json::Value> = match serde_json::from_str(data_json) {
        Ok(d) => d,
        Err(e) => return format!("Error: invalid data: {e}"),
    };

    let compiler = GrammarCompiler::new();
    let constraints = all_tufte_constraints();
    let (_, report) = compiler.compile_with_constraints(&expr, &data, &constraints);

    match serde_json::to_string(&report) {
        Ok(json) => json,
        Err(e) => format!("Error: report serialization failed: {e}"),
    }
}

// ── Meta ────────────────────────────────────────────────────────────────

/// Return the petalTongue WASM module version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ── Internal helpers ────────────────────────────────────────────────────

fn all_tufte_constraints() -> Vec<TufteConstraintImpl> {
    vec![
        TufteConstraintImpl::DataInkRatio,
        TufteConstraintImpl::LieFactor,
        TufteConstraintImpl::ChartjunkDetection,
        TufteConstraintImpl::ColorAccessibility,
        TufteConstraintImpl::DataDensity,
        TufteConstraintImpl::SmallestEffectiveDifference,
        TufteConstraintImpl::SmallMultiplesPreference,
    ]
}

fn scene_to_svg(scene: &SceneGraph) -> String {
    let compiler = SvgCompiler::new();
    match compiler.compile(scene) {
        ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        _ => "Error: SVG compilation produced unexpected modality".to_string(),
    }
}

fn compile_scene_to_modality(scene: &SceneGraph, modality: &str) -> String {
    match modality {
        "svg" | "" => scene_to_svg(scene),
        "description" => {
            let compiler = petal_tongue_scene::modality::description::DescriptionCompiler::new();
            match compiler.compile(scene) {
                ModalityOutput::Description(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                _ => "Error: description compilation produced unexpected modality".to_string(),
            }
        }
        "terminal" => {
            let compiler = petal_tongue_scene::modality::terminal::TerminalCompiler::new(80, 24);
            match compiler.compile(scene) {
                ModalityOutput::TerminalCells(cells) => cells
                    .iter()
                    .map(|row| row.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => "Error: terminal compilation produced unexpected modality".to_string(),
            }
        }
        other => format!("Error: unsupported modality: {other}"),
    }
}

fn parse_dashboard_config(json: &str) -> DashboardConfig {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    let layout = match v
        .get("layout")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("grid")
    {
        "vertical" => DashboardLayout::Vertical,
        "horizontal" => DashboardLayout::Horizontal,
        _ => DashboardLayout::Grid {
            max_columns: v
                .get("max_columns")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(3) as usize,
        },
    };
    DashboardConfig {
        layout,
        panel_width: v
            .get("panel_width")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(400.0),
        panel_height: v
            .get("panel_height")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(300.0),
        spacing: v
            .get("spacing")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(20.0),
        title: v
            .get("title")
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        domain: v
            .get("domain")
            .and_then(serde_json::Value::as_str)
            .map(String::from),
    }
}

fn binding_id(binding: &DataBinding) -> &str {
    match binding {
        DataBinding::TimeSeries { id, .. }
        | DataBinding::Distribution { id, .. }
        | DataBinding::Bar { id, .. }
        | DataBinding::Gauge { id, .. }
        | DataBinding::Spectrum { id, .. }
        | DataBinding::Heatmap { id, .. }
        | DataBinding::Scatter { id, .. }
        | DataBinding::Scatter3D { id, .. }
        | DataBinding::FieldMap { id, .. }
        | DataBinding::GameScene { id, .. }
        | DataBinding::Soundscape { id, .. }
        | DataBinding::GenomeTrack { id, .. }
        | DataBinding::CircularMap { id, .. } => id,
    }
}

fn binding_label(binding: &DataBinding) -> &str {
    match binding {
        DataBinding::TimeSeries { label, .. }
        | DataBinding::Distribution { label, .. }
        | DataBinding::Bar { label, .. }
        | DataBinding::Gauge { label, .. }
        | DataBinding::Spectrum { label, .. }
        | DataBinding::Heatmap { label, .. }
        | DataBinding::Scatter { label, .. }
        | DataBinding::Scatter3D { label, .. }
        | DataBinding::FieldMap { label, .. }
        | DataBinding::GameScene { label, .. }
        | DataBinding::Soundscape { label, .. }
        | DataBinding::GenomeTrack { label, .. }
        | DataBinding::CircularMap { label, .. } => label,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
}
