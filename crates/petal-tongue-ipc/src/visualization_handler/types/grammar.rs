// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.render.grammar`.

use petal_tongue_scene::grammar::GrammarExpr;
use serde::{Deserialize, Serialize};

/// Request payload for `visualization.render.grammar` (declarative scene engine path).
///
/// Springs send a `GrammarExpr` (data source, variable bindings, geometry type) plus
/// raw data. petalTongue compiles this through the scene engine with Tufte validation
/// and returns SVG (or another modality) output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRenderRequest {
    /// Unique session identifier.
    pub session_id: String,
    /// Grammar expression describing the visualization.
    pub grammar: GrammarExpr,
    /// Raw data rows (each row is a JSON object with field values).
    pub data: Vec<serde_json::Value>,
    /// Requested output modality: "svg" (default), "audio", "description".
    #[serde(default = "super::defaults::default_modality")]
    pub modality: String,
    /// Whether to run Tufte constraint validation.
    #[serde(default = "super::defaults::default_true")]
    pub validate_tufte: bool,
    /// Domain hint (e.g. "health", "physics") for constraint tuning.
    #[serde(default)]
    pub domain: Option<String>,
}

/// Response for `visualization.render.grammar`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRenderResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// The compiled output (SVG string, audio params JSON, or description text).
    pub output: serde_json::Value,
    /// Output modality that was used.
    pub modality: String,
    /// Number of scene graph nodes.
    pub scene_nodes: usize,
    /// Total rendering primitives.
    pub total_primitives: usize,
    /// Tufte constraint report (if validation was requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tufte_report: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};
    use serde_json::json;

    #[test]
    fn grammar_render_request_defaults() {
        let req = GrammarRenderRequest {
            session_id: "s1".into(),
            grammar: GrammarExpr::new("data", GeometryType::Point),
            data: vec![json!({"x": 1, "y": 2})],
            modality: "svg".into(),
            validate_tufte: true,
            domain: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: GrammarRenderRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.modality, "svg");
        assert!(restored.validate_tufte);
    }

    #[test]
    fn grammar_render_response_roundtrip() {
        let resp = GrammarRenderResponse {
            session_id: "s1".into(),
            output: json!("<svg></svg>"),
            modality: "svg".into(),
            scene_nodes: 5,
            total_primitives: 10,
            tufte_report: Some(json!({"score": 0.9})),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: GrammarRenderResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.scene_nodes, 5);
    }
}
