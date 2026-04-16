// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Enum dispatch for [`TufteConstraint`] (replaces `&dyn TufteConstraint`).
#[derive(Debug, Clone, Copy)]
pub enum TufteConstraintImpl {
    /// Data-ink ratio.
    DataInkRatio,
    /// Lie factor.
    LieFactor,
    /// Chartjunk detection.
    ChartjunkDetection,
    /// Color accessibility.
    ColorAccessibility,
    /// Data density.
    DataDensity,
    /// Smallest effective difference.
    SmallestEffectiveDifference,
    /// Small multiples preference.
    SmallMultiplesPreference,
}

impl TufteConstraint for TufteConstraintImpl {
    fn name(&self) -> &str {
        match self {
            Self::DataInkRatio => TufteConstraint::name(&DataInkRatio),
            Self::LieFactor => TufteConstraint::name(&LieFactor),
            Self::ChartjunkDetection => TufteConstraint::name(&ChartjunkDetection),
            Self::ColorAccessibility => TufteConstraint::name(&ColorAccessibility),
            Self::DataDensity => TufteConstraint::name(&DataDensity),
            Self::SmallestEffectiveDifference => {
                TufteConstraint::name(&SmallestEffectiveDifference)
            }
            Self::SmallMultiplesPreference => TufteConstraint::name(&SmallMultiplesPreference),
        }
    }

    fn evaluate(
        &self,
        primitives: &[Primitive],
        expr: &GrammarExpr,
        data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
        match self {
            Self::DataInkRatio => TufteConstraint::evaluate(&DataInkRatio, primitives, expr, data),
            Self::LieFactor => TufteConstraint::evaluate(&LieFactor, primitives, expr, data),
            Self::ChartjunkDetection => {
                TufteConstraint::evaluate(&ChartjunkDetection, primitives, expr, data)
            }
            Self::ColorAccessibility => {
                TufteConstraint::evaluate(&ColorAccessibility, primitives, expr, data)
            }
            Self::DataDensity => TufteConstraint::evaluate(&DataDensity, primitives, expr, data),
            Self::SmallestEffectiveDifference => {
                TufteConstraint::evaluate(&SmallestEffectiveDifference, primitives, expr, data)
            }
            Self::SmallMultiplesPreference => {
                TufteConstraint::evaluate(&SmallMultiplesPreference, primitives, expr, data)
            }
        }
    }

    fn evaluate_plan(&self, plan: &RenderPlan) -> ConstraintResult {
        match self {
            Self::DataInkRatio => TufteConstraint::evaluate_plan(&DataInkRatio, plan),
            Self::LieFactor => TufteConstraint::evaluate_plan(&LieFactor, plan),
            Self::ChartjunkDetection => TufteConstraint::evaluate_plan(&ChartjunkDetection, plan),
            Self::ColorAccessibility => TufteConstraint::evaluate_plan(&ColorAccessibility, plan),
            Self::DataDensity => TufteConstraint::evaluate_plan(&DataDensity, plan),
            Self::SmallestEffectiveDifference => {
                TufteConstraint::evaluate_plan(&SmallestEffectiveDifference, plan)
            }
            Self::SmallMultiplesPreference => {
                TufteConstraint::evaluate_plan(&SmallMultiplesPreference, plan)
            }
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        match self {
            Self::DataInkRatio => TufteConstraint::severity(&DataInkRatio),
            Self::LieFactor => TufteConstraint::severity(&LieFactor),
            Self::ChartjunkDetection => TufteConstraint::severity(&ChartjunkDetection),
            Self::ColorAccessibility => TufteConstraint::severity(&ColorAccessibility),
            Self::DataDensity => TufteConstraint::severity(&DataDensity),
            Self::SmallestEffectiveDifference => {
                TufteConstraint::severity(&SmallestEffectiveDifference)
            }
            Self::SmallMultiplesPreference => TufteConstraint::severity(&SmallMultiplesPreference),
        }
    }

    fn auto_correctable(&self) -> bool {
        match self {
            Self::DataInkRatio => TufteConstraint::auto_correctable(&DataInkRatio),
            Self::LieFactor => TufteConstraint::auto_correctable(&LieFactor),
            Self::ChartjunkDetection => TufteConstraint::auto_correctable(&ChartjunkDetection),
            Self::ColorAccessibility => TufteConstraint::auto_correctable(&ColorAccessibility),
            Self::DataDensity => TufteConstraint::auto_correctable(&DataDensity),
            Self::SmallestEffectiveDifference => {
                TufteConstraint::auto_correctable(&SmallestEffectiveDifference)
            }
            Self::SmallMultiplesPreference => {
                TufteConstraint::auto_correctable(&SmallMultiplesPreference)
            }
        }
    }
}

/// Overall Tufte report from evaluating all constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufteReport {
    /// Average score across all constraints (0.0 to 1.0).
    pub overall_score: f64,
    /// Per-constraint results: (`constraint_name`, result).
    pub results: Vec<(String, ConstraintResult)>,
    /// List of corrections that were applied (if any).
    pub corrections_applied: Vec<String>,
}

impl TufteReport {
    /// Evaluate all constraints and produce a report.
    /// `data` is optional raw data for constraints that need category counts (e.g. color).
    pub fn evaluate_all(
        constraints: &[TufteConstraintImpl],
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
