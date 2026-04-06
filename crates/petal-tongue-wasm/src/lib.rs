// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
//! Client-side WASM rendering module for petalTongue.
//!
//! Exposes the grammar-to-SVG pipeline as `wasm_bindgen` functions so springs
//! can embed offline-capable visualization rendering in browser UIs without
//! calling petalTongue's server-side RPC.
//!
//! # Architecture
//!
//! ```text
//! Browser JS  ──► render_grammar(grammar, data)  ──► SVG string
//!             ──► render_bindings(bindings)       ──► SVG string
//!             ──► compile_scene(grammar, data)    ──► SceneGraph JSON
//! ```
//!
//! # Usage from JavaScript
//!
//! ```js
//! import init, { render_grammar, version } from './petal_tongue_wasm.js';
//!
//! await init();
//! console.log(version());
//!
//! const svg = render_grammar(grammarJson, dataJson);
//! document.getElementById('viz').innerHTML = svg;
//! ```

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::data_binding::DataBindingCompiler;
use petal_tongue_scene::grammar::GrammarExpr;
use petal_tongue_scene::modality::svg::SvgCompiler;
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput};
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_types::DataBinding;

/// Render a grammar expression and data array to SVG.
///
/// Accepts JSON strings for both the grammar expression and the data array.
/// Returns an SVG string, or an error message prefixed with "Error:".
///
/// # Arguments
///
/// * `grammar_json` - JSON string of a `GrammarExpr`
/// * `data_json` - JSON string of a `[{...}, ...]` data array
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
    let scene = compiler.compile(&expr, &data);

    let svg = SvgCompiler::new();
    match svg.compile(&scene) {
        ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        _ => "Error: SVG compilation produced unexpected modality".to_string(),
    }
}

/// Render a grammar expression to a specified output modality.
///
/// Supported modalities: `"svg"` (default), `"description"`, `"terminal"`.
///
/// Returns the rendered output as a string, or an error message prefixed with
/// "Error:".
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
    let scene = compiler.compile(&expr, &data);

    compile_scene_to_modality(&scene, modality)
}

/// Render a single data binding (the universal DataBinding enum) to SVG.
///
/// Springs push `DataBinding` values via `visualization.render`. This function
/// compiles one binding through the grammar pipeline and returns SVG.
///
/// # Arguments
///
/// * `binding_json` - JSON string of a single `DataBinding`
/// * `domain` - Optional domain hint (e.g. `"ecology"`, `"health"`) for palette selection.
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

    let svg = SvgCompiler::new();
    match svg.compile(&scene) {
        ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        _ => "Error: SVG compilation produced unexpected modality".to_string(),
    }
}

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
    let scene = compiler.compile(&expr, &data);

    match serde_json::to_string(&scene) {
        Ok(json) => json,
        Err(e) => format!("Error: scene serialization failed: {e}"),
    }
}

/// Return the petalTongue WASM module version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn compile_scene_to_modality(scene: &SceneGraph, modality: &str) -> String {
    match modality {
        "svg" | "" => {
            let compiler = SvgCompiler::new();
            match compiler.compile(scene) {
                ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                _ => "Error: SVG compilation produced unexpected modality".to_string(),
            }
        }
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

    #[test]
    fn render_grammar_produces_svg() {
        let svg = render_grammar(GRAMMAR, DATA);
        assert!(svg.starts_with("<svg"), "expected SVG, got: {svg}");
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn render_grammar_to_modality_svg() {
        let svg = render_grammar_to_modality(GRAMMAR, DATA, "svg");
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn render_grammar_to_modality_description() {
        let desc = render_grammar_to_modality(GRAMMAR, DATA, "description");
        assert!(!desc.starts_with("Error:"), "got error: {desc}");
    }

    #[test]
    fn render_grammar_invalid_json_returns_error() {
        let result = render_grammar("{bad", "[]");
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn compile_scene_returns_json() {
        let json = compile_scene(GRAMMAR, DATA);
        assert!(!json.starts_with("Error:"), "got error: {json}");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed.is_object());
    }

    #[test]
    fn render_binding_timeseries() {
        let binding = r#"{
            "channel_type": "timeseries",
            "id": "test",
            "label": "Test Series",
            "x_label": "Time",
            "y_label": "Value",
            "unit": "u",
            "x_values": [0.0, 1.0, 2.0],
            "y_values": [1.0, 4.0, 2.0]
        }"#;
        let svg = render_binding(binding, "");
        assert!(svg.starts_with("<svg"), "expected SVG, got: {svg}");
    }

    #[test]
    fn version_matches_cargo() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
