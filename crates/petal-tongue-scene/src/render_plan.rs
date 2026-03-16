// SPDX-License-Identifier: AGPL-3.0-or-later
//! `RenderPlan`: the intermediate representation between grammar compilation
//! and modality dispatch.
//!
//! A `RenderPlan` wraps a `SceneGraph` with the metadata that modality
//! compilers, Tufte constraints, and the interaction engine's inverse
//! pipeline need: axis/scale information, panel bounds, data-to-visual
//! mappings, and the original grammar expression.

use serde::{Deserialize, Serialize};

use crate::grammar::{GrammarExpr, ScaleType};
use crate::scene_graph::SceneGraph;
use crate::tufte::TufteReport;

/// A compiled visualization ready for modality dispatch.
///
/// Produced by `GrammarCompiler::compile_plan`. Consumed by
/// `ModalityCompiler::compile_plan` and `TufteConstraint::evaluate_plan`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderPlan {
    pub scene: SceneGraph,
    pub panels: Vec<PanelMeta>,
    pub grammar: GrammarExpr,
    pub constraints_report: Option<TufteReport>,
}

impl RenderPlan {
    #[must_use]
    pub const fn new(scene: SceneGraph, grammar: GrammarExpr) -> Self {
        Self {
            scene,
            panels: Vec::new(),
            grammar,
            constraints_report: None,
        }
    }

    #[must_use]
    pub fn with_panel(mut self, panel: PanelMeta) -> Self {
        self.panels.push(panel);
        self
    }

    #[must_use]
    pub fn with_constraints_report(mut self, report: TufteReport) -> Self {
        self.constraints_report = Some(report);
        self
    }

    /// Overall Tufte score (1.0 if no report).
    #[must_use]
    pub fn tufte_score(&self) -> f64 {
        self.constraints_report
            .as_ref()
            .map_or(1.0, |r| r.overall_score)
    }

    /// Whether a bar/area geometry is present (for lie factor checks).
    #[must_use]
    pub const fn has_bar_or_area_geom(&self) -> bool {
        use crate::grammar::GeometryType;
        matches!(
            self.grammar.geometry,
            GeometryType::Bar | GeometryType::Area
        )
    }

    /// Y-axis domain minimum from the first panel's y-axis scale, if any.
    #[must_use]
    pub fn y_domain_min(&self) -> Option<f64> {
        self.panels
            .first()
            .and_then(|p| p.axes.iter().find(|a| a.variable == "y"))
            .map(|a| a.domain_min)
    }

    /// Y-axis domain maximum from the first panel's y-axis scale, if any.
    #[must_use]
    pub fn y_domain_max(&self) -> Option<f64> {
        self.panels
            .first()
            .and_then(|p| p.axes.iter().find(|a| a.variable == "y"))
            .map(|a| a.domain_max)
    }

    /// Whether a size aesthetic is mapped.
    #[must_use]
    pub fn has_size_aesthetic(&self) -> bool {
        self.grammar
            .aesthetics
            .iter()
            .any(|a| matches!(a, crate::grammar::Aesthetic::Size(_)))
    }

    /// Grid line density (lines per axis, max across panels).
    #[must_use]
    pub fn grid_line_density(&self) -> usize {
        self.panels
            .iter()
            .map(|p| p.grid_lines_per_axis)
            .max()
            .unwrap_or(0)
    }

    /// Whether any panel uses dual Y axes.
    #[must_use]
    pub fn has_dual_y_axes(&self) -> bool {
        self.panels.iter().any(|p| p.dual_y_axes)
    }
}

/// Metadata for a single panel (facet) within the render plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelMeta {
    pub id: String,
    pub bounds: PanelBounds,
    pub axes: Vec<AxisMeta>,
    pub grid_lines_per_axis: usize,
    pub dual_y_axes: bool,
    pub title: Option<String>,
}

impl PanelMeta {
    pub fn new(id: impl Into<String>, bounds: PanelBounds) -> Self {
        Self {
            id: id.into(),
            bounds,
            axes: Vec::new(),
            grid_lines_per_axis: 5,
            dual_y_axes: false,
            title: None,
        }
    }

    #[must_use]
    pub fn with_axis(mut self, axis: AxisMeta) -> Self {
        self.axes.push(axis);
        self
    }
}

/// Bounding rectangle for a panel in scene coordinates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PanelBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PanelBounds {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Metadata for an axis: variable, scale type, domain, range, and inverse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMeta {
    pub variable: String,
    pub scale_type: ScaleType,
    pub domain_min: f64,
    pub domain_max: f64,
    pub range_min: f64,
    pub range_max: f64,
}

impl AxisMeta {
    pub fn new(variable: impl Into<String>, scale_type: ScaleType) -> Self {
        Self {
            variable: variable.into(),
            scale_type,
            domain_min: 0.0,
            domain_max: 1.0,
            range_min: 0.0,
            range_max: 1.0,
        }
    }

    #[must_use]
    pub const fn with_domain(mut self, min: f64, max: f64) -> Self {
        self.domain_min = min;
        self.domain_max = max;
        self
    }

    #[must_use]
    pub const fn with_range(mut self, min: f64, max: f64) -> Self {
        self.range_min = min;
        self.range_max = max;
        self
    }

