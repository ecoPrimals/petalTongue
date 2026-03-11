// SPDX-License-Identifier: AGPL-3.0-only
//! GPU command compiler: scene graph to GPU command buffer.
//!
//! Compiles scene primitives into a serialized GPU command format
//! that can be sent to toadStool (rendering) or barraCuda (compute)
//! via IPC. The command format is a flat buffer of typed draw calls
//! suitable for batch submission to a GPU pipeline.
//!
//! When shaders (.wgsl) are implemented, this compiler will emit
//! pipeline bind + draw commands. Until then, it emits high-level
//! draw descriptors that the GPU primals can interpret.

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::modality::{ModalityCompiler, ModalityOutput};
use crate::primitive::{Color, Primitive};
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

/// A GPU draw command (high-level, primal-agnostic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuDrawCommand {
    /// Draw a filled circle.
    Circle {
        center: [f32; 2],
        radius: f32,
        color: [f32; 4],
    },
    /// Draw a polyline or closed polygon.
    Polyline {
        vertices: Vec<[f32; 2]>,
        color: [f32; 4],
        width: f32,
        closed: bool,
    },
    /// Draw a filled rectangle.
    FillRect {
        min: [f32; 2],
        max: [f32; 2],
        color: [f32; 4],
        corner_radius: f32,
    },
    /// Draw a triangle mesh (for Mesh primitives).
    Mesh {
        vertices: Vec<[f32; 3]>,
        indices: Vec<u32>,
        colors: Vec<[f32; 4]>,
    },
    /// Set the viewport/scissor rect.
    SetViewport {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// Clear the framebuffer.
    Clear { color: [f32; 4] },
}

/// Compiles scene graph to GPU draw commands.
///
/// The output is `ModalityOutput::GpuCommands` containing a bincode/JSON
/// serialized `Vec<GpuDrawCommand>` that toadStool or barraCuda can execute.
#[derive(Debug, Clone, Default)]
pub struct GpuCompiler {
    viewport_width: f32,
    viewport_height: f32,
}

impl GpuCompiler {
    #[must_use]
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }

    fn color_to_f32(c: Color) -> [f32; 4] {
        [c.r, c.g, c.b, c.a]
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "scene coordinates to GPU f32 is acceptable truncation"
    )]
    fn compile_primitive(
        prim: &Primitive,
        transform: &Transform2D,
        commands: &mut Vec<GpuDrawCommand>,
    ) {
        match prim {
            Primitive::Point {
                x, y, radius, fill, ..
            } => {
                let (tx, ty) = transform.apply(*x, *y);
                let color = fill.map_or([0.0, 0.0, 0.0, 1.0], Self::color_to_f32);
                commands.push(GpuDrawCommand::Circle {
                    center: [tx as f32, ty as f32],
                    radius: *radius as f32,
                    color,
                });
            }
            Primitive::Line {
                points,
                stroke,
                closed,
                ..
            } => {
                let verts: Vec<[f32; 2]> = points
                    .iter()
                    .map(|[px, py]| {
                        let (tx, ty) = transform.apply(*px, *py);
                        [tx as f32, ty as f32]
                    })
                    .collect();
                commands.push(GpuDrawCommand::Polyline {
                    vertices: verts,
                    color: Self::color_to_f32(stroke.color),
                    width: stroke.width,
                    closed: *closed,
                });
            }
            Primitive::Rect {
                x,
                y,
                width,
                height,
                fill,
                corner_radius,
                ..
            } => {
                let (tx, ty) = transform.apply(*x, *y);
                let (tx2, ty2) = transform.apply(x + width, y + height);
                let color = fill.map_or([0.0, 0.0, 0.0, 0.0], Self::color_to_f32);
                commands.push(GpuDrawCommand::FillRect {
                    min: [tx as f32, ty as f32],
                    max: [tx2 as f32, ty2 as f32],
                    color,
                    corner_radius: *corner_radius as f32,
                });
            }
            Primitive::Polygon { points, fill, .. } => {
                let verts: Vec<[f32; 2]> = points
                    .iter()
                    .map(|[px, py]| {
                        let (tx, ty) = transform.apply(*px, *py);
                        [tx as f32, ty as f32]
                    })
                    .collect();
                commands.push(GpuDrawCommand::Polyline {
                    vertices: verts,
                    color: Self::color_to_f32(*fill),
                    width: 0.0,
                    closed: true,
                });
            }
            Primitive::Mesh {
                vertices, indices, ..
            } => {
                let verts: Vec<[f32; 3]> = vertices
                    .iter()
                    .map(|v| {
                        let (tx, ty) = transform.apply(v.position[0], v.position[1]);
                        [tx as f32, ty as f32, v.position[2] as f32]
                    })
                    .collect();
                let colors: Vec<[f32; 4]> = vertices
                    .iter()
                    .map(|v| Self::color_to_f32(v.color))
                    .collect();
                commands.push(GpuDrawCommand::Mesh {
                    vertices: verts,
                    indices: indices.clone(),
                    colors,
                });
            }
            Primitive::Arc { .. } | Primitive::BezierPath { .. } | Primitive::Text { .. } => {
                // Text requires glyph rasterization; Arc/Bezier need tessellation.
                // Both would be handled by a shader stage in the full GPU pipeline.
            }
        }
    }
}

