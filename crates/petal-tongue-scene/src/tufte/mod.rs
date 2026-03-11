// SPDX-License-Identifier: AGPL-3.0-only
//! Machine-checkable Tufte constraints for visualization quality.
//!
//! These constraints evaluate primitives against Edward Tufte's principles:
//! data-ink ratio, lie factor, chartjunk detection, color accessibility,
//! data density, and smallest effective difference.

mod constraints;
mod pipeline;

use serde::{Deserialize, Serialize};

use crate::grammar::GrammarExpr;
use crate::primitive::Primitive;
use crate::render_plan::RenderPlan;

pub use constraints::{
    ChartjunkDetection, ColorAccessibility, DataDensity, DataInkRatio, LieFactor,
    SmallMultiplesPreference, SmallestEffectiveDifference,
};

/// Severity of a constraint violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    Info,
    Warning,
    Error,
}

/// Result of evaluating a single constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Whether the constraint passed.
    pub passed: bool,
    /// Numeric score (0.0 to 1.0 typically; higher is better).
    pub score: f64,
    /// Human-readable message.
    pub message: String,
}

/// Trait for machine-checkable Tufte constraints.
pub trait TufteConstraint: Send + Sync {
    /// Human-readable constraint name.
    fn name(&self) -> &str;

    /// Evaluate the constraint against primitives and grammar expression.
    /// `data` is optional raw data for constraints that need category counts (e.g. color).
    fn evaluate(
        &self,
        primitives: &[Primitive],
        expr: &GrammarExpr,
        data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult;

    /// Evaluate with full render plan context (axis/scale/panel metadata).
    /// Default delegates to `evaluate` on the plan's flattened primitives.
    /// No raw data is available in plan context; constraints that need it use None.
    fn evaluate_plan(&self, plan: &RenderPlan) -> ConstraintResult {
        let primitives: Vec<Primitive> = plan
            .scene
            .flatten()
            .into_iter()
            .map(|(_, p)| p.clone())
            .collect();
        self.evaluate(&primitives, &plan.grammar, None)
    }

    /// Severity when the constraint fails.
    fn severity(&self) -> ConstraintSeverity;

    /// Whether this constraint can be auto-corrected.
    fn auto_correctable(&self) -> bool;
}

/// Overall Tufte report from evaluating all constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufteReport {
    /// Average score across all constraints (0.0 to 1.0).
    pub overall_score: f64,
    /// Per-constraint results: (constraint_name, result).
    pub results: Vec<(String, ConstraintResult)>,
    /// List of corrections that were applied (if any).
    pub corrections_applied: Vec<String>,
}

impl TufteReport {
    /// Evaluate all constraints and produce a report.
    /// `data` is optional raw data for constraints that need category counts (e.g. color).
    pub fn evaluate_all(
        constraints: &[&dyn TufteConstraint],
        primitives: &[Primitive],
        expr: &GrammarExpr,
        data: Option<&[serde_json::Value]>,
    ) -> Self {
        pipeline::evaluate_all(constraints, primitives, expr, data)
    }
}

#[cfg(test)]
mod proptest_tests;
#[cfg(test)]
mod tests;
