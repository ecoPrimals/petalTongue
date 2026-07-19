// SPDX-License-Identifier: AGPL-3.0-or-later
//! Geometry compilation: map `GrammarExpr` geometry types to primitives.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::domain_palette::{DomainPalette, categorical_color};
use crate::grammar::{CoordinateSystem, GeometryType, GrammarExpr, VariableRole};
use crate::math::Axes;
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, MeshVertex, Primitive, StrokeStyle};

use super::utils::get_number;

/// Extract a semantic `data_id` from a data row, falling back to a synthetic ID.
fn row_data_id(data: &[Value], index: usize, fallback_prefix: &str) -> String {
    data.get(index)
        .and_then(|row| row.as_object())
        .and_then(|o| o.get("data_id"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map_or_else(|| format!("{fallback_prefix}-{index}"), String::from)
}

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
                    data_id: Some(row_data_id(data, i, "pt")),
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
                        data_id: Some(row_data_id(data, i, "bar")),
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
                    data_id: Some("line-0".to_owned()),
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
                    data_id: Some("area-fill".to_owned()),
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
                    data_id: Some("area-line".to_owned()),
                });
                prims
            }
        }

        GeometryType::Ribbon => {
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
                                    let t =
                                        ((val - val_min) / val_range).clamp(0.0, 1.0) as f32;
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
                                        let t = ((val - val_min) / val_range).clamp(0.0, 1.0)
                                            as f32;
                                        Color::rgba(
                                            primary
                                                .r
                                                .mul_add(t, palette.chart_bg.r * (1.0 - t)),
                                            primary
                                                .g
                                                .mul_add(t, palette.chart_bg.g * (1.0 - t)),
                                            primary
                                                .b
                                                .mul_add(t, palette.chart_bg.b * (1.0 - t)),
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

        GeometryType::Arc => {
            if expr.coordinate == CoordinateSystem::Polar && points.len() > 1 {
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

                for (i, (&[mid_angle_deg, ring_idx], row)) in
                    points.iter().zip(data.iter()).enumerate()
                {
                    let span_deg = row
                        .as_object()
                        .and_then(|o| get_number(o, "value"))
                        .unwrap_or(10.0);
                    let start_deg = mid_angle_deg - span_deg / 2.0;
                    let end_deg = mid_angle_deg + span_deg / 2.0;

                    let start_rad = start_deg.to_radians() - std::f64::consts::FRAC_PI_2;
                    let end_rad = end_deg.to_radians() - std::f64::consts::FRAC_PI_2;

                    #[expect(clippy::cast_sign_loss, reason = "clamped to non-negative")]
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "grid indices fit in usize"
                    )]
                    let ring = ring_idx.max(0.0) as usize;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "ring count well within f64 precision"
                    )]
                    let r = (ring as f64 + 1.0).mul_add(ring_spacing, base_radius);

                    let fill = categorical_color(palette, i);

                    // Sample arc polygon (inner + outer arcs, closed)
                    #[expect(
                        clippy::cast_sign_loss,
                        reason = "abs + ceil + max(8) is non-negative"
                    )]
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
            } else if let Some(&[_, value]) = points.first() {
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
            } else {
                Vec::new()
            }
        }

        GeometryType::Text => points
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
            .collect(),

        GeometryType::ErrorBar => {
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

        GeometryType::Sphere => {
            let z_field = expr
                .variables
                .iter()
                .find(|v| v.role == VariableRole::Z)
                .map(|v| v.field.as_str());

            points
                .iter()
                .zip(data.iter())
                .enumerate()
                .map(|(i, (&[x, y], row))| {
                    let z = z_field
                        .and_then(|f| row.as_object().and_then(|o| get_number(o, f)))
                        .unwrap_or(0.0);
                    let radius = row
                        .as_object()
                        .and_then(|o| get_number(o, "radius"))
                        .unwrap_or(1.0);
                    let color = categorical_color(palette, i);
                    let mesh = generate_sphere_mesh(x, y, z, radius, color, 16);
                    Primitive::Mesh {
                        vertices: mesh.0,
                        indices: mesh.1,
                        data_id: Some(row_data_id(data, i, "sphere")),
                    }
                })
                .collect()
        }

        GeometryType::Cylinder => {
            let z_field = expr
                .variables
                .iter()
                .find(|v| v.role == VariableRole::Z)
                .map(|v| v.field.as_str());

            points
                .iter()
                .zip(data.iter())
                .enumerate()
                .map(|(i, (&[x, y], row))| {
                    let z = z_field
                        .and_then(|f| row.as_object().and_then(|o| get_number(o, f)))
                        .unwrap_or(0.0);
                    let radius = row
                        .as_object()
                        .and_then(|o| get_number(o, "radius"))
                        .unwrap_or(0.5);
                    let height = row
                        .as_object()
                        .and_then(|o| get_number(o, "height"))
                        .unwrap_or(2.0);
                    let color = categorical_color(palette, i);
                    let mesh = generate_cylinder_mesh(x, y, z, radius, height, color, 16);
                    Primitive::Mesh {
                        vertices: mesh.0,
                        indices: mesh.1,
                        data_id: Some(row_data_id(data, i, "cyl")),
                    }
                })
                .collect()
        }

        GeometryType::Mesh3D => {
            // Mesh3D expects pre-built vertex/index data in the data rows.
            // Each row should have `vertices` (array of [x,y,z]) and `indices` (array of u32).
            data.iter()
                .enumerate()
                .filter_map(|(i, row)| {
                    let obj = row.as_object()?;
                    let verts_val = obj.get("vertices")?;
                    let indices_val = obj.get("indices")?;

                    let vertices: Vec<MeshVertex> = verts_val
                        .as_array()?
                        .iter()
                        .map(|v| {
                            let arr = v.as_array();
                            let pos = arr.map_or([0.0, 0.0, 0.0], |a| {
                                [
                                    a.first().and_then(Value::as_f64).unwrap_or(0.0),
                                    a.get(1).and_then(Value::as_f64).unwrap_or(0.0),
                                    a.get(2).and_then(Value::as_f64).unwrap_or(0.0),
                                ]
                            });
                            MeshVertex {
                                position: pos,
                                normal: [0.0, 1.0, 0.0],
                                color: categorical_color(palette, i),
                            }
                        })
                        .collect();

                    #[expect(clippy::cast_possible_truncation, reason = "mesh indices fit u32")]
                    let indices: Vec<u32> = indices_val
                        .as_array()?
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u32))
                        .collect();

                    Some(Primitive::Mesh {
                        vertices,
                        indices,
                        data_id: Some(row_data_id(data, i, "mesh")),
                    })
                })
                .collect()
        }
    }
}

