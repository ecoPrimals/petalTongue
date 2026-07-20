// SPDX-License-Identifier: AGPL-3.0-or-later
//! `DataBinding` rendering entry points for the WASM API.

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::data_binding::DataBindingCompiler;
use petal_tongue_types::DataBinding;

use crate::compile::{compile_scene_to_modality, scene_to_svg, scene_to_webgl};

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
#[must_use]
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
#[must_use]
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
#[must_use]
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

pub fn binding_id(binding: &DataBinding) -> &str {
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
        | DataBinding::CircularMap { id, .. }
        | DataBinding::ColorGrid { id, .. } => id,
    }
}

pub fn binding_label(binding: &DataBinding) -> &str {
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
        | DataBinding::CircularMap { label, .. }
        | DataBinding::ColorGrid { label, .. } => label,
    }
}

/// Render a data binding directly to WebGL draw commands (JSON).
///
/// Returns a JSON string containing `vertices`, `indices`, `draw_calls`,
/// and `view_projection` arrays ready for GPU upload via `gl.bufferData()`.
///
/// This is the primary entry point for browser-side 3D/WebGL rendering
/// of petalTongue visualizations without a server round-trip.
#[must_use]
#[wasm_bindgen]
pub fn render_binding_webgl(binding_json: &str, domain: &str) -> String {
    let binding: DataBinding = match serde_json::from_str(binding_json) {
        Ok(b) => b,
        Err(e) => return format!("{{\"error\": \"invalid binding: {e}\"}}"),
    };

    let domain_opt = if domain.is_empty() {
        None
    } else {
        Some(domain)
    };

    let (expr, data) = DataBindingCompiler::compile(&binding, domain_opt);
    let grammar = GrammarCompiler::new();
    let scene = grammar.compile(&expr, &data);

    scene_to_webgl(&scene)
}

/// Render a `ColorGrid` binding to WebGL for bingoCube commitment visualization.
///
/// Convenience wrapper that constructs a `ColorGrid` `DataBinding` from raw parameters
/// and compiles to WebGL draw commands in a single call.
///
/// # Arguments
/// * `id` - Grid identifier
/// * `cols` - Number of columns
/// * `rows` - Number of rows  
/// * `colors_json` - JSON array of `[r,g,b,a]` arrays (row-major, f32 0.0-1.0)
/// * `reveal_fraction` - 0.0 (all hidden) to 1.0 (all revealed)
#[must_use]
#[wasm_bindgen]
pub fn render_color_grid_webgl(
    id: &str,
    cols: u32,
    rows: u32,
    colors_json: &str,
    reveal_fraction: f64,
) -> String {
    let colors: Vec<[f32; 4]> = match serde_json::from_str(colors_json) {
        Ok(c) => c,
        Err(e) => return format!("{{\"error\": \"invalid colors: {e}\"}}"),
    };

    let binding = DataBinding::ColorGrid {
        id: id.to_owned(),
        label: format!("{id} grid"),
        cols: cols as usize,
        rows: rows as usize,
        colors,
        reveal_fraction,
    };

    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    let grammar = GrammarCompiler::new();
    let scene = grammar.compile(&expr, &data);

    scene_to_webgl(&scene)
}
