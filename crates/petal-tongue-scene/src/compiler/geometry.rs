// SPDX-License-Identifier: AGPL-3.0-or-later
//! Geometry compilation: map `GrammarExpr` geometry types to primitives.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::domain_palette::{DomainPalette, categorical_color};
use crate::grammar::{GeometryType, GrammarExpr};
use crate::math::Axes;
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};

use super::utils::get_number;

/// Compile geometry from grammar expression into primitives.
#[expect(
    clippy::too_many_lines,
    reason = "geometry compilation is a cohesive match over grammar variants"
)]
pub fn compile_geometry(
    expr: &GrammarExpr,
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
    stroke: &StrokeStyle,
) -> Vec<Primitive> {
    let primary = palette.primary;
    let secondary = palette.secondary;

    match expr.geometry {
        GeometryType::Point => points
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
                    data_id: Some(format!("pt-{i}")),
                }
            })
            .collect(),

        GeometryType::Bar => {
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
                        data_id: Some(format!("bar-{i}")),
                    }
                })
                .collect()
        }

        GeometryType::Line => {
            if points.len() < 2 {
                Vec::new()
            } else {
                let screen_points: Vec<[f64; 2]> = points
                    .iter()
                    .map(|&[x, y]| axes.data_to_screen(x, y))
                    .map(|(sx, sy)| <[f64; 2]>::from((sx, sy)))
                    .collect();
                vec![Primitive::Line {
                    points: screen_points,
                    stroke: *stroke,
                    closed: false,
                    data_id: Some("line-0".to_string()),
                }]
            }
        }

        GeometryType::Area => {
            if points.len() < 2 {
                Vec::new()
            } else {
                let mut screen_points: Vec<[f64; 2]> = points
                    .iter()
                    .map(|&[x, y]| axes.data_to_screen(x, y))
                    .map(|(sx, sy)| <[f64; 2]>::from((sx, sy)))
                    .collect();

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
                    data_id: Some("area-fill".to_string()),
                }];

                let line_points: Vec<[f64; 2]> = points
                    .iter()
                    .map(|&[x, y]| axes.data_to_screen(x, y))
                    .map(|(sx, sy)| <[f64; 2]>::from((sx, sy)))
                    .collect();
                prims.push(Primitive::Line {
                    points: line_points,
                    stroke: *stroke,
                    closed: false,
                    data_id: Some("area-line".to_string()),
                });
                prims
            }
        }

        GeometryType::Ribbon => {
            let fill_color = Color::rgba(secondary.r, secondary.g, secondary.b, 0.2);
            vec![Primitive::Text {
                x: axes.origin.0 + axes.width / 2.0,
                y: axes.origin.1 - axes.height / 2.0,
                content: "Ribbon (requires ymin/ymax roles)".to_string(),
                font_size: 12.0,
                color: fill_color,
                anchor: AnchorPoint::Center,
                bold: false,
                italic: false,
                data_id: None,
            }]
        }

        GeometryType::Tile => {
            if points.is_empty() {
                Vec::new()
            } else {
                let values: Vec<f64> = data
                    .iter()
                    .map(|obj| {
                        obj.as_object()
                            .and_then(|o| get_number(o, "value"))
                            .unwrap_or(0.0)
                    })
                    .collect();
                let val_min = values.iter().copied().fold(f64::INFINITY, f64::min);
                let val_max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let val_range = (val_max - val_min).max(f64::EPSILON);

                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "grid coordinate quantization to i64 is intentional"
                )]
                let x_vals: BTreeSet<i64> = points.iter().map(|p| (p[0] * 1000.0) as i64).collect();
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "grid coordinate quantization to i64 is intentional"
                )]
                let y_vals: BTreeSet<i64> = points.iter().map(|p| (p[1] * 1000.0) as i64).collect();
                let cols = x_vals.len().max(1);
                let rows = y_vals.len().max(1);
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "tile dimensions: f64 sufficient"
                )]
                let tile_w = (axes.width / cols as f64).max(2.0);
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "tile dimensions: f64 sufficient"
                )]
                let tile_h = (axes.height / rows as f64).max(2.0);

                points
                    .iter()
                    .zip(values.iter())
                    .enumerate()
                    .map(|(i, (point, &val))| {
                        let [x, y] = *point;
                        let (sx, sy) = axes.data_to_screen(x, y);
                        let status = data.get(i).and_then(|d| {
                            d.as_object()
                                .and_then(|o| o.get("status"))
                                .and_then(|s| s.as_str())
                        });
                        let fill = status.map_or_else(
                            || {
                                #[expect(
                                    clippy::cast_possible_truncation,
                                    reason = "color interpolation t is clamped to 0.0..1.0"
                                )]
                                let t = ((val - val_min) / val_range).clamp(0.0, 1.0) as f32;
                                Color::rgba(
                                    primary.r.mul_add(t, palette.chart_bg.r * (1.0 - t)),
                                    primary.g.mul_add(t, palette.chart_bg.g * (1.0 - t)),
                                    primary.b.mul_add(t, palette.chart_bg.b * (1.0 - t)),
                                    0.9,
                                )
                            },
                            |status| match status {
                                "normal" => palette.normal,
                                "warning" => palette.warning,
                                "critical" => palette.critical,
                                _ => {
                                    #[expect(
                                        clippy::cast_possible_truncation,
                                        reason = "color interpolation t is clamped to 0.0..1.0"
                                    )]
                                    let t = ((val - val_min) / val_range).clamp(0.0, 1.0) as f32;
                                    Color::rgba(
                                        primary.r.mul_add(t, palette.chart_bg.r * (1.0 - t)),
                                        primary.g.mul_add(t, palette.chart_bg.g * (1.0 - t)),
                                        primary.b.mul_add(t, palette.chart_bg.b * (1.0 - t)),
                                        0.9,
                                    )
                                }
                            },
                        );
                        Primitive::Rect {
                            x: sx - tile_w / 2.0,
                            y: sy - tile_h / 2.0,
                            width: tile_w,
                            height: tile_h,
                            fill: Some(fill),
                            stroke: Some(StrokeStyle {
                                color: Color::rgba(0.0, 0.0, 0.0, 0.1),
                                width: 0.5,
                                cap: LineCap::Butt,
                                join: LineJoin::Miter,
                            }),
                            corner_radius: 0.0,
                            data_id: Some(format!("tile-{i}")),
                        }
                    })
                    .collect()
            }
        }

        GeometryType::Arc => {
            if let Some(&[_, value]) = points.first() {
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
                    data_id: Some("gauge-bg".to_string()),
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
                    data_id: Some("gauge-fill".to_string()),
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
            } else {
                Vec::new()
            }
        }

        _ => {
            vec![Primitive::Text {
                x: axes.origin.0 + axes.width / 2.0,
                y: axes.origin.1 - axes.height / 2.0,
                content: format!("Unsupported geometry: {:?}", expr.geometry),
                font_size: 12.0,
                color: Color::BLACK,
                anchor: AnchorPoint::Center,
                bold: false,
                italic: false,
                data_id: None,
            }]
        }
    }
}
