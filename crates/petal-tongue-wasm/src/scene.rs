// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene graph compile and render entry points for the WASM API.

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::grammar::GrammarExpr;
use petal_tongue_scene::scene_graph::SceneGraph;

use crate::compile::{compile_scene_to_modality, scene_to_svg};

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
