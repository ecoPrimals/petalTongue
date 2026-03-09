// SPDX-License-Identifier: AGPL-3.0-only
//! Machine-checkable Tufte constraints for visualization quality.
//!
//! These constraints evaluate primitives against Edward Tufte's principles:
//! data-ink ratio, lie factor, chartjunk detection, color accessibility,
//! data density, and smallest effective difference.

use serde::{Deserialize, Serialize};

use crate::grammar::GrammarExpr;
use crate::primitive::Primitive;

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
    fn evaluate(&self, primitives: &[Primitive], expr: &GrammarExpr) -> ConstraintResult;

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
    pub fn evaluate_all(
        constraints: &[&dyn TufteConstraint],
        primitives: &[Primitive],
        expr: &GrammarExpr,
    ) -> Self {
        let mut results = Vec::with_capacity(constraints.len());
        for c in constraints {
            let result = c.evaluate(primitives, expr);
            results.push((c.name().to_string(), result));
        }
        let overall_score = if results.is_empty() {
            1.0
        } else {
            results.iter().map(|(_, r)| r.score).sum::<f64>() / results.len() as f64
        };
        Self {
            overall_score,
            results,
            corrections_applied: Vec::new(),
        }
    }
}

// -----------------------------------------------------------------------------
// Constraint implementations
// -----------------------------------------------------------------------------

/// Data-ink ratio: proportion of ink that carries data vs total ink.
/// Tufte: maximize data-ink, minimize non-data-ink.
pub struct DataInkRatio;

