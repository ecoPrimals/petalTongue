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

mod binding;
mod compile;
mod dashboard;
mod grammar;
mod scene;
mod validation;

#[cfg(test)]
mod tests;

pub use binding::{render_binding, render_binding_to_modality, render_binding_with_thresholds};
pub use dashboard::{render_bindings, render_dashboard};
pub use grammar::{render_grammar, render_grammar_to_modality};
pub use scene::{compile_scene, render_scene, render_scene_to_modality};
pub use validation::validate_grammar;

/// Initialize the WASM module with better panic messages.
///
/// Call this once from JavaScript before using any other functions.
/// Wires `console.error` as the panic handler so Rust panics produce
/// readable stack traces in the browser devtools.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Return the petalTongue WASM module version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
