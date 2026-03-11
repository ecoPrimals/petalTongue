// SPDX-License-Identifier: AGPL-3.0-only
//! Curve types: FunctionPlot (y=f(x)) and ParametricCurve ((x(t), y(t))).

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
    pub fn from_points(points: Vec<[f64; 2]>, axes: &Axes, stroke: StrokeStyle) -> Self {
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
        let sx = self.origin.0 + tx * self.width;
        let sy = self.origin.1 - ty * self.height;
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
            let t = t0 + (t1 - t0) * (i as f64 / (num_samples - 1) as f64);
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
