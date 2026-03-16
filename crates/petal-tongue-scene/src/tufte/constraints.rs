// SPDX-License-Identifier: AGPL-3.0-or-later
//! Individual Tufte constraint implementations.

use crate::grammar::GrammarExpr;
use crate::primitive::{Color, Primitive};
use crate::render_plan::RenderPlan;

use super::{ConstraintResult, ConstraintSeverity, TufteConstraint};

/// Data-ink ratio: proportion of ink that carries data vs total ink.
/// Tufte: maximize data-ink, minimize non-data-ink.
pub struct DataInkRatio;

impl TufteConstraint for DataInkRatio {
    fn name(&self) -> &'static str {
        "DataInkRatio"
    }

    fn evaluate(
        &self,
        primitives: &[Primitive],
        _expr: &GrammarExpr,
        _data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
        let total = primitives.len();
        if total == 0 {
            return ConstraintResult {
                passed: true,
                score: 1.0,
                message: "No primitives to evaluate".to_string(),
            };
        }
        let data_count = primitives.iter().filter(|p| p.carries_data()).count();
        #[expect(
            clippy::cast_precision_loss,
            reason = "ratio: f64 sufficient for scoring"
        )]
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

/// Lie factor: ratio of visual effect to data effect.
///
/// Tufte: `lie_factor = (visual effect) / (data effect)`. A factor of 1.0
/// means the graphic truthfully represents the data. Common violations:
/// - Truncated Y axis (bar chart not starting at zero)
/// - Area/volume scaling for 1D data (size aesthetic → πr² distortion)
pub struct LieFactor;

impl TufteConstraint for LieFactor {
    fn name(&self) -> &'static str {
        "LieFactor"
    }

    fn evaluate(
        &self,
        _primitives: &[Primitive],
        _expr: &GrammarExpr,
        _data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
        ConstraintResult {
            passed: true,
            score: 1.0,
            message: "Lie factor (basic): no plan context available".to_string(),
        }
    }

    fn evaluate_plan(&self, plan: &RenderPlan) -> ConstraintResult {
        let mut issues = Vec::new();
        let mut factor = 1.0_f64;

        if plan.has_bar_or_area_geom()
            && let Some(y_min) = plan.y_domain_min()
            && y_min > 0.0
            && let Some(y_max) = plan.y_domain_max()
            && y_max > 0.0
        {
            let data_range = y_max - y_min;
            factor = y_max / data_range;
            issues.push(format!(
                "Y axis truncated at {y_min:.1} (lie factor {factor:.2})"
            ));
        }

        if plan.has_size_aesthetic() {
            factor = factor.max(std::f64::consts::PI);
            issues.push("Size aesthetic maps to area (πr²); consider radius scaling".to_string());
        }

        let score = if factor.abs() < f64::EPSILON {
            0.0
        } else {
            (1.0 / factor).clamp(0.0, 1.0)
        };
        let passed = (factor - 1.0).abs() < 0.5;
        let message = if issues.is_empty() {
            format!("Lie factor: {factor:.2} (truthful)")
        } else {
            format!("Lie factor: {factor:.2} — {}", issues.join("; "))
        };

        ConstraintResult {
            passed,
            score,
            message,
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Warning
    }

    fn auto_correctable(&self) -> bool {
        true
    }
}

/// Chartjunk detection: decorative elements that don't carry data.
/// Simple heuristic: primitives with fill but no `data_id`.
pub struct ChartjunkDetection;

impl TufteConstraint for ChartjunkDetection {
    fn name(&self) -> &'static str {
        "ChartjunkDetection"
    }

    fn evaluate(
        &self,
        primitives: &[Primitive],
        _expr: &GrammarExpr,
        _data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
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
            #[expect(
                clippy::cast_precision_loss,
                reason = "ratio: f64 sufficient for scoring"
            )]
            let junk_ratio = chartjunk_count as f64 / total as f64;
            (1.0 - junk_ratio).max(0.0)
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

/// Color accessibility: WCAG contrast ratio and distinguishability.
///
/// Checks that fill colors used in data-carrying primitives have
/// sufficient contrast against each other and against a dark background.
/// Uses relative luminance and WCAG 2.1 AA contrast ratio (4.5:1 for text,
/// 3:1 for graphical objects).
pub struct ColorAccessibility;