    /// Forward transform: data value -> visual coordinate.
    #[must_use]
    pub fn transform(&self, value: f64) -> f64 {
        let domain_span = self.domain_max - self.domain_min;
        if domain_span.abs() < f64::EPSILON {
            return self.range_min;
        }
        let t = match self.scale_type {
            ScaleType::Linear
            | ScaleType::Temporal
            | ScaleType::Ordinal
            | ScaleType::Categorical => (value - self.domain_min) / domain_span,
            ScaleType::Log => {
                let log_min = self.domain_min.max(f64::EPSILON).ln();
                let log_max = self.domain_max.max(f64::EPSILON).ln();
                let log_span = log_max - log_min;
                if log_span.abs() < f64::EPSILON {
                    0.0
                } else {
                    (value.max(f64::EPSILON).ln() - log_min) / log_span
                }
            }
            ScaleType::Sqrt => {
                let sqrt_min = self.domain_min.max(0.0).sqrt();
                let sqrt_max = self.domain_max.max(0.0).sqrt();
                let sqrt_span = sqrt_max - sqrt_min;
                if sqrt_span.abs() < f64::EPSILON {
                    0.0
                } else {
                    (value.max(0.0).sqrt() - sqrt_min) / sqrt_span
                }
            }
        };
        self.range_min + t * (self.range_max - self.range_min)
    }

    /// Inverse transform: visual coordinate -> data value.
    #[must_use]
    pub fn inverse(&self, visual: f64) -> f64 {
        let range_span = self.range_max - self.range_min;
        if range_span.abs() < f64::EPSILON {
            return self.domain_min;
        }
        let t = (visual - self.range_min) / range_span;
        match self.scale_type {
            ScaleType::Linear
            | ScaleType::Temporal
            | ScaleType::Ordinal
            | ScaleType::Categorical => {
                t.mul_add(self.domain_max - self.domain_min, self.domain_min)
            }
            ScaleType::Log => {
                let log_min = self.domain_min.max(f64::EPSILON).ln();
                let log_max = self.domain_max.max(f64::EPSILON).ln();
                t.mul_add(log_max - log_min, log_min).exp()
            }
            ScaleType::Sqrt => {
                let sqrt_min = self.domain_min.max(0.0).sqrt();
                let sqrt_max = self.domain_max.max(0.0).sqrt();
                let v = t.mul_add(sqrt_max - sqrt_min, sqrt_min);
                v * v
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::GeometryType;

    #[test]
    fn axis_linear_roundtrip() {
        let axis = AxisMeta::new("x", ScaleType::Linear)
            .with_domain(0.0, 100.0)
            .with_range(50.0, 750.0);
        for v in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let visual = axis.transform(v);
            let back = axis.inverse(visual);
            assert!(
                (back - v).abs() < 1e-10,
                "roundtrip failed for {v}: got {back}"
            );
        }
    }

    #[test]
    fn axis_log_roundtrip() {
        let axis = AxisMeta::new("y", ScaleType::Log)
            .with_domain(1.0, 1000.0)
            .with_range(0.0, 600.0);
        for v in [1.0, 10.0, 100.0, 1000.0] {
            let visual = axis.transform(v);
            let back = axis.inverse(visual);
            assert!(
                (back - v).abs() / v < 1e-9,
                "log roundtrip failed for {v}: got {back}"
            );
        }
    }

    #[test]
    fn axis_sqrt_roundtrip() {
        let axis = AxisMeta::new("size", ScaleType::Sqrt)
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);
        for v in [0.0, 25.0, 50.0, 100.0] {
            let visual = axis.transform(v);
            let back = axis.inverse(visual);
            assert!(
                (back - v).abs() < 1e-9,
                "sqrt roundtrip failed for {v}: got {back}"
            );
        }
    }

    #[test]
    fn panel_bounds_contains() {
        let b = PanelBounds::new(10.0, 20.0, 100.0, 50.0);
        assert!(b.contains(50.0, 40.0));
        assert!(!b.contains(5.0, 40.0));
        assert!(!b.contains(50.0, 80.0));
    }

    #[test]
    fn render_plan_creation() {
        let scene = SceneGraph::new();
        let grammar = GrammarExpr::new("test", GeometryType::Point);
        let plan = RenderPlan::new(scene, grammar).with_panel(
            PanelMeta::new("main", PanelBounds::new(50.0, 50.0, 700.0, 500.0))
                .with_axis(
                    AxisMeta::new("x", ScaleType::Linear)
                        .with_domain(0.0, 100.0)
                        .with_range(50.0, 750.0),
                )
                .with_axis(
                    AxisMeta::new("y", ScaleType::Linear)
                        .with_domain(0.0, 50.0)
                        .with_range(550.0, 50.0),
                ),
        );
        assert_eq!(plan.panels.len(), 1);
        assert_eq!(plan.panels[0].axes.len(), 2);
        assert!((plan.tufte_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn render_plan_y_domain() {
        let scene = SceneGraph::new();
        let grammar = GrammarExpr::new("test", GeometryType::Bar);
        let plan = RenderPlan::new(scene, grammar).with_panel(
            PanelMeta::new("main", PanelBounds::new(0.0, 0.0, 800.0, 600.0)).with_axis(
                AxisMeta::new("y", ScaleType::Linear)
                    .with_domain(10.0, 100.0)
                    .with_range(0.0, 600.0),
            ),
        );
        assert_eq!(plan.y_domain_min(), Some(10.0));
        assert_eq!(plan.y_domain_max(), Some(100.0));
        assert!(plan.has_bar_or_area_geom());
    }
}