impl ModalityCompiler for GpuCompiler {
    fn name(&self) -> &'static str {
        "GpuCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let mut commands = vec![
            GpuDrawCommand::SetViewport {
                x: 0.0,
                y: 0.0,
                width: self.viewport_width,
                height: self.viewport_height,
            },
            GpuDrawCommand::Clear {
                color: [0.0, 0.0, 0.0, 1.0],
            },
        ];

        for (transform, prim) in scene.flatten() {
            Self::compile_primitive(prim, &transform, &mut commands);
        }

        let json = serde_json::to_vec(&commands).unwrap_or_default();
        ModalityOutput::GpuCommands(Bytes::from(json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::MeshVertex;
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn gpu_compiler_emits_viewport_and_clear() {
        let graph = SceneGraph::new();
        let out = GpuCompiler::new(800.0, 600.0).compile(&graph);
        let ModalityOutput::GpuCommands(bytes) = &out else {
            panic!("expected GpuCommands");
        };
        let commands: Vec<GpuDrawCommand> = serde_json::from_slice(bytes).unwrap();
        assert!(commands.len() >= 2);
        assert!(matches!(commands[0], GpuDrawCommand::SetViewport { .. }));
        assert!(matches!(commands[1], GpuDrawCommand::Clear { .. }));
    }

    #[test]
    fn gpu_compiler_handles_point() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("p").with_primitive(Primitive::Point {
            x: 100.0,
            y: 200.0,
            radius: 5.0,
            fill: Some(Color::WHITE),
            stroke: None,
            data_id: Some("d1".to_string()),
        }));
        let out = GpuCompiler::new(800.0, 600.0).compile(&graph);
        let ModalityOutput::GpuCommands(bytes) = &out else {
            panic!("expected GpuCommands");
        };
        let commands: Vec<GpuDrawCommand> = serde_json::from_slice(bytes).unwrap();
        let circle_count = commands
            .iter()
            .filter(|c| matches!(c, GpuDrawCommand::Circle { .. }))
            .count();
        assert_eq!(circle_count, 1);
    }

    #[test]
    fn gpu_compiler_handles_mesh() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("m").with_primitive(Primitive::Mesh {
            vertices: vec![
                MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                },
                MeshVertex {
                    position: [1.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: Color::rgba(0.0, 1.0, 0.0, 1.0),
                },
                MeshVertex {
                    position: [0.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: Color::rgba(0.0, 0.0, 1.0, 1.0),
                },
            ],
            indices: vec![0, 1, 2],
            data_id: Some("mesh1".to_string()),
        }));
        let out = GpuCompiler::new(800.0, 600.0).compile(&graph);
        let ModalityOutput::GpuCommands(bytes) = &out else {
            panic!("expected GpuCommands");
        };
        let commands: Vec<GpuDrawCommand> = serde_json::from_slice(bytes).unwrap();
        let mesh_count = commands
            .iter()
            .filter(|c| matches!(c, GpuDrawCommand::Mesh { .. }))
            .count();
        assert_eq!(mesh_count, 1);
    }
}