impl TufteConstraint for DataInkRatio {
    fn name(&self) -> &'static str {
        "DataInkRatio"
    }

    fn evaluate(&self, primitives: &[Primitive], _expr: &GrammarExpr) -> ConstraintResult {
        let total = primitives.len();
        if total == 0 {
            return ConstraintResult {
                passed: true,
                score: 1.0,
                message: "No primitives to evaluate".to_string(),
            };
        }
        let data_count = primitives.iter().filter(|p| p.carries_data()).count();
        let ratio = data_count as f64 / total as f64;
        let passed = ratio >= 0.5;
        ConstraintResult {
            passed,
            score: ratio,
            message: format!("Data-ink ratio: {ratio:.2} ({data_count} data / {total} total)"),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Warning
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

/// Lie factor: ratio of effect shown in graphic vs effect in data.
/// Placeholder: needs axis/scale context to properly evaluate.
pub struct LieFactor;

impl TufteConstraint for LieFactor {
    fn name(&self) -> &'static str {
        "LieFactor"
    }

    fn evaluate(&self, _primitives: &[Primitive], _expr: &GrammarExpr) -> ConstraintResult {
        ConstraintResult {
            passed: true,
            score: 1.0,
            message: "Lie factor requires axis/scale context".to_string(),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Info
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

/// Chartjunk detection: decorative elements that don't carry data.
/// Simple heuristic: primitives with fill but no data_id.
pub struct ChartjunkDetection;

impl TufteConstraint for ChartjunkDetection {
    fn name(&self) -> &'static str {
        "ChartjunkDetection"
    }

    fn evaluate(&self, primitives: &[Primitive], _expr: &GrammarExpr) -> ConstraintResult {
        let chartjunk_count = primitives
            .iter()
            .filter(|p| {
                !p.carries_data()
                    && matches!(
                        p,
                        Primitive::Point { fill: Some(_), .. }
                            | Primitive::Rect { fill: Some(_), .. }
                            | Primitive::Polygon { .. }
                            | Primitive::Arc { fill: Some(_), .. }
                            | Primitive::BezierPath { fill: Some(_), .. }
                    )
            })
            .count();
        let total = primitives.len();
        let score = if total == 0 {
            1.0
        } else {
            (1.0 - (chartjunk_count as f64 / total as f64)).max(0.0)
        };
        let passed = chartjunk_count == 0;
        ConstraintResult {
            passed,
            score,
            message: format!("Chartjunk: {chartjunk_count} decorative primitives without data"),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Warning
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

/// Color accessibility: contrast and colorblind-safe palettes.
/// Placeholder: requires palette context.
pub struct ColorAccessibility;

impl TufteConstraint for ColorAccessibility {
    fn name(&self) -> &'static str {
        "ColorAccessibility"
    }

    fn evaluate(&self, _primitives: &[Primitive], _expr: &GrammarExpr) -> ConstraintResult {
        ConstraintResult {
            passed: true,
            score: 1.0,
            message: "Color accessibility requires palette context".to_string(),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Info
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

/// Data density: amount of data per unit of graphic.
/// Higher density (more data primitives) is generally better.
pub struct DataDensity;

impl TufteConstraint for DataDensity {
    fn name(&self) -> &'static str {
        "DataDensity"
    }

    fn evaluate(&self, primitives: &[Primitive], _expr: &GrammarExpr) -> ConstraintResult {
        let data_count = primitives.iter().filter(|p| p.carries_data()).count();
        let score = if data_count > 0 { 1.0 } else { 0.0 };
        ConstraintResult {
            passed: data_count > 0,
            score,
            message: format!("Data density: {data_count} data-carrying primitives"),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Info
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

/// Smallest effective difference: visual distinctions should be just noticeable.
/// Placeholder: requires perceptual model.
pub struct SmallestEffectiveDifference;

impl TufteConstraint for SmallestEffectiveDifference {
    fn name(&self) -> &'static str {
        "SmallestEffectiveDifference"
    }

    fn evaluate(&self, _primitives: &[Primitive], _expr: &GrammarExpr) -> ConstraintResult {
        ConstraintResult {
            passed: true,
            score: 1.0,
            message: "Smallest effective difference requires perceptual model".to_string(),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Info
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GeometryType, GrammarExpr};
    use crate::primitive::{Color, Primitive};

    fn make_data_primitive() -> Primitive {
        Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("d1".to_string()),
        }
    }

    fn make_non_data_primitive() -> Primitive {
        Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            fill: Some(Color::rgb(0.5, 0.5, 0.5)),
            stroke: None,
            data_id: None,
        }
    }

    #[test]
    fn data_ink_ratio_three_data_one_non_data() {
        let primitives = vec![
            make_data_primitive(),
            make_data_primitive(),
            make_data_primitive(),
            make_non_data_primitive(),
        ];
        let expr = GrammarExpr::new("data", GeometryType::Point);
        let result = DataInkRatio.evaluate(&primitives, &expr);
        assert!((result.score - 0.75).abs() < 1e-10);
        assert!(result.passed);
    }

    #[test]
    fn data_ink_ratio_empty_primitives() {
        let primitives: Vec<Primitive> = vec![];
        let expr = GrammarExpr::new("data", GeometryType::Point);
        let result = DataInkRatio.evaluate(&primitives, &expr);
        assert!((result.score - 1.0).abs() < 1e-10);
        assert!(result.passed);
    }

    #[test]
    fn tufte_report_evaluate_all_computes_average() {
        let primitives = vec![
            make_data_primitive(),
            make_data_primitive(),
            make_non_data_primitive(),
        ];
        let expr = GrammarExpr::new("data", GeometryType::Point);
        let constraints: Vec<&dyn TufteConstraint> =
            vec![&DataInkRatio, &LieFactor, &ColorAccessibility];
        let report = TufteReport::evaluate_all(&constraints, &primitives, &expr);
        assert_eq!(report.results.len(), 3);
        // DataInkRatio: 2/3 = 0.667, LieFactor: 1.0, ColorAccessibility: 1.0
        let expected = (2.0 / 3.0 + 1.0 + 1.0) / 3.0;
        assert!((report.overall_score - expected).abs() < 1e-10);
    }

    #[test]
    fn chartjunk_detection_all_data_primitives_passes() {
        let primitives = vec![
            make_data_primitive(),
            Primitive::Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 5.0,
                fill: Some(Color::BLACK),
                stroke: None,
                corner_radius: 0.0,
                data_id: Some("r1".to_string()),
            },
        ];
        let expr = GrammarExpr::new("data", GeometryType::Bar);
        let result = ChartjunkDetection.evaluate(&primitives, &expr);
        assert!(result.passed);
        assert!(result.score >= 1.0 || (result.score - 1.0).abs() < 1e-10);
    }
}
