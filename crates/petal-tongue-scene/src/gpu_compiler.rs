// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// A GPU draw command with provenance for primitive ID buffer (barraCuda).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCommandWithProvenance {
    pub command: GpuDrawCommand,
    pub node_id: Option<String>,
    pub data_id: Option<String>,
    pub primitive_index: usize,
}

/// Maps command indices to their scene-graph source for GPU provenance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuProvenanceMap {
    entries: Vec<GpuProvenanceEntry>,
}

/// Single provenance entry mapping a command index to scene-graph source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProvenanceEntry {
    pub command_index: usize,
    pub node_id: String,
    pub data_id: Option<String>,
    pub primitive_index: usize,
}

impl GpuProvenanceMap {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, entry: GpuProvenanceEntry) {
        self.entries.push(entry);
    }

    #[must_use]
    pub fn query_command(&self, command_index: usize) -> Option<&GpuProvenanceEntry> {
        self.entries
            .iter()
            .find(|e| e.command_index == command_index)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
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
    pub const fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }

    const fn color_to_f32(c: Color) -> [f32; 4] {
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

    /// Compile scene to GPU commands with provenance for primitive ID buffer.
    ///
    /// Returns both the command buffer (each draw command wrapped with provenance)
    /// and a map from command indices to scene-graph source. `SetViewport` and Clear
    /// have no provenance; only draw commands (Circle, Polyline, `FillRect`, Mesh)
    /// are registered in the map.
    #[must_use]
    pub fn compile_with_provenance(
        &self,
        scene: &SceneGraph,
    ) -> (Vec<GpuCommandWithProvenance>, GpuProvenanceMap) {
        let mut commands = vec![
            GpuCommandWithProvenance {
                command: GpuDrawCommand::SetViewport {
                    x: 0.0,
                    y: 0.0,
                    width: self.viewport_width,
                    height: self.viewport_height,
                },
                node_id: None,
                data_id: None,
                primitive_index: 0,
            },
            GpuCommandWithProvenance {
                command: GpuDrawCommand::Clear {
                    color: [0.0, 0.0, 0.0, 1.0],
                },
                node_id: None,
                data_id: None,
                primitive_index: 0,
            },
        ];
        let mut provenance = GpuProvenanceMap::new();
        let mut primitive_index = 0_usize;

        for (transform, prim, node_id) in scene.flatten_with_ids() {
            let mut temp = Vec::new();
            Self::compile_primitive(prim, &transform, &mut temp);
            if let Some(cmd) = temp.into_iter().next() {
                let command_index = commands.len();
                let node_id_str = node_id.as_str().to_string();
                let data_id = prim.data_id().map(str::to_string);
                commands.push(GpuCommandWithProvenance {
                    command: cmd,
                    node_id: Some(node_id_str.clone()),
                    data_id: data_id.clone(),
                    primitive_index,
                });
                provenance.register(GpuProvenanceEntry {
                    command_index,
                    node_id: node_id_str,
                    data_id,
                    primitive_index,
                });
                primitive_index += 1;
            }
        }

        (commands, provenance)
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

    #[test]
    fn compile_with_provenance_empty_scene() {
        let graph = SceneGraph::new();
        let (commands, provenance) = GpuCompiler::new(800.0, 600.0).compile_with_provenance(&graph);
        assert_eq!(commands.len(), 2);
        assert!(matches!(
            commands[0].command,
            GpuDrawCommand::SetViewport { .. }
        ));
        assert!(matches!(commands[1].command, GpuDrawCommand::Clear { .. }));
        assert!(commands[0].node_id.is_none());
        assert!(commands[1].node_id.is_none());
        assert!(provenance.is_empty());
    }

    #[test]
    fn compile_with_provenance_point_produces_entry() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("p").with_primitive(Primitive::Point {
            x: 100.0,
            y: 200.0,
            radius: 5.0,
            fill: Some(Color::WHITE),
            stroke: None,
            data_id: Some("d1".to_string()),
        }));
        let (commands, provenance) = GpuCompiler::new(800.0, 600.0).compile_with_provenance(&graph);
        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[2].command, GpuDrawCommand::Circle { .. }));
        assert_eq!(commands[2].node_id.as_deref(), Some("p"));
        assert_eq!(commands[2].data_id.as_deref(), Some("d1"));
        assert_eq!(commands[2].primitive_index, 0);
        assert_eq!(provenance.len(), 1);
        let entry = provenance.query_command(2).unwrap();
        assert_eq!(entry.node_id, "p");
        assert_eq!(entry.data_id.as_deref(), Some("d1"));
        assert_eq!(entry.primitive_index, 0);
    }

    #[test]
    fn gpu_draw_command_circle_serialization() {
        let cmd = GpuDrawCommand::Circle {
            center: [100.0, 200.0],
            radius: 5.0,
            color: [1.0, 0.0, 0.0, 1.0],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let restored: GpuDrawCommand = serde_json::from_str(&json).unwrap();
        match restored {
            GpuDrawCommand::Circle {
                center,
                radius,
                color,
            } => {
                assert_eq!(center, [100.0, 200.0]);
                assert!((radius - 5.0).abs() < f32::EPSILON);
                assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
            }
            _ => panic!("expected Circle"),
        }
    }

    #[test]
    fn gpu_draw_command_polyline_serialization() {
        let cmd = GpuDrawCommand::Polyline {
            vertices: vec![[0.0, 0.0], [1.0, 1.0]],
            color: [0.0, 1.0, 0.0, 1.0],
            width: 2.0,
            closed: false,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let restored: GpuDrawCommand = serde_json::from_str(&json).unwrap();
        match restored {
            GpuDrawCommand::Polyline {
                vertices,
                color,
                width,
                closed,
            } => {
                assert_eq!(vertices.len(), 2);
                assert_eq!(color, [0.0, 1.0, 0.0, 1.0]);
                assert!((width - 2.0).abs() < f32::EPSILON);
                assert!(!closed);
            }
            _ => panic!("expected Polyline"),
        }
    }

    #[test]
    fn gpu_draw_command_fill_rect_serialization() {
        let cmd = GpuDrawCommand::FillRect {
            min: [10.0, 20.0],
            max: [110.0, 120.0],
            color: [0.0, 0.0, 1.0, 0.5],
            corner_radius: 4.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let restored: GpuDrawCommand = serde_json::from_str(&json).unwrap();
        match restored {
            GpuDrawCommand::FillRect {
                min,
                max,
                color,
                corner_radius,
            } => {
                assert_eq!(min, [10.0, 20.0]);
                assert_eq!(max, [110.0, 120.0]);
                assert_eq!(color[3], 0.5);
                assert!((corner_radius - 4.0).abs() < f32::EPSILON);
            }
            _ => panic!("expected FillRect"),
        }
    }

    #[test]
    fn gpu_draw_command_set_viewport_serialization() {
        let cmd = GpuDrawCommand::SetViewport {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("SetViewport"));
        assert!(json.contains("800"));
    }

    #[test]
    fn gpu_draw_command_clear_serialization() {
        let cmd = GpuDrawCommand::Clear {
            color: [0.1, 0.2, 0.3, 1.0],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let restored: GpuDrawCommand = serde_json::from_str(&json).unwrap();
        match restored {
            GpuDrawCommand::Clear { color } => assert_eq!(color[0], 0.1),
            _ => panic!("expected Clear"),
        }
    }

    #[test]
    fn gpu_provenance_map_is_empty_new() {
        let map = GpuProvenanceMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn gpu_provenance_map_query_nonexistent() {
        let map = GpuProvenanceMap::new();
        assert!(map.query_command(0).is_none());
    }

    #[test]
    fn gpu_provenance_map_register_and_query() {
        let mut map = GpuProvenanceMap::new();
        assert!(map.is_empty());
        map.register(GpuProvenanceEntry {
            command_index: 2,
            node_id: "n1".to_string(),
            data_id: Some("d1".to_string()),
            primitive_index: 0,
        });
        map.register(GpuProvenanceEntry {
            command_index: 3,
            node_id: "n2".to_string(),
            data_id: None,
            primitive_index: 1,
        });
        assert_eq!(map.len(), 2);
        let e = map.query_command(2).unwrap();
        assert_eq!(e.node_id, "n1");
        assert_eq!(e.data_id.as_deref(), Some("d1"));
        assert!(map.query_command(0).is_none());
    }
}
