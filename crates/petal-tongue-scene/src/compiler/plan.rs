// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render plan construction: panel bounds, axis domains, optional Tufte evaluation.

use serde_json::Value;

use crate::grammar::{GrammarExpr, ScaleType};
use crate::primitive::Primitive;
use crate::render_plan::{AxisMeta, PanelBounds, PanelMeta, RenderPlan};
use crate::tufte::{TufteConstraint, TufteReport};

use super::GrammarCompiler;
use super::utils::{x_field, y_field};

impl GrammarCompiler {
    /// Compile grammar expression and data into a `RenderPlan`.
    ///
    /// The render plan wraps the scene graph with panel bounds, axis metadata,
    /// and optional Tufte constraint results. Modality compilers and the
    /// interaction engine consume the plan for inverse transforms.
    pub fn compile_plan(
        &self,
        expr: &GrammarExpr,
        data: &[Value],
        constraints: &[&dyn TufteConstraint],
    ) -> RenderPlan {
        let graph = self.compile(expr, data);
        let axes = crate::math::Axes::default();

        let x_scale = expr
            .scales
            .iter()
            .find(|s| s.variable == "x")
            .map_or(ScaleType::Linear, |s| s.scale_type);
        let y_scale = expr
            .scales
            .iter()
            .find(|s| s.variable == "y")
            .map_or(ScaleType::Linear, |s| s.scale_type);

        let (x_min, x_max) = Self::data_extent(data, x_field(expr));
        let (y_min, y_max) = Self::data_extent(data, y_field(expr));

        let panel = PanelMeta::new(
            "main",
            PanelBounds::new(
                axes.origin.0,
                axes.origin.1 - axes.height,
                axes.width,
                axes.height,
            ),
        )
        .with_axis(
            AxisMeta::new("x", x_scale)
                .with_domain(x_min, x_max)
                .with_range(axes.origin.0, axes.origin.0 + axes.width),
        )
        .with_axis(
            AxisMeta::new("y", y_scale)
                .with_domain(y_min, y_max)
                .with_range(axes.origin.1, axes.origin.1 - axes.height),
        );

        let mut plan = RenderPlan::new(graph, expr.clone()).with_panel(panel);

        if !constraints.is_empty() {
            let primitives: Vec<Primitive> = plan
                .scene
                .flatten()
                .into_iter()
                .map(|(_, p)| p.clone())
                .collect();
            let report = TufteReport::evaluate_all(constraints, &primitives, expr, Some(data));
            plan = plan.with_constraints_report(report);
        }

        plan
    }

    fn data_extent(data: &[Value], field: Option<&str>) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for (i, obj) in data.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "fallback index: f64 sufficient")]
            let v = obj
                .as_object()
                .and_then(|o| field.and_then(|f| super::utils::get_number(o, f)))
                .unwrap_or(i as f64);
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        if min.is_infinite() {
            (0.0, 1.0)
        } else {
            (min, max)
        }
    }
}
