// SPDX-License-Identifier: AGPL-3.0-only
//! Mathematical objects that compile to rendering primitives.
//!
//! These are the Manim-equivalent layer: axes, function plots, parametric curves,
//! vector fields, number lines. Each implements `MathObject` to produce `Primitive`
//! collections that any modality compiler can render.

use serde::{Deserialize, Serialize};

use crate::primitive::{AnchorPoint, Color, FillRule, LineCap, LineJoin, Primitive, StrokeStyle};

/// Trait for mathematical objects that compile to rendering primitives.
pub trait MathObject {
    /// Produce a collection of primitives for rendering.
    fn to_primitives(&self) -> Vec<Primitive>;
}

/// A number line: axis with tick marks and optional labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberLine {
    /// Start value (data space).
    pub start: f64,
    /// End value (data space).
    pub end: f64,
    /// Step between ticks.
    pub step: f64,
    /// Origin x in screen coordinates.
    pub origin_x: f64,
    /// Origin y in screen coordinates.
    pub origin_y: f64,
    /// Pixel length of the axis.
    pub length: f64,
    /// Color for axis and ticks.
    pub color: Color,
    /// Whether to show numeric labels.
    pub show_labels: bool,
    /// Font size for labels.
    pub label_font_size: f64,
}

impl Default for NumberLine {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 10.0,
            step: 1.0,
            origin_x: 0.0,
            origin_y: 0.0,
            length: 400.0,
            color: Color::BLACK,
            show_labels: true,
            label_font_size: 12.0,
        }
    }
}

impl NumberLine {
    /// Map a data value to screen x (horizontal number line).
    fn data_to_screen_x(&self, value: f64) -> f64 {
        let t = (value - self.start) / (self.end - self.start);
        self.origin_x + t * self.length
    }
}

impl MathObject for NumberLine {
    fn to_primitives(&self) -> Vec<Primitive> {
        let mut prims = Vec::new();
        let stroke = StrokeStyle {
            color: self.color,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        // Axis line (horizontal)
        let x0 = self.origin_x;
        let x1 = self.origin_x + self.length;
        prims.push(Primitive::Line {
            points: vec![[x0, self.origin_y], [x1, self.origin_y]],
            stroke,
            closed: false,
            data_id: None,
        });

        // Tick marks
        let mut v = self.start;
        while v <= self.end {
            let sx = self.data_to_screen_x(v);
            let tick_len = 5.0;
            prims.push(Primitive::Line {
                points: vec![[sx, self.origin_y], [sx, self.origin_y - tick_len]],
                stroke,
                closed: false,
                data_id: None,
            });
            v += self.step;
        }

        // Labels
        if self.show_labels {
            let mut v = self.start;
            while v <= self.end {
                let sx = self.data_to_screen_x(v);
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "axis labels are integer tick values"
                )]
                let label = if (v - v.round()).abs() < 1e-10 {
                    format!("{}", v.round() as i64)
                } else {
                    format!("{v:.2}")
                };
                prims.push(Primitive::Text {
                    x: sx,
                    y: self.origin_y - 8.0,
                    content: label,
                    font_size: self.label_font_size,
                    color: self.color,
                    anchor: AnchorPoint::TopCenter,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
                v += self.step;
            }
        }

        prims
    }
}

/// 2D axes with x and y number lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axes {
    /// X range: (min, max, step).
    pub x_range: (f64, f64, f64),
    /// Y range: (min, max, step).
    pub y_range: (f64, f64, f64),
    /// Origin in screen coordinates.
    pub origin: (f64, f64),
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
    /// Color for axes.
    pub color: Color,
    /// Whether to show labels.
    pub show_labels: bool,
    /// Font size for labels.
    pub label_font_size: f64,
}

impl Default for Axes {
    fn default() -> Self {
        Self {
            x_range: (-10.0, 10.0, 2.0),
            y_range: (-10.0, 10.0, 2.0),
            origin: (200.0, 200.0),
            width: 400.0,
            height: 400.0,
            color: Color::BLACK,
            show_labels: true,
            label_font_size: 12.0,
        }
    }
}

impl Axes {
    /// Map data coordinates (x, y) to screen coordinates.
    pub fn data_to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let (x_min, x_max, _) = self.x_range;
        let (y_min, y_max, _) = self.y_range;
        let tx = (x - x_min) / (x_max - x_min);
        let ty = (y - y_min) / (y_max - y_min);
        let sx = self.origin.0 + tx * self.width;
        let sy = self.origin.1 - ty * self.height; // y flipped (screen y down)
        (sx, sy)
    }

    /// Map screen coordinates to data coordinates.
    pub fn screen_to_data(&self, sx: f64, sy: f64) -> (f64, f64) {
        let (x_min, x_max, _) = self.x_range;
        let (y_min, y_max, _) = self.y_range;
        let tx = (sx - self.origin.0) / self.width;
        let ty = (sy - self.origin.1) / -self.height;
        let x = x_min + tx * (x_max - x_min);
        let y = y_min + ty * (y_max - y_min);
        (x, y)
    }
}

