// SPDX-License-Identifier: AGPL-3.0-only
//! Mathematical objects that compile to rendering primitives.
//!
//! These are the Manim-equivalent layer: axes, function plots, parametric curves,
//! vector fields, number lines. Each implements `MathObject` to produce `Primitive`
//! collections that any modality compiler can render.

mod axes;
mod curves;
mod vector_field;

pub use axes::{Axes, NumberLine};
pub use curves::{FunctionPlot, ParametricCurve};
pub use vector_field::{ArrowSpec, VectorField};

use crate::primitive::Primitive;

/// Trait for mathematical objects that compile to rendering primitives.
pub trait MathObject {
    /// Produce a collection of primitives for rendering.
    fn to_primitives(&self) -> Vec<Primitive>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Color, StrokeStyle};

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

    #[test]
    fn function_plot_data_to_screen_edge_equal_range() {
        let axes = Axes {
            x_range: (5.0, 5.0, 1.0),
            y_range: (0.0, 10.0, 1.0),
            ..Axes::default()
        };
        let plot = FunctionPlot::sample(&axes, |_| 5.0, StrokeStyle::default());
        assert_eq!(plot.points.len(), 200);
        assert!((plot.points[0][0] - 5.0).abs() < 1e-10);
        assert!((plot.points[199][0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn function_plot_from_points_single_point() {
        let axes = Axes::default();
        let plot = FunctionPlot::from_points(vec![[1.0, 2.0]], &axes, StrokeStyle::default());
        let prims = plot.to_primitives();
        assert_eq!(prims.len(), 1);
        if let Primitive::Line { points, .. } = &prims[0] {
            assert_eq!(points.len(), 1);
        }
    }

    #[test]
    fn function_plot_from_points_multiple() {
        let axes = Axes::default();
        let points = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 4.0]];
        let plot = FunctionPlot::from_points(points.clone(), &axes, StrokeStyle::default());
        assert_eq!(plot.points.len(), 3);
        assert_eq!(plot.points, points);
    }

    #[test]
    fn parametric_curve_sample_single_sample() {
        let curve =
            ParametricCurve::sample(|t| t, |t| t * 2.0, (0.0, 1.0), 2, StrokeStyle::default());
        assert_eq!(curve.points.len(), 2);
        assert!((curve.points[0][0] - 0.0).abs() < 1e-10);
        assert!((curve.points[1][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn math_object_function_plot() {
        let axes = Axes::default();
        let plot = FunctionPlot::sample(&axes, |x| x * x, StrokeStyle::default());
        let prims = plot.to_primitives();
        assert_eq!(prims.len(), 1);
        assert!(matches!(&prims[0], Primitive::Line { .. }));
    }

    #[test]
    fn math_object_axes() {
        let axes = Axes::default();
        let prims = axes.to_primitives();
        assert!(!prims.is_empty());
    }
}
