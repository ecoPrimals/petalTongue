// SPDX-License-Identifier: AGPL-3.0-or-later
//! 3D mesh geometry compilation: spheres, cylinders, and pre-built meshes.

use serde_json::Value;

use crate::domain_palette::{DomainPalette, categorical_color};
use crate::grammar::{GrammarExpr, VariableRole};
use crate::primitive::{Color, MeshVertex, Primitive};

use super::super::utils::get_number;
use super::row_data_id;

pub(super) fn compile_sphere(
    expr: &GrammarExpr,
    data: &[Value],
    points: &[[f64; 2]],
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let z_field = z_field_from_expr(expr);

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

pub(super) fn compile_cylinder(
    expr: &GrammarExpr,
    data: &[Value],
    points: &[[f64; 2]],
    palette: &DomainPalette,
) -> Vec<Primitive> {
    let z_field = z_field_from_expr(expr);

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

pub(super) fn compile_mesh_3d(data: &[Value], palette: &DomainPalette) -> Vec<Primitive> {
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

fn z_field_from_expr(expr: &GrammarExpr) -> Option<&str> {
    expr.variables
        .iter()
        .find(|v| v.role == VariableRole::Z)
        .map(|v| v.field.as_str())
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
