// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation pipeline - runs all Tufte constraints and produces a report.

use crate::grammar::GrammarExpr;
use crate::primitive::Primitive;

use super::{TufteConstraint, TufteReport};

/// Run the validation pipeline: evaluate all constraints and produce a report.
///
/// `data` is optional raw data for constraints that need category counts (e.g. color).
pub fn evaluate_all(
    constraints: &[super::TufteConstraintImpl],
    primitives: &[Primitive],
    expr: &GrammarExpr,
    data: Option<&[serde_json::Value]>,
) -> TufteReport {
    let mut results = Vec::with_capacity(constraints.len());
    for c in constraints {
        let result = TufteConstraint::evaluate(c, primitives, expr, data);
        results.push((TufteConstraint::name(c).to_string(), result));
    }
    #[expect(clippy::cast_precision_loss, reason = "average score: f64 sufficient")]
    let overall_score = if results.is_empty() {
        1.0
    } else {
        results.iter().map(|(_, r)| r.score).sum::<f64>() / results.len() as f64
    };
    TufteReport {
        overall_score,
        results,
        corrections_applied: Vec::new(),
    }
}
