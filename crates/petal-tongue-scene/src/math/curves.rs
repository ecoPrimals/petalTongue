// SPDX-License-Identifier: AGPL-3.0-only
//! Curve types: `FunctionPlot` (y=f(x)) and `ParametricCurve` ((x(t), y(t))).

use serde::{Deserialize, Serialize};

use crate::math::{Axes, MathObject};
use crate::primitive::{Primitive, StrokeStyle};

/// A function plot: y = f(x) sampled over an axes domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPlot {
    /// Sampled points (x, y) in data coordinates.
    pub points: Vec<[f64; 2]>,
    /// X range (min, max) for the plot.
    pub x_range: (f64, f64),
    /// Y range (min, max) for the plot.
    pub y_range: (f64, f64),
    /// Origin and dimensions for coordinate mapping.
    pub origin: (f64, f64),
    pub width: f64,
    pub height: f64,
    /// Stroke style for the curve.
    pub stroke: StrokeStyle,
    /// Number of samples used.
    pub num_samples: usize,
}

impl FunctionPlot {
    /// Sample a function over the axes domain.
    pub fn sample(axes: &Axes, f: impl Fn(f64) -> f64, stroke: StrokeStyle) -> Self {
        let num_samples = 200;
        let (x_min, x_max, _) = axes.x_range;
        let (y_min, y_max, _) = axes.y_range;
        let mut points = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f64 / (num_samples - 1) as f64;
            let x = x_min + t * (x_max - x_min);
            let y = f(x);
            points.push([x, y]);
        }
        Self {
            points,
            x_range: (x_min, x_max),
            y_range: (y_min, y_max),
            origin: axes.origin,
            width: axes.width,
            height: axes.height,
            stroke,
            num_samples,
        }
    }

    /// Create from pre-computed points.
    #[must_use]
    pub const fn from_points(points: Vec<[f64; 2]>, axes: &Axes, stroke: StrokeStyle) -> Self {
        let (x_min, x_max, _) = axes.x_range;
        let (y_min, y_max, _) = axes.y_range;
        Self {
            points,
            x_range: (x_min, x_max),
            y_range: (y_min, y_max),
            origin: axes.origin,
            width: axes.width,
            height: axes.height,
            stroke,
            num_samples: 0,
        }
    }

    fn data_to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let (x_min, x_max) = self.x_range;
        let (y_min, y_max) = self.y_range;
        let tx = (x - x_min) / (x_max - x_min);
        let ty = (y - y_min) / (y_max - y_min);
        let sx = tx.mul_add(self.width, self.origin.0);
        let sy = ty.mul_add(-self.height, self.origin.1);
        (sx, sy)
    }
}

impl MathObject for FunctionPlot {
    fn to_primitives(&self) -> Vec<Primitive> {
        let screen_points: Vec<[f64; 2]> = self
            .points
            .iter()
            .map(|&[x, y]| {
                let (sx, sy) = self.data_to_screen(x, y);
                [sx, sy]
            })
            .collect();
        vec![Primitive::Line {
            points: screen_points,
            stroke: self.stroke,
            closed: false,
            data_id: None,
        }]
    }
}

/// A parametric curve: (x(t), y(t)).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametricCurve {
    /// Pre-sampled points in data coordinates.
    pub points: Vec<[f64; 2]>,
    /// Stroke style.
    pub stroke: StrokeStyle,
    /// Whether the curve is closed (connect last to first).
    pub closed: bool,
}

impl ParametricCurve {
    /// Sample a parametric curve.
    pub fn sample(
        x_fn: impl Fn(f64) -> f64,
        y_fn: impl Fn(f64) -> f64,
        t_range: (f64, f64),
        num_samples: usize,
        stroke: StrokeStyle,
    ) -> Self {
        let (t0, t1) = t_range;
        let mut points = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = (t1 - t0).mul_add(i as f64 / (num_samples - 1) as f64, t0);
            points.push([x_fn(t), y_fn(t)]);
        }
        Self {
            points,
            stroke,
            closed: false,
        }
    }
}

impl MathObject for ParametricCurve {
    fn to_primitives(&self) -> Vec<Primitive> {
        vec![Primitive::Line {
            points: self.points.clone(),
            stroke: self.stroke,
            closed: self.closed,
            data_id: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::StrokeStyle;

    #[test]
    fn function_plot_sample_linear() {
        let axes = Axes::default();
        let plot = FunctionPlot::sample(&axes, |x| x, StrokeStyle::default());
        assert_eq!(plot.points.len(), 200);
        assert!((plot.points[0][1] - plot.points[0][0]).abs() < 1e-6);
        assert!((plot.points[199][1] - plot.points[199][0]).abs() < 1e-6);
    }

    #[test]
    fn function_plot_sample_constant() {
        let axes = Axes::default();
        let plot = FunctionPlot::sample(&axes, |_| 42.0, StrokeStyle::default());
        assert_eq!(plot.points.len(), 200);
        for pt in &plot.points {
            assert!((pt[1] - 42.0).abs() < 1e-6);
        }
    }

    #[test]
    fn parametric_curve_sample_line() {
        let curve =
            ParametricCurve::sample(|t| t, |t| t * 2.0, (0.0, 1.0), 11, StrokeStyle::default());
        assert_eq!(curve.points.len(), 11);
        assert!((curve.points[0][0] - 0.0).abs() < 1e-10);
        assert!((curve.points[10][0] - 1.0).abs() < 1e-10);
        assert!((curve.points[0][1] - 0.0).abs() < 1e-10);
        assert!((curve.points[10][1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn parametric_curve_sample_circle() {
        let curve = ParametricCurve::sample(
            f64::cos,
            f64::sin,
            (0.0, std::f64::consts::TAU),
            64,
            StrokeStyle::default(),
        );
        assert_eq!(curve.points.len(), 64);
        let first = curve.points[0];
        let last = curve.points[63];
        assert!((first[0] - last[0]).abs() < 0.1);
        assert!((first[1] - last[1]).abs() < 0.1);
    }

    #[test]
    fn function_plot_from_points_to_primitives() {
        let axes = Axes::default();
        let points = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 4.0]];
        let plot = FunctionPlot::from_points(points, &axes, StrokeStyle::default());
        let prims = plot.to_primitives();
        assert_eq!(prims.len(), 1);
        if let Primitive::Line { points: pts, .. } = &prims[0] {
            assert_eq!(pts.len(), 3);
        }
    }

    #[test]
    fn parametric_curve_to_primitives_closed() {
        let curve = ParametricCurve {
            points: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]],
            stroke: StrokeStyle::default(),
            closed: true,
        };
        let prims = curve.to_primitives();
        assert_eq!(prims.len(), 1);
        if let Primitive::Line { closed, .. } = &prims[0] {
            assert!(*closed);
        }
    }

    #[test]
    fn function_plot_serialization_roundtrip() {
        let axes = Axes::default();
        let plot = FunctionPlot::sample(&axes, |x| x * x, StrokeStyle::default());
        let json = serde_json::to_string(&plot).expect("serialize");
        let restored: FunctionPlot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(plot.points.len(), restored.points.len());
        assert!((plot.x_range.0 - restored.x_range.0).abs() < 1e-10);
    }

    #[test]
    fn parametric_curve_serialization_roundtrip() {
        let curve = ParametricCurve::sample(|t| t, |t| t, (0.0, 1.0), 5, StrokeStyle::default());
        let json = serde_json::to_string(&curve).expect("serialize");
        let restored: ParametricCurve = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(curve.points.len(), restored.points.len());
    }
}