impl MathObject for Axes {
    #[expect(
        clippy::too_many_lines,
        reason = "axis rendering is a cohesive sequence: x-axis, y-axis, ticks, labels, gridlines"
    )]
    fn to_primitives(&self) -> Vec<Primitive> {
        let mut prims = Vec::new();
        let stroke = StrokeStyle {
            color: self.color,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let (x_min, x_max, x_step) = self.x_range;
        let (y_min, y_max, y_step) = self.y_range;
        let (ox, oy) = self.origin;

        // X axis (horizontal)
        prims.push(Primitive::Line {
            points: vec![[ox, oy], [ox + self.width, oy]],
            stroke,
            closed: false,
            data_id: None,
        });
        // X ticks
        let mut v = x_min;
        while v <= x_max {
            let (sx, _) = self.data_to_screen(v, 0.0);
            prims.push(Primitive::Line {
                points: vec![[sx, oy], [sx, oy + 5.0]],
                stroke,
                closed: false,
                data_id: None,
            });
            if self.show_labels {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "axis labels are integer tick values"
                )]
                prims.push(Primitive::Text {
                    x: sx,
                    y: oy + 8.0,
                    content: format!("{}", v.round() as i64),
                    font_size: self.label_font_size,
                    color: self.color,
                    anchor: AnchorPoint::TopCenter,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
            }
            v += x_step;
        }
        // X arrow head
        let arrow_size = 8.0;
        let (sx_end, _) = self.data_to_screen(x_max, 0.0);
        prims.push(Primitive::Line {
            points: vec![[sx_end, oy], [sx_end - arrow_size, oy - arrow_size * 0.5]],
            stroke,
            closed: false,
            data_id: None,
        });
        prims.push(Primitive::Line {
            points: vec![[sx_end, oy], [sx_end - arrow_size, oy + arrow_size * 0.5]],
            stroke,
            closed: false,
            data_id: None,
        });

        // Y axis (vertical)
        prims.push(Primitive::Line {
            points: vec![[ox, oy], [ox, oy - self.height]],
            stroke,
            closed: false,
            data_id: None,
        });
        // Y ticks
        let mut v = y_min;
        while v <= y_max {
            let (_, sy) = self.data_to_screen(0.0, v);
            prims.push(Primitive::Line {
                points: vec![[ox, sy], [ox - 5.0, sy]],
                stroke,
                closed: false,
                data_id: None,
            });
            if self.show_labels {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "axis labels are integer tick values"
                )]
                prims.push(Primitive::Text {
                    x: ox - 8.0,
                    y: sy,
                    content: format!("{}", v.round() as i64),
                    font_size: self.label_font_size,
                    color: self.color,
                    anchor: AnchorPoint::CenterRight,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
            }
            v += y_step;
        }
        // Y arrow head
        let (_, sy_end) = self.data_to_screen(0.0, y_max);
        prims.push(Primitive::Line {
            points: vec![[ox, sy_end], [ox - arrow_size * 0.5, sy_end + arrow_size]],
            stroke,
            closed: false,
            data_id: None,
        });
        prims.push(Primitive::Line {
            points: vec![[ox, sy_end], [ox + arrow_size * 0.5, sy_end + arrow_size]],
            stroke,
            closed: false,
            data_id: None,
        });

        prims
    }
}

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

/// A single arrow in a vector field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowSpec {
    /// Start point (x, y).
    pub start: [f64; 2],
    /// Direction vector (dx, dy).
    pub direction: [f64; 2],
}

/// A vector field: arrows at grid points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorField {
    /// Arrows: start + direction.
    pub arrows: Vec<ArrowSpec>,
    /// Scale factor for arrow length.
    pub scale: f64,
    /// Color for arrows.
    pub color: Color,
}

impl VectorField {
    /// Build from a field function F(x,y) -> (dx, dy).
    pub fn from_fn(
        field: impl Fn(f64, f64) -> (f64, f64),
        x_range: (f64, f64),
        y_range: (f64, f64),
        density: usize,
        scale: f64,
        color: Color,
    ) -> Self {
        let (x_min, x_max) = x_range;
        let (y_min, y_max) = y_range;
        let mut arrows = Vec::new();
        for i in 0..density {
            for j in 0..density {
                let t_x = (i as f64 + 0.5) / density as f64;
                let t_y = (j as f64 + 0.5) / density as f64;
                let x = x_min + t_x * (x_max - x_min);
                let y = y_min + t_y * (y_max - y_min);
                let (dx, dy) = field(x, y);
                arrows.push(ArrowSpec {
                    start: [x, y],
                    direction: [dx * scale, dy * scale],
                });
            }
        }
        Self {
            arrows,
            scale,
            color,
        }
    }
}

