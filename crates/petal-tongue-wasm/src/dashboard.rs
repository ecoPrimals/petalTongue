// SPDX-License-Identifier: AGPL-3.0-or-later
//! Dashboard and batch DataBinding rendering for the WASM API.

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::dashboard::{DashboardConfig, DashboardLayout};
use petal_tongue_scene::data_binding::DataBindingCompiler;
use petal_tongue_types::DataBinding;

use crate::binding::{binding_id, binding_label};
use crate::compile::scene_to_svg;

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
        return "Error: empty bindings array".to_owned();
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