/// Generate a UV sphere mesh at the given center with `segments` longitudinal slices.
#[expect(clippy::cast_precision_loss, reason = "sphere tessellation indices")]
fn generate_sphere_mesh(
    cx: f64,
    cy: f64,
    cz: f64,
    radius: f64,
    color: Color,
    segments: usize,
) -> (Vec<MeshVertex>, Vec<u32>) {
    let rings = segments / 2;
    let mut vertices = Vec::with_capacity((rings + 1) * (segments + 1));
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let phi = std::f64::consts::PI * ring as f64 / rings as f64;
        for seg in 0..=segments {
            let theta = 2.0 * std::f64::consts::PI * seg as f64 / segments as f64;
            let nx = phi.sin() * theta.cos();
            let ny = phi.cos();
            let nz = phi.sin() * theta.sin();
            vertices.push(MeshVertex {
                position: [
                    radius.mul_add(nx, cx),
                    radius.mul_add(ny, cy),
                    radius.mul_add(nz, cz),
                ],
                normal: [nx, ny, nz],
                color,
            });
        }
    }

    let stride = segments + 1;
    for ring in 0..rings {
        for seg in 0..segments {
            #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
            let tl = (ring * stride + seg) as u32;
            #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
            let tr = (ring * stride + seg + 1) as u32;
            #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
            let bl = ((ring + 1) * stride + seg) as u32;
            #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
            let br = ((ring + 1) * stride + seg + 1) as u32;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    (vertices, indices)
}

/// Generate a cylinder mesh at the given center with `segments` around the circumference.
#[expect(clippy::cast_precision_loss, reason = "cylinder tessellation indices")]
fn generate_cylinder_mesh(
    cx: f64,
    cy: f64,
    cz: f64,
    radius: f64,
    height: f64,
    color: Color,
    segments: usize,
) -> (Vec<MeshVertex>, Vec<u32>) {
    let half_h = height / 2.0;
    let mut vertices = Vec::with_capacity((segments + 1) * 2);
    let mut indices = Vec::new();

    // Bottom and top rings
    for ring in 0..=1 {
        let y_off = if ring == 0 { -half_h } else { half_h };
        for seg in 0..=segments {
            let theta = 2.0 * std::f64::consts::PI * seg as f64 / segments as f64;
            let nx = theta.cos();
            let nz = theta.sin();
            vertices.push(MeshVertex {
                position: [radius.mul_add(nx, cx), cy + y_off, radius.mul_add(nz, cz)],
                normal: [nx, 0.0, nz],
                color,
            });
        }
    }

    let stride = segments + 1;
    for seg in 0..segments {
        #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
        let bl = seg as u32;
        #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
        let br = (seg + 1) as u32;
        #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
        let tl = (stride + seg) as u32;
        #[expect(clippy::cast_possible_truncation, reason = "mesh indices")]
        let tr = (stride + seg + 1) as u32;
        indices.extend_from_slice(&[bl, tl, br, br, tl, tr]);
    }

    (vertices, indices)
}