impl MathObject for VectorField {
    fn to_primitives(&self) -> Vec<Primitive> {
        let mut prims = Vec::new();
        let stroke = StrokeStyle {
            color: self.color,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let head_size = 4.0;
        for arrow in &self.arrows {
            let [sx, sy] = arrow.start;
            let [dx, dy] = arrow.direction;
            let ex = sx + dx;
            let ey = sy + dy;

            // Shaft
            prims.push(Primitive::Line {
                points: vec![[sx, sy], [ex, ey]],
                stroke,
                closed: false,
                data_id: None,
            });

            // Arrow head (triangle)
            let len = (dx * dx + dy * dy).sqrt();
            if len > 1e-10 {
                let ux = dx / len;
                let uy = dy / len;
                let perp_x = -uy;
                let perp_y = ux;
                let tip = [ex, ey];
                let base1 = [
                    ex - ux * head_size + perp_x * head_size * 0.5,
                    ey - uy * head_size + perp_y * head_size * 0.5,
                ];
                let base2 = [
                    ex - ux * head_size - perp_x * head_size * 0.5,
                    ey - uy * head_size - perp_y * head_size * 0.5,
                ];
                prims.push(Primitive::Polygon {
                    points: vec![tip, base1, base2],
                    fill: self.color,
                    stroke: None,
                    fill_rule: FillRule::NonZero,
                    data_id: None,
                });
            }
        }

        prims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numberline_default_produces_axis_ticks_labels() {
        let nl = NumberLine::default();
        let prims = nl.to_primitives();
        assert!(!prims.is_empty());
        let has_line = prims.iter().any(|p| matches!(p, Primitive::Line { .. }));
        let has_text = prims.iter().any(|p| matches!(p, Primitive::Text { .. }));
        assert!(has_line);
        assert!(has_text);
    }

    #[test]
    fn axes_data_to_screen_roundtrip() {
        let axes = Axes::default();
        let (x, y) = (3.0, -5.0);
        let (sx, sy) = axes.data_to_screen(x, y);
        let (x2, y2) = axes.screen_to_data(sx, sy);
        assert!((x - x2).abs() < 1e-10);
        assert!((y - y2).abs() < 1e-10);
    }

    #[test]
    fn function_plot_sample_produces_correct_points() {
        let axes = Axes::default();
        let plot = FunctionPlot::sample(&axes, |x| x * x, StrokeStyle::default());
        assert_eq!(plot.points.len(), 200);
        assert!((plot.points[0][0] - (-10.0)).abs() < 1e-6);
        assert!((plot.points[199][0] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn parametric_curve_circle_closed() {
        let curve = ParametricCurve::sample(
            f64::cos,
            f64::sin,
            (0.0, std::f64::consts::TAU - 0.01),
            64,
            StrokeStyle::default(),
        );
        let prims = curve.to_primitives();
        assert_eq!(prims.len(), 1);
        if let Primitive::Line { points, closed, .. } = &prims[0] {
            assert_eq!(points.len(), 64);
            let first = points[0];
            let last = points[63];
            assert!((first[0] - last[0]).abs() < 0.2);
            assert!((first[1] - last[1]).abs() < 0.2);
            assert!(!closed);
        }
    }

    #[test]
    fn vector_field_produces_arrows() {
        let vf = VectorField::from_fn(
            |x, y| (x, y),
            (-1.0, 1.0),
            (-1.0, 1.0),
            3,
            0.2,
            Color::BLACK,
        );
        let prims = vf.to_primitives();
        assert!(!prims.is_empty());
        let has_line = prims.iter().any(|p| matches!(p, Primitive::Line { .. }));
        let has_poly = prims.iter().any(|p| matches!(p, Primitive::Polygon { .. }));
        assert!(has_line);
        assert!(has_poly);
    }

    #[test]
    fn math_object_trait_object_safe() {
        fn collect_primitives(obj: &dyn MathObject) -> Vec<Primitive> {
            obj.to_primitives()
        }
        let nl = NumberLine::default();
        let prims = collect_primitives(&nl);
        assert!(!prims.is_empty());

        let boxed: Box<dyn MathObject> = Box::new(NumberLine::default());
        let prims2 = boxed.to_primitives();
        assert!(!prims2.is_empty());
    }

    #[test]
    fn numberline_custom_range_and_tick_count() {
        let nl = NumberLine {
            start: -5.0,
            end: 5.0,
            step: 0.5,
            origin_x: 100.0,
            origin_y: 200.0,
            length: 200.0,
            color: Color::BLACK,
            show_labels: true,
            label_font_size: 10.0,
        };
        let prims = nl.to_primitives();
        assert!(!prims.is_empty());
        let tick_count = prims
            .iter()
            .filter(|p| matches!(p, Primitive::Line { .. }))
            .count();
        assert!(
            tick_count >= 20,
            "Should have ~21 ticks from -5 to 5 by 0.5"
        );
    }

    #[test]
    fn numberline_no_labels_produces_no_text() {
        let nl = NumberLine {
            show_labels: false,
            ..NumberLine::default()
        };
        let prims = nl.to_primitives();
        let has_text = prims.iter().any(|p| matches!(p, Primitive::Text { .. }));
        assert!(!has_text);
    }

    #[test]
    fn axes_various_aspect_ratios() {
        let axes_square = Axes {
            width: 400.0,
            height: 400.0,
            ..Axes::default()
        };
        let (sx, sy) = axes_square.data_to_screen(0.0, 0.0);
        assert!((sx - 400.0).abs() < 1e-10);
        assert!((sy - 0.0).abs() < 1e-10); // y flipped: origin.1 - ty*height

        let axes_wide = Axes {
            width: 800.0,
            height: 200.0,
            ..Axes::default()
        };
        let (sx2, sy2) = axes_wide.data_to_screen(0.0, 0.0);
        assert!((sx2 - 600.0).abs() < 1e-10); // origin.0 + 0.5*800
        assert!((sy2 - 100.0).abs() < 1e-10); // origin.1 - 0.5*200

        let axes_tall = Axes {
            width: 200.0,
            height: 600.0,
            ..Axes::default()
        };
        let prims = axes_tall.to_primitives();
        assert!(!prims.is_empty());
    }

    #[test]
    fn function_plot_nan_and_infinity_handling() {
        let axes = Axes {
            x_range: (-1.0, 1.0, 0.5),
            y_range: (-10.0, 10.0, 2.0),
            ..Axes::default()
        };
        let plot = FunctionPlot::sample(&axes, |x| 1.0 / (x - 1.0), StrokeStyle::default());
        assert_eq!(plot.points.len(), 200);
        let has_special = plot
            .points
            .iter()
            .any(|[_, y]| y.is_nan() || y.is_infinite());
        assert!(has_special, "1/(x-1) at x=1 produces infinity");
    }

    #[test]
    fn function_plot_very_large_range() {
        let axes = Axes {
            x_range: (-1e6, 1e6, 1e5),
            y_range: (-1e6, 1e6, 1e5),
            ..Axes::default()
        };
        let plot = FunctionPlot::sample(&axes, |x| x, StrokeStyle::default());
        assert_eq!(plot.points.len(), 200);
        assert!((plot.points[0][0] - (-1e6)).abs() < 1e3);
        assert!((plot.points[199][0] - 1e6).abs() < 1e3);
    }

    #[test]
    fn function_plot_from_points_edge_case_empty() {
        let axes = Axes::default();
        let plot = FunctionPlot::from_points(vec![], &axes, StrokeStyle::default());
        let prims = plot.to_primitives();
        assert_eq!(prims.len(), 1);
        if let Primitive::Line { points, .. } = &prims[0] {
            assert!(points.is_empty());
        }
    }

    #[test]
    fn vector_field_various_grid_sizes() {
        for density in [1, 2, 5, 10] {
            let vf = VectorField::from_fn(
                |x, y| (x + y, x - y),
                (-1.0, 1.0),
                (-1.0, 1.0),
                density,
                0.1,
                Color::BLACK,
            );
            assert_eq!(vf.arrows.len(), density * density);
            let prims = vf.to_primitives();
            assert!(!prims.is_empty());
        }
    }

    #[test]
    fn vector_field_empty_density_zero() {
        let vf = VectorField::from_fn(
            |_, _| (0.0, 0.0),
            (0.0, 1.0),
            (0.0, 1.0),
            0,
            1.0,
            Color::BLACK,
        );
        assert!(vf.arrows.is_empty());
        let prims = vf.to_primitives();
        assert!(prims.is_empty());
    }

    #[test]
    fn parametric_curve_zero_range() {
        let curve = ParametricCurve::sample(|t| t, |t| t, (1.0, 1.0), 10, StrokeStyle::default());
        assert_eq!(curve.points.len(), 10);
        let first = curve.points[0];
        let last = curve.points[9];
        assert!((first[0] - 1.0).abs() < 1e-10);
        assert!((last[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn parametric_curve_single_point() {
        let curve = ParametricCurve {
            points: vec![[0.0, 0.0]],
            stroke: StrokeStyle::default(),
            closed: false,
        };
        let prims = curve.to_primitives();
        assert_eq!(prims.len(), 1);
        if let Primitive::Line { points, .. } = &prims[0] {
            assert_eq!(points.len(), 1);
        }
    }

    #[test]
    fn parametric_curve_closed_produces_line() {
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
}