impl ColorAccessibility {
    fn relative_luminance(c: Color) -> f64 {
        fn linearize(v: f64) -> f64 {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        0.0722f64.mul_add(
            linearize(f64::from(c.b)),
            0.2126f64.mul_add(
                linearize(f64::from(c.r)),
                0.7152 * linearize(f64::from(c.g)),
            ),
        )
    }

    fn contrast_ratio(c1: Color, c2: Color) -> f64 {
        let l1 = Self::relative_luminance(c1);
        let l2 = Self::relative_luminance(c2);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

impl TufteConstraint for ColorAccessibility {
    fn name(&self) -> &'static str {
        "ColorAccessibility"
    }

    fn evaluate(
        &self,
        primitives: &[Primitive],
        _expr: &GrammarExpr,
        _data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
        let bg = Color::rgba(0.12, 0.12, 0.14, 1.0);
        let fills: Vec<Color> = primitives
            .iter()
            .filter(|p| p.carries_data())
            .filter_map(|p| match p {
                Primitive::Point { fill, .. } | Primitive::Rect { fill, .. } => *fill,
                Primitive::Polygon { fill, .. } => Some(*fill),
                _ => None,
            })
            .collect();

        if fills.is_empty() {
            return ConstraintResult {
                passed: true,
                score: 1.0,
                message: "No data colors to evaluate".to_string(),
            };
        }

        let mut min_bg_contrast = f64::INFINITY;
        for &f in &fills {
            let cr = Self::contrast_ratio(f, bg);
            if cr < min_bg_contrast {
                min_bg_contrast = cr;
            }
        }

        let mut min_pair_contrast = f64::INFINITY;
        for i in 0..fills.len() {
            for j in (i + 1)..fills.len() {
                let cr = Self::contrast_ratio(fills[i], fills[j]);
                if cr < min_pair_contrast {
                    min_pair_contrast = cr;
                }
            }
        }
        if min_pair_contrast.is_infinite() {
            min_pair_contrast = min_bg_contrast;
        }

        let bg_ok = min_bg_contrast >= 3.0;
        let pair_ok = min_pair_contrast >= 1.5;
        let passed = bg_ok && pair_ok;
        let score = f64::midpoint(
            (min_bg_contrast / 4.5).min(1.0),
            (min_pair_contrast / 3.0).min(1.0),
        );

        ConstraintResult {
            passed,
            score,
            message: format!(
                "Min background contrast: {min_bg_contrast:.1}:1 (need 3:1), \
                 min pair contrast: {min_pair_contrast:.1}:1 (need 1.5:1)"
            ),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Warning
    }

    fn auto_correctable(&self) -> bool {
        true
    }
}

/// Data density: amount of data per unit of graphic.
/// Higher density (more data primitives) is generally better.
pub struct DataDensity;

impl TufteConstraint for DataDensity {
    fn name(&self) -> &'static str {
        "DataDensity"
    }

    fn evaluate(
        &self,
        primitives: &[Primitive],
        _expr: &GrammarExpr,
        _data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
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
///
/// Checks that data-carrying primitives at different data values produce
/// visually distinguishable positions. If two points representing different
/// data map to the same pixel, the visualization has lost information.
pub struct SmallestEffectiveDifference;

impl SmallestEffectiveDifference {
    const MIN_PIXEL_DISTANCE: f64 = 2.0;
}

impl TufteConstraint for SmallestEffectiveDifference {
    fn name(&self) -> &'static str {
        "SmallestEffectiveDifference"
    }

    fn evaluate(
        &self,
        primitives: &[Primitive],
        _expr: &GrammarExpr,
        _data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
        let positions: Vec<(f64, f64)> = primitives
            .iter()
            .filter(|p| p.carries_data())
            .filter_map(|p| match p {
                Primitive::Point { x, y, .. } | Primitive::Rect { x, y, .. } => Some((*x, *y)),
                _ => None,
            })
            .collect();

        if positions.len() < 2 {
            return ConstraintResult {
                passed: true,
                score: 1.0,
                message: "Fewer than 2 data points; no difference to measure".to_string(),
            };
        }

        let mut min_dist = f64::INFINITY;
        let mut overlapping_pairs = 0_usize;
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let dist = dx.hypot(dy);
                if dist < min_dist {
                    min_dist = dist;
                }
                if dist < Self::MIN_PIXEL_DISTANCE {
                    overlapping_pairs += 1;
                }
            }
        }

        let total_pairs = positions.len() * (positions.len() - 1) / 2;
        #[expect(
            clippy::cast_precision_loss,
            reason = "ratio: f64 sufficient for scoring"
        )]
        let overlap_ratio = if total_pairs > 0 {
            overlapping_pairs as f64 / total_pairs as f64
        } else {
            0.0
        };
        let passed = overlap_ratio < 0.1;
        let score = (1.0 - overlap_ratio).clamp(0.0, 1.0);

        ConstraintResult {
            passed,
            score,
            message: format!(
                "Min distance: {min_dist:.1}px, {overlapping_pairs}/{total_pairs} pairs overlap (<{:.0}px)",
                Self::MIN_PIXEL_DISTANCE
            ),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Warning
    }

    fn auto_correctable(&self) -> bool {
        false
    }
}

/// Small multiples preference: faceting over overloading.
///
/// When a grammar expression maps many categories to color or shape,
/// small multiples (faceting) typically produce clearer visualizations
/// than cramming everything into one panel. This constraint recommends
/// faceting when the category count exceeds a threshold.
pub struct SmallMultiplesPreference;

impl TufteConstraint for SmallMultiplesPreference {
    fn name(&self) -> &'static str {
        "SmallMultiplesPreference"
    }

    fn evaluate(
        &self,
        _primitives: &[Primitive],
        expr: &GrammarExpr,
        data: Option<&[serde_json::Value]>,
    ) -> ConstraintResult {
        let color_count = expr.color_category_count_with_data(data);
        let already_faceted = expr.has_facets();

        if already_faceted || color_count <= 4 {
            return ConstraintResult {
                passed: true,
                score: 1.0,
                message: if already_faceted {
                    "Already using facets (small multiples)".to_string()
                } else {
                    format!("Color categories ({color_count}) within comfortable range")
                },
            };
        }

        #[expect(clippy::cast_precision_loss, reason = "score: f64 sufficient")]
        let score = (4.0 / color_count.max(1) as f64).clamp(0.0, 1.0);
        ConstraintResult {
            passed: false,
            score,
            message: format!("{color_count} color categories: consider faceting for clarity"),
        }
    }

    fn severity(&self) -> ConstraintSeverity {
        ConstraintSeverity::Info
    }

    fn auto_correctable(&self) -> bool {
        true
    }
}
