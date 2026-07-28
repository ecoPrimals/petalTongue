// SPDX-License-Identifier: AGPL-3.0-or-later
//! Geometry compilation: map `GrammarExpr` geometry types to primitives.

mod arc;
mod cartesian_2d;
mod mesh_3d;
mod tile;

use serde_json::Value;

use crate::domain_palette::DomainPalette;
use crate::grammar::{GeometryType, GrammarExpr};
use crate::math::Axes;
use crate::primitive::{Primitive, StrokeStyle};

/// Extract a semantic `data_id` from a data row, falling back to a synthetic ID.
pub(super) fn row_data_id(data: &[Value], index: usize, fallback_prefix: &str) -> String {
    data.get(index)
        .and_then(|row| row.as_object())
        .and_then(|o| o.get("data_id"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map_or_else(|| format!("{fallback_prefix}-{index}"), String::from)
}

/// Map data-space points to screen-space coordinates.
pub(super) fn points_to_screen(axes: &Axes, points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    points
        .iter()
        .map(|&[x, y]| axes.data_to_screen(x, y))
        .map(|(sx, sy)| <[f64; 2]>::from((sx, sy)))
        .collect()
}

/// Compile geometry from grammar expression into primitives.
pub fn compile_geometry(
    expr: &GrammarExpr,
    data: &[Value],
    points: &[[f64; 2]],
    axes: &Axes,
    palette: &DomainPalette,
    stroke: &StrokeStyle,
) -> Vec<Primitive> {
    match expr.geometry {
        GeometryType::Point => cartesian_2d::compile_point(data, points, axes, palette),
        GeometryType::Bar => cartesian_2d::compile_bar(data, points, axes, palette),
        GeometryType::Line => cartesian_2d::compile_line(points, axes, stroke),
        GeometryType::Area => cartesian_2d::compile_area(points, axes, palette, stroke),
        GeometryType::Ribbon => cartesian_2d::compile_ribbon(data, points, axes, palette),
        GeometryType::Tile => tile::compile_tile(data, points, axes, palette),
        GeometryType::Arc => arc::compile_arc(expr, data, points, axes, palette),
        GeometryType::Text => cartesian_2d::compile_text(data, points, axes, palette),
        GeometryType::ErrorBar => {
            cartesian_2d::compile_error_bar(data, points, axes, palette, stroke)
        }
        GeometryType::Sphere => mesh_3d::compile_sphere(expr, data, points, palette),
        GeometryType::Cylinder => mesh_3d::compile_cylinder(expr, data, points, palette),
        GeometryType::Mesh3D => mesh_3d::compile_mesh_3d(data, palette),
    }
}
