// SPDX-License-Identifier: AGPL-3.0-or-later
//! WebGL modality compiler: scene graph to GPU draw commands.
//!
//! Produces a serializable [`WebGlScene`] containing vertex buffers, index buffers,
//! and draw calls that a browser-side WebGL renderer can consume directly.
//! Supports both 2D primitives (projected to clip space) and 3D mesh primitives
//! (with camera view-projection).

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::primitive::{Color, MeshVertex, Primitive};
use crate::scene_graph::SceneGraph;
use crate::transform::{Camera, Projection, Transform3D};

use super::{ModalityCompiler, ModalityOutput};

/// A complete WebGL scene ready for GPU upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGlScene {
    /// Interleaved vertex data: position (3×f32) + color (4×f32) per vertex.
    pub vertices: Vec<f32>,
    /// Triangle index buffer (3 indices per triangle).
    pub indices: Vec<u32>,
    /// Draw call ranges into the index buffer.
    pub draw_calls: Vec<DrawCall>,
    /// View-projection matrix (column-major 4×4).
    pub view_projection: [f32; 16],
    /// Viewport dimensions for 2D coordinate mapping.
    pub viewport: [f32; 2],
}

/// A single draw call referencing a range of indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawCall {
    /// Byte offset into the index buffer.
    pub index_offset: u32,
    /// Number of indices to draw.
    pub index_count: u32,
    /// Primitive topology (triangles, lines, points).
    pub topology: Topology,
    /// Optional data ID for interaction picking.
    pub data_id: Option<String>,
}

/// GPU primitive topology.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Topology {
    /// Triangle list (3 indices per triangle).
    Triangles,
    /// Line list (2 indices per line segment).
    Lines,
    /// Point list (1 index per point).
    Points,
}

/// Compiles a scene graph into WebGL-ready vertex/index buffers and draw commands.
pub struct WebGlCompiler;

impl WebGlCompiler {
    /// Create a new `WebGlCompiler`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for WebGlCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ModalityCompiler for WebGlCompiler {
    #[expect(clippy::too_many_lines, reason = "cohesive primitive dispatch")]
    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let camera = scene.effective_camera();
        let vp_matrix = compute_view_projection(&camera);
        let (vw, vh) = viewport_dims(&camera);

        let flat = scene.flatten_3d();
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut draw_calls: Vec<DrawCall> = Vec::new();

        for (transform, prim, node_id) in &flat {
            let base_vertex = vertices.len() / 7;
            let base_index = indices.len();

            match prim {
                Primitive::Mesh {
                    vertices: mesh_verts,
                    indices: mesh_idx,
                    data_id,
                } => {
                    emit_mesh(&mut vertices, &mut indices, mesh_verts, mesh_idx, transform);
                    draw_calls.push(DrawCall {
                        index_offset: u32_from_usize(base_index),
                        index_count: u32_from_usize(mesh_idx.len()),
                        topology: Topology::Triangles,
                        data_id: data_id.clone().or_else(|| Some(node_id.to_string())),
                    });
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    fill,
                    data_id,
                    ..
                } => {
                    let color = fill.unwrap_or(Color::BLACK);
                    emit_quad(
                        &mut vertices,
                        &mut indices,
                        *x,
                        *y,
                        *width,
                        *height,
                        color,
                        vw,
                        vh,
                    );
                    draw_calls.push(DrawCall {
                        index_offset: u32_from_usize(base_index),
                        index_count: 6,
                        topology: Topology::Triangles,
                        data_id: data_id.clone().or_else(|| Some(node_id.to_string())),
                    });
                }
                Primitive::Point {
                    x,
                    y,
                    fill,
                    data_id,
                    ..
                } => {
                    let color = fill.unwrap_or(Color::BLACK);
                    push_vertex(&mut vertices, *x, *y, 0.0, color, vw, vh);
                    #[expect(clippy::cast_possible_truncation, reason = "vertex index")]
                    indices.push(base_vertex as u32);
                    draw_calls.push(DrawCall {
                        index_offset: u32_from_usize(base_index),
                        index_count: 1,
                        topology: Topology::Points,
                        data_id: data_id.clone().or_else(|| Some(node_id.to_string())),
                    });
                }
                Primitive::Line {
                    points,
                    stroke,
                    data_id,
                    ..
                } => {
                    if points.len() >= 2 {
                        let color = stroke.color;
                        for pt in points {
                            push_vertex(&mut vertices, pt[0], pt[1], 0.0, color, vw, vh);
                        }
                        #[expect(clippy::cast_possible_truncation, reason = "vertex index")]
                        for i in 0..points.len() - 1 {
                            indices.push((base_vertex + i) as u32);
                            indices.push((base_vertex + i + 1) as u32);
                        }
                        draw_calls.push(DrawCall {
                            index_offset: u32_from_usize(base_index),
                            #[expect(
                                clippy::cast_possible_truncation,
                                reason = "line segment count"
                            )]
                            index_count: ((points.len() - 1) * 2) as u32,
                            topology: Topology::Lines,
                            data_id: data_id.clone().or_else(|| Some(node_id.to_string())),
                        });
                    }
                }
                _ => {}
            }

            let _ = base_vertex;
        }

        let webgl_scene = WebGlScene {
            vertices,
            indices,
            draw_calls,
            view_projection: vp_matrix,
            #[expect(clippy::cast_possible_truncation, reason = "viewport dims fit f32")]
            viewport: [vw as f32, vh as f32],
        };

        let json = serde_json::to_vec(&webgl_scene).unwrap_or_default();
        ModalityOutput::GpuCommands(Bytes::from(json))
    }

    fn name(&self) -> &'static str {
        "webgl"
    }
}

