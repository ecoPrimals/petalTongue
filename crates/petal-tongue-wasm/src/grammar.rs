// SPDX-License-Identifier: AGPL-3.0-or-later
//! Grammar expression rendering entry points for the WASM API.

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::grammar::GrammarExpr;

use crate::compile::{compile_scene_to_modality, scene_to_svg};

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
