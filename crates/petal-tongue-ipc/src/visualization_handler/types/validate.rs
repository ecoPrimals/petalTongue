// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.validate`.

use petal_tongue_scene::grammar::GrammarExpr;
use serde::{Deserialize, Serialize};

/// Request for `visualization.validate`: validate grammar + data against Tufte constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    /// Grammar expression to validate.
    pub grammar: GrammarExpr,
    /// Raw data rows for the grammar.
    pub data: Vec<serde_json::Value>,
}

/// Response for `visualization.validate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    /// Overall Tufte score (0.0 to 1.0).
    pub score: f64,
    /// Whether the visualization passed validation.
    pub passed: bool,
    /// Per-constraint results.
    pub constraints: Vec<ConstraintResult>,
}

/// Result of evaluating a single Tufte constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Constraint name (e.g. "`DataInkRatio`", "`ChartjunkDetection`").
    pub name: String,
    /// Numeric score (0.0 to 1.0).
    pub score: f64,
    /// Whether the constraint passed.
    pub passed: bool,
    /// Human-readable details.
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};

    #[test]
    fn validate_request_response_roundtrip() {
        let req = ValidateRequest {
            grammar: GrammarExpr::new("data", GeometryType::Line),
            data: vec![],
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: ValidateRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = ValidateResponse {
            score: 0.85,
            passed: true,
            constraints: vec![ConstraintResult {
                name: "DataInkRatio".into(),
                score: 0.9,
                passed: true,
                details: "Good".into(),
            }],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: ValidateResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.constraints.len(), 1);
    }
}