fn compute_view_projection(camera: &Camera) -> [f32; 16] {
    let proj = match camera.projection {
        Projection::Orthographic {
            width,
            height,
            near,
            far,
        } => ortho_matrix(width, height, near, far),
        Projection::Perspective {
            fov_y,
            aspect,
            near,
            far,
        } => perspective_matrix(fov_y, aspect, near, far),
    };

    let view = invert_transform(&camera.transform);
    mul_mat4(&proj, &view)
}

fn viewport_dims(camera: &Camera) -> (f64, f64) {
    match camera.projection {
        Projection::Orthographic { width, height, .. } => (width, height),
        Projection::Perspective { aspect, .. } => (600.0 * aspect, 600.0),
    }
}

#[expect(clippy::cast_possible_truncation, reason = "matrix elements fit f32")]
fn ortho_matrix(width: f64, height: f64, near: f64, far: f64) -> [f32; 16] {
    let r = width / 2.0;
    let t = height / 2.0;
    let d = far - near;
    [
        (1.0 / r) as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        (1.0 / t) as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        (-2.0 / d) as f32,
        0.0,
        0.0,
        0.0,
        (-(far + near) / d) as f32,
        1.0,
    ]
}

#[expect(clippy::cast_possible_truncation, reason = "matrix elements fit f32")]
fn perspective_matrix(fov_y: f64, aspect: f64, near: f64, far: f64) -> [f32; 16] {
    let f = 1.0 / (fov_y / 2.0).tan();
    let d = near - far;
    [
        (f / aspect) as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        f as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        ((far + near) / d) as f32,
        -1.0,
        0.0,
        0.0,
        (2.0 * far * near / d) as f32,
        0.0,
    ]
}

#[expect(clippy::cast_possible_truncation, reason = "matrix elements fit f32")]
fn invert_transform(t: &Transform3D) -> [f32; 16] {
    let m = &t.matrix;
    [
        m[0] as f32,
        m[4] as f32,
        m[8] as f32,
        0.0,
        m[1] as f32,
        m[5] as f32,
        m[9] as f32,
        0.0,
        m[2] as f32,
        m[6] as f32,
        m[10] as f32,
        0.0,
        (-(m[0].mul_add(m[12], m[1].mul_add(m[13], m[2] * m[14])))) as f32,
        (-(m[4].mul_add(m[12], m[5].mul_add(m[13], m[6] * m[14])))) as f32,
        (-(m[8].mul_add(m[12], m[9].mul_add(m[13], m[10] * m[14])))) as f32,
        1.0,
    ]
}

fn mul_mat4(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            out[col * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[col * 4 + k]).sum();
        }
    }
    out
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "vertex color channels fit f32"
)]
fn push_vertex(verts: &mut Vec<f32>, x: f64, y: f64, z: f64, color: Color, _vw: f64, _vh: f64) {
    verts.extend_from_slice(&[
        x as f32, y as f32, z as f32, color.r, color.g, color.b, color.a,
    ]);
}

