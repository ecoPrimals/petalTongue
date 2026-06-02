// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tufte principle validation entry points for the WASM API.

use wasm_bindgen::prelude::*;

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::grammar::GrammarExpr;

use crate::compile::all_tufte_constraints;

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
