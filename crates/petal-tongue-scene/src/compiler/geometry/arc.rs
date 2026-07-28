// SPDX-License-Identifier: AGPL-3.0-or-later
//! Arc geometry compilation: polar plasmid maps and cartesian gauges.

use serde_json::Value;

use crate::domain_palette::{DomainPalette, categorical_color};
use crate::grammar::{CoordinateSystem, GrammarExpr};
use crate::math::Axes;
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};

use super::super::utils::get_number;
use super::row_data_id;

pub(super) fn compile_arc(
    expr: &GrammarExpr,
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let primary = palette.primary;
    if expr.coordinate == CoordinateSystem::Polar && points.len() > 1 {
        compile_polar_arcs(data, points, axes, palette)
    } else if let Some(&[_, value]) = points.first() {
        compile_cartesian_gauge(value, axes, palette, primary)
    } else {
        Vec::new()
    }
}

fn compile_polar_arcs(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    // Polar multi-arc: each row is (midpoint_angle, ring_index) with
    // `value` = angular span. Renders concentric arc features like
    // pLannotate circular plasmid maps.
    let cx = axes.origin.0 + axes.width / 2.0;
    let cy = axes.origin.1 - axes.height / 2.0;
    let base_radius = axes.width.min(axes.height) * 0.25;
    let ring_spacing = axes.width.min(axes.height) * 0.06;
    let arc_thickness = ring_spacing * 0.7;

    // Backbone circle
    let mut prims = vec![Primitive::Arc {
        cx,
        cy,
        radius: base_radius,
        start_angle: 0.0,
        end_angle: std::f64::consts::TAU,
        fill: None,
        stroke: Some(StrokeStyle {
            color: palette.primary,
            width: 1.5,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        }),
        data_id: Some("backbone".to_owned()),
    }];

    for (i, (&[mid_angle_deg, ring_idx], row)) in points.iter().zip(data.iter()).enumerate() {
        let span_deg = row
            .as_object()
            .and_then(|o| get_number(o, "value"))
            .unwrap_or(10.0);
        let start_deg = mid_angle_deg - span_deg / 2.0;
        let end_deg = mid_angle_deg + span_deg / 2.0;

        let start_rad = start_deg.to_radians() - std::f64::consts::FRAC_PI_2;
        let end_rad = end_deg.to_radians() - std::f64::consts::FRAC_PI_2;

        #[expect(clippy::cast_sign_loss, reason = "clamped to non-negative")]
        #[expect(clippy::cast_possible_truncation, reason = "grid indices fit in usize")]
        let ring = ring_idx.max(0.0) as usize;
        #[expect(
            clippy::cast_precision_loss,
            reason = "ring count well within f64 precision"
        )]
        let r = (ring as f64 + 1.0).mul_add(ring_spacing, base_radius);

        let fill = categorical_color(palette, i);

        // Sample arc polygon (inner + outer arcs, closed)
        #[expect(clippy::cast_sign_loss, reason = "abs + ceil + max(8) is non-negative")]
        #[expect(
            clippy::cast_possible_truncation,
            reason = "sample count well within usize"
        )]
        let n_samples = ((end_rad - start_rad).abs() * 20.0).ceil().max(8.0) as usize;
        let mut poly_pts = Vec::with_capacity(n_samples * 2 + 2);
        let r_inner = r - arc_thickness / 2.0;
        let r_outer = r + arc_thickness / 2.0;

        // Outer arc (forward)
        for j in 0..=n_samples {
            #[expect(clippy::cast_precision_loss, reason = "arc sampling")]
            let t = j as f64 / n_samples as f64;
            let angle = start_rad + t * (end_rad - start_rad);
            poly_pts.push([cx + r_outer * angle.cos(), cy + r_outer * angle.sin()]);
        }
        // Inner arc (reverse)
        for j in (0..=n_samples).rev() {
            #[expect(clippy::cast_precision_loss, reason = "arc sampling")]
            let t = j as f64 / n_samples as f64;
            let angle = start_rad + t * (end_rad - start_rad);
            poly_pts.push([cx + r_inner * angle.cos(), cy + r_inner * angle.sin()]);
        }

        prims.push(Primitive::Polygon {
            points: poly_pts,
            fill,
            stroke: Some(StrokeStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 0.15),
                width: 0.5,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
            }),
            fill_rule: crate::primitive::FillRule::NonZero,
            data_id: Some(row_data_id(data, i, "arc")),
        });

        // Label at midpoint of outer arc
        let label = row
            .as_object()
            .and_then(|o| o.get("label"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if !label.is_empty() && span_deg > 15.0 {
            let mid_rad = f64::midpoint(start_rad, end_rad);
            let label_r = r_outer + 6.0;
            prims.push(Primitive::Text {
                x: cx + label_r * mid_rad.cos(),
                y: cy + label_r * mid_rad.sin(),
                content: label.to_string(),
                font_size: 8.0,
                color: Color::BLACK,
                anchor: AnchorPoint::Center,
                bold: false,
                italic: false,
                data_id: None,
            });
        }
    }

    prims
}

fn compile_cartesian_gauge(
    value: f64,
    axes: &Axes,
    palette: &DomainPalette,
    primary: Color,
) -> Vec<Primitive> {
    // Cartesian gauge (single-arc)
    let cx = axes.origin.0 + axes.width / 2.0;
    let cy = axes.origin.1;
    let radius = axes.width.min(axes.height) * 0.4;

    let bg_color = Color::rgba(
        palette.chart_bg.r,
        palette.chart_bg.g,
        palette.chart_bg.b,
        0.5,
    );
    let mut prims = vec![Primitive::Arc {
        cx,
        cy,
        radius,
        start_angle: std::f64::consts::PI,
        end_angle: 2.0 * std::f64::consts::PI,
        fill: Some(bg_color),
        stroke: None,
        data_id: Some("gauge-bg".to_owned()),
    }];

    let normalized = value.clamp(0.0, 1.0);
    let sweep = std::f64::consts::PI * normalized;
    prims.push(Primitive::Arc {
        cx,
        cy,
        radius,
        start_angle: std::f64::consts::PI,
        end_angle: std::f64::consts::PI + sweep,
        fill: Some(primary),
        stroke: None,
        data_id: Some("gauge-fill".to_owned()),
    });

    prims.push(Primitive::Text {
        x: cx,
        y: cy - radius * 0.15,
        content: format!("{value:.1}"),
        font_size: 18.0,
        color: primary,
        anchor: AnchorPoint::Center,
        bold: true,
        italic: false,
        data_id: None,
    });

    prims
}