#[expect(clippy::cast_possible_truncation, reason = "vertex index fits u32")]
#[expect(clippy::too_many_arguments, reason = "quad geometry needs all params")]
fn emit_quad(
    verts: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: Color,
    vw: f64,
    vh: f64,
) {
    let base = (verts.len() / 7) as u32;
    push_vertex(verts, x, y, 0.0, color, vw, vh);
    push_vertex(verts, x + w, y, 0.0, color, vw, vh);
    push_vertex(verts, x + w, y + h, 0.0, color, vw, vh);
    push_vertex(verts, x, y + h, 0.0, color, vw, vh);
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[expect(clippy::cast_possible_truncation, reason = "mesh vertex fit f32")]
fn emit_mesh(
    verts: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    mesh_verts: &[MeshVertex],
    mesh_indices: &[u32],
    transform: &Transform3D,
) {
    let base = (verts.len() / 7) as u32;
    let m = &transform.matrix;

    for v in mesh_verts {
        let [px, py, pz] = v.position;
        let tx = m[0].mul_add(px, m[4].mul_add(py, m[8].mul_add(pz, m[12])));
        let ty = m[1].mul_add(px, m[5].mul_add(py, m[9].mul_add(pz, m[13])));
        let tz = m[2].mul_add(px, m[6].mul_add(py, m[10].mul_add(pz, m[14])));
        verts.extend_from_slice(&[
            tx as f32, ty as f32, tz as f32, v.color.r, v.color.g, v.color.b, v.color.a,
        ]);
    }

    for &idx in mesh_indices {
        indices.push(base + idx);
    }
}

#[expect(clippy::cast_possible_truncation, reason = "index count fits u32")]
const fn u32_from_usize(n: usize) -> u32 {
    n as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::GrammarCompiler;
    use crate::grammar::{GeometryType, GrammarExpr};

    #[test]
    fn webgl_compiler_produces_gpu_commands() {
        let compiler = WebGlCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Bar)
            .with_x("x")
            .with_y("y");
        let data = vec![
            serde_json::json!({"x": 0.0, "y": 10.0}),
            serde_json::json!({"x": 1.0, "y": 20.0}),
        ];
        let grammar_compiler = GrammarCompiler::new();
        let scene = grammar_compiler.compile(&expr, &data);
        let output = compiler.compile(&scene);
        assert!(matches!(output, ModalityOutput::GpuCommands(_)));
        if let ModalityOutput::GpuCommands(bytes) = output {
            let webgl: WebGlScene = serde_json::from_slice(&bytes).expect("valid JSON");
            assert!(!webgl.vertices.is_empty());
            assert!(!webgl.indices.is_empty());
            assert!(!webgl.draw_calls.is_empty());
        }
    }

    #[test]
    fn webgl_compiler_handles_mesh_primitives() {
        let compiler = WebGlCompiler::new();
        let expr = GrammarExpr::new("data", GeometryType::Sphere)
            .with_x("x")
            .with_y("y");
        let data = vec![serde_json::json!({"x": 0.0, "y": 0.0, "radius": 1.0})];
        let grammar_compiler = GrammarCompiler::new();
        let scene = grammar_compiler.compile(&expr, &data);
        let output = compiler.compile(&scene);
        if let ModalityOutput::GpuCommands(bytes) = output {
            let webgl: WebGlScene = serde_json::from_slice(&bytes).expect("valid JSON");
            let has_mesh_calls = webgl
                .draw_calls
                .iter()
                .any(|c| matches!(c.topology, Topology::Triangles));
            assert!(has_mesh_calls, "sphere should produce triangle draw calls");
        } else {
            panic!("expected GpuCommands output");
        }
    }

    #[test]
    fn webgl_compiler_name() {
        assert_eq!(WebGlCompiler::new().name(), "webgl");
    }

    #[test]
    fn webgl_scene_view_projection_is_valid() {
        let compiler = WebGlCompiler::new();
        let scene = SceneGraph::new();
        let output = compiler.compile(&scene);
        if let ModalityOutput::GpuCommands(bytes) = output {
            let webgl: WebGlScene = serde_json::from_slice(&bytes).expect("valid JSON");
            assert_eq!(webgl.view_projection.len(), 16);
            let has_nonzero = webgl.view_projection.iter().any(|&v| v != 0.0);
            assert!(has_nonzero, "view-projection should not be all zeros");
        }
    }
}
