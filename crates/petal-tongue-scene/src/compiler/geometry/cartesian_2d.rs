// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cartesian 2D geometry compilation: points, bars, lines, areas, ribbons, text, error bars.

use serde_json::Value;

use crate::domain_palette::{DomainPalette, categorical_color};
use crate::math::Axes;
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};

use super::super::utils::get_number;
use super::{points_to_screen, row_data_id};

pub(super) fn compile_point(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let primary = palette.primary;
    points
        .iter()
        .enumerate()
        .map(|(i, &[x, y])| {
            let (sx, sy) = axes.data_to_screen(x, y);
            Primitive::Point {
                x: sx,
                y: sy,
                radius: 4.0,
                fill: Some(primary),
                stroke: None,
                data_id: Some(row_data_id(data, i, "pt")),
            }
        })
        .collect()
}

pub(super) fn compile_bar(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    #[expect(clippy::cast_precision_loss, reason = "bar width: f64 sufficient")]
    let bar_width = if points.is_empty() {
        0.0
    } else {
        (axes.width / points.len() as f64).max(2.0) * 0.8
    };
    points
        .iter()
        .enumerate()
        .map(|(i, &[x, y])| {
            let (sx, sy) = axes.data_to_screen(x, y);
            let (_, sy_base) = axes.data_to_screen(x, 0.0);
            let height = (sy_base - sy).abs();
            let bar_x = sx - bar_width / 2.0;
            let bar_y = sy.min(sy_base);
            Primitive::Rect {
                x: bar_x,
                y: bar_y,
                width: bar_width,
                height: height.max(1.0),
                fill: Some(categorical_color(palette, i)),
                stroke: None,
                corner_radius: 0.0,
                data_id: Some(row_data_id(data, i, "bar")),
            }
        })
        .collect()
}

pub(super) fn compile_line(
    points: &[[f64; 2]],
    axes: &Axes,
    stroke: &StrokeStyle,
) -> Vec<Primitive> {
    if points.len() < 2 {
        Vec::new()
    } else {
        let screen_points = points_to_screen(axes, points);
        vec![Primitive::Line {
            points: screen_points,
            stroke: *stroke,
            closed: false,
            data_id: Some("line-0".to_owned()),
        }]
    }
}

pub(super) fn compile_area(
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
    stroke: &StrokeStyle,
) -> Vec<Primitive> {
    let primary = palette.primary;
    if points.len() < 2 {
        Vec::new()
    } else {
        let mut screen_points = points_to_screen(axes, points);

        let (_, baseline_y) = axes.data_to_screen(0.0, 0.0);
        if let Some(last) = screen_points.last() {
            screen_points.push([last[0], baseline_y]);
        }
        if let Some(first_x) = points.first().map(|p| p[0]) {
            let (sx, _) = axes.data_to_screen(first_x, 0.0);
            screen_points.push([sx, baseline_y]);
        }

        let fill_color = Color::rgba(primary.r, primary.g, primary.b, 0.3);
        let mut prims = vec![Primitive::Polygon {
            points: screen_points,
            fill: fill_color,
            stroke: None,
            fill_rule: crate::primitive::FillRule::NonZero,
            data_id: Some("area-fill".to_owned()),
        }];

        let line_points = points_to_screen(axes, points);
        prims.push(Primitive::Line {
            points: line_points,
            stroke: *stroke,
            closed: false,
            data_id: Some("area-line".to_owned()),
        });
        prims
    }
}

pub(super) fn compile_ribbon(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let secondary = palette.secondary;
    if points.len() < 2 {
        Vec::new()
    } else {
        let fill_color = Color::rgba(secondary.r, secondary.g, secondary.b, 0.2);

        // Build upper boundary from ymax in data rows
        let mut poly_pts: Vec<[f64; 2]> = points
            .iter()
            .zip(data.iter())
            .map(|(&[x, _], row)| {
                let ymax = row
                    .as_object()
                    .and_then(|o| get_number(o, "ymax"))
                    .unwrap_or(0.0);
                axes.data_to_screen(x, ymax).into()
            })
            .collect();

        // Append lower boundary reversed (closed shape)
        poly_pts.extend(points.iter().zip(data.iter()).rev().map(|(&[x, _], row)| {
            let ymin = row
                .as_object()
                .and_then(|o| get_number(o, "ymin"))
                .unwrap_or(0.0);
            let pt: [f64; 2] = axes.data_to_screen(x, ymin).into();
            pt
        }));

        vec![Primitive::Polygon {
            points: poly_pts,
            fill: fill_color,
            stroke: Some(StrokeStyle {
                color: Color::rgba(secondary.r, secondary.g, secondary.b, 0.5),
                width: 1.0,
                cap: LineCap::Butt,
                join: LineJoin::Round,
            }),
            fill_rule: crate::primitive::FillRule::NonZero,
            data_id: Some("ribbon-0".to_owned()),
        }]
    }
}

pub(super) fn compile_text(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let primary = palette.primary;
    points
        .iter()
        .zip(data.iter())
        .enumerate()
        .map(|(i, (&[x, y], row))| {
            let (sx, sy) = axes.data_to_screen(x, y);
            let content = row
                .as_object()
                .and_then(|o| o.get("label").or_else(|| o.get("text")))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            Primitive::Text {
                x: sx,
                y: sy,
                content,
                font_size: 12.0,
                color: primary,
                anchor: AnchorPoint::Center,
                bold: false,
                italic: false,
                data_id: Some(row_data_id(data, i, "text")),
            }
        })
        .collect()
}

pub(super) fn compile_error_bar(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
    stroke: &StrokeStyle,
) -> Vec<Primitive> {
    let primary = palette.primary;
    let mut prims = Vec::new();
    for (i, &[x, y]) in points.iter().enumerate() {
        let ymin = data
            .get(i)
            .and_then(|d| d.as_object())
            .and_then(|o| get_number(o, "ymin"))
            .unwrap_or(y * 0.9);
        let ymax = data
            .get(i)
            .and_then(|d| d.as_object())
            .and_then(|o| get_number(o, "ymax"))
            .unwrap_or(y * 1.1);

        let (sx, sy) = axes.data_to_screen(x, y);
        let (_, sy_min) = axes.data_to_screen(x, ymin);
        let (_, sy_max) = axes.data_to_screen(x, ymax);
        let cap_width = 6.0;

        // Vertical whisker
        prims.push(Primitive::Line {
            points: vec![[sx, sy_min], [sx, sy_max]],
            stroke: *stroke,
            closed: false,
            data_id: Some(row_data_id(data, i, "errbar-whisker")),
        });
        // Top cap
        prims.push(Primitive::Line {
            points: vec![[sx - cap_width, sy_max], [sx + cap_width, sy_max]],
            stroke: *stroke,
            closed: false,
            data_id: None,
        });
        // Bottom cap
        prims.push(Primitive::Line {
            points: vec![[sx - cap_width, sy_min], [sx + cap_width, sy_min]],
            stroke: *stroke,
            closed: false,
            data_id: None,
        });
        // Center point
        prims.push(Primitive::Point {
            x: sx,
            y: sy,
            radius: 3.0,
            fill: Some(primary),
            stroke: None,
            data_id: Some(row_data_id(data, i, "errbar-pt")),
        });
    }
    prims
}
