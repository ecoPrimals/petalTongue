// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tile (heatmap) geometry compilation with color interpolation.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::domain_palette::DomainPalette;
use crate::math::Axes;
use crate::primitive::{Color, LineCap, LineJoin, Primitive, StrokeStyle};

use super::super::utils::get_number;
use super::row_data_id;

#[expect(
    clippy::too_many_lines,
    reason = "tile color interpolation is a cohesive unit"
)]
pub(super) fn compile_tile(
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let primary = palette.primary;
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

                // Explicit RGBA color (e.g., from ColorGrid binding)
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "color channels are 0.0–1.0 range, f32 is sufficient"
                )]
                let explicit_color = data.get(i).and_then(|d| {
                    let o = d.as_object()?;
                    let cr = get_number(o, "r")? as f32;
                    let cg = get_number(o, "g")? as f32;
                    let cb = get_number(o, "b")? as f32;
                    let ca = get_number(o, "a").unwrap_or(1.0) as f32;
                    Some(Color::rgba(cr, cg, cb, ca))
                });

                let fill = explicit_color.unwrap_or_else(|| {
                    let status = data.get(i).and_then(|d| {
                        d.as_object()
                            .and_then(|o| o.get("status"))
                            .and_then(|s| s.as_str())
                    });
                    status.map_or_else(
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
                    )
                });
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
                    data_id: Some(row_data_id(data, i, "tile")),
                }
            })
            .collect()
    }
}
