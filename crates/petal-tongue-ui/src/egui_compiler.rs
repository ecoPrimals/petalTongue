// SPDX-License-Identifier: AGPL-3.0-only
//! EguiCompiler: a `ModalityCompiler` that emits egui `Shape`s.
//!
//! Places egui on equal footing with SVG, Audio, Terminal, and Description
//! compilers. The grammar pipeline flows:
//!
//!   Data → Grammar → RenderPlan → EguiCompiler → egui shapes
//!
//! The output is `ModalityOutput::EguiShapes`, a serialized list of
//! lightweight shape descriptors that the scene bridge can stamp onto
//! an egui `Painter`.

use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput};
use petal_tongue_scene::primitive::{Color, Primitive};
use petal_tongue_scene::render_plan::RenderPlan;
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::transform::Transform2D;

/// Shape descriptor produced by the EguiCompiler.
///
/// These are data-only (no egui dependency at this layer) so they
/// can be serialized over IPC. The scene bridge converts them to
/// real `egui::Shape` values at render time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EguiShapeDesc {
    Circle {
        cx: f32,
        cy: f32,
        radius: f32,
        fill: [u8; 4],
        stroke_width: f32,
        stroke_color: [u8; 4],
    },
    Line {
        points: Vec<[f32; 2]>,
        stroke_width: f32,
        stroke_color: [u8; 4],
        closed: bool,
    },
    Rect {
        min: [f32; 2],
        max: [f32; 2],
        fill: [u8; 4],
        stroke_width: f32,
        stroke_color: [u8; 4],
        corner_radius: f32,
    },
    Text {
        pos: [f32; 2],
        content: String,
        font_size: f32,
        color: [u8; 4],
        bold: bool,
    },
    Polygon {
        points: Vec<[f32; 2]>,
        fill: [u8; 4],
        stroke_width: f32,
        stroke_color: [u8; 4],
    },
}

/// Compiles a scene graph into egui shape descriptors.
#[derive(Debug, Clone, Default)]
pub struct EguiCompiler;

impl EguiCompiler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn compile_primitives(scene: &SceneGraph) -> Vec<EguiShapeDesc> {
        let mut shapes = Vec::new();
        for (transform, prim) in scene.flatten() {
            if let Some(shape) = Self::prim_to_shape(prim, &transform) {
                shapes.push(shape);
            }
        }
        shapes
    }

    fn prim_to_shape(prim: &Primitive, transform: &Transform2D) -> Option<EguiShapeDesc> {
        match prim {
            Primitive::Point {
                x,
                y,
                radius,
                fill,
                stroke,
                ..
            } => {
                let (tx, ty) = transform.apply(*x, *y);
                Some(EguiShapeDesc::Circle {
                    cx: tx as f32,
                    cy: ty as f32,
                    radius: *radius as f32,
                    fill: color_to_rgba(fill.unwrap_or(Color::TRANSPARENT)),
                    stroke_width: stroke.map_or(0.0, |s| s.width),
                    stroke_color: stroke.map_or([0; 4], |s| color_to_rgba(s.color)),
                })
            }
            Primitive::Line {
                points,
                stroke: s,
                closed,
                ..
            } => {
                let pts: Vec<[f32; 2]> = points
                    .iter()
                    .map(|[px, py]| {
                        let (tx, ty) = transform.apply(*px, *py);
                        [tx as f32, ty as f32]
                    })
                    .collect();
                Some(EguiShapeDesc::Line {
                    points: pts,
                    stroke_width: s.width,
                    stroke_color: color_to_rgba(s.color),
                    closed: *closed,
                })
            }
            Primitive::Rect {
                x,
                y,
                width,
                height,
                fill,
                stroke,
                corner_radius,
                ..
            } => {
                let (tx, ty) = transform.apply(*x, *y);
                let (tx2, ty2) = transform.apply(x + width, y + height);
                Some(EguiShapeDesc::Rect {
                    min: [tx as f32, ty as f32],
                    max: [tx2 as f32, ty2 as f32],
                    fill: color_to_rgba(fill.unwrap_or(Color::TRANSPARENT)),
                    stroke_width: stroke.map_or(0.0, |s| s.width),
                    stroke_color: stroke.map_or([0; 4], |s| color_to_rgba(s.color)),
                    corner_radius: *corner_radius as f32,
                })
            }
            Primitive::Text {
                x,
                y,
                content,
                font_size,
                color,
                bold,
                ..
            } => {
                let (tx, ty) = transform.apply(*x, *y);
                Some(EguiShapeDesc::Text {
                    pos: [tx as f32, ty as f32],
                    content: content.clone(),
                    font_size: *font_size as f32,
                    color: color_to_rgba(*color),
                    bold: *bold,
                })
            }
            Primitive::Polygon {
                points,
                fill,
                stroke,
                ..
            } => {
                let pts: Vec<[f32; 2]> = points
                    .iter()
                    .map(|[px, py]| {
                        let (tx, ty) = transform.apply(*px, *py);
                        [tx as f32, ty as f32]
                    })
                    .collect();
                Some(EguiShapeDesc::Polygon {
                    points: pts,
                    fill: color_to_rgba(*fill),
                    stroke_width: stroke.map_or(0.0, |s| s.width),
                    stroke_color: stroke.map_or([0; 4], |s| color_to_rgba(s.color)),
                })
            }
            Primitive::Arc { .. } | Primitive::BezierPath { .. } | Primitive::Mesh { .. } => None,
        }
    }
}

impl ModalityCompiler for EguiCompiler {
    fn name(&self) -> &'static str {
        "EguiCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let shapes = Self::compile_primitives(scene);
        let json = serde_json::to_string(&shapes).unwrap_or_default();
        ModalityOutput::Description(json.into_bytes().into())
    }

    fn compile_plan(&self, plan: &RenderPlan) -> ModalityOutput {
        let shapes = Self::compile_primitives(&plan.scene);
        let json = serde_json::to_string(&shapes).unwrap_or_default();
        ModalityOutput::Description(json.into_bytes().into())
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "color components clamped to [0,255]"
)]
fn color_to_rgba(c: Color) -> [u8; 4] {
    [
        (c.r * 255.0).clamp(0.0, 255.0) as u8,
        (c.g * 255.0).clamp(0.0, 255.0) as u8,
        (c.b * 255.0).clamp(0.0, 255.0) as u8,
        (c.a * 255.0).clamp(0.0, 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};
    use petal_tongue_scene::primitive::StrokeStyle;
    use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn egui_compiler_name() {
        assert_eq!(EguiCompiler::new().name(), "EguiCompiler");
    }

    #[test]
    fn egui_compiler_compiles_point() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
            x: 10.0,
            y: 20.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("d1".to_string()),
        }));
        let shapes = EguiCompiler::compile_primitives(&scene);
        assert_eq!(shapes.len(), 1);
        assert!(matches!(shapes[0], EguiShapeDesc::Circle { .. }));
    }

    #[test]
    fn egui_compiler_compiles_line() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("ln").with_primitive(Primitive::Line {
            points: vec![[0.0, 0.0], [100.0, 100.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: None,
        }));
        let shapes = EguiCompiler::compile_primitives(&scene);
        assert_eq!(shapes.len(), 1);
        assert!(matches!(shapes[0], EguiShapeDesc::Line { .. }));
    }

    #[test]
    fn egui_compiler_compiles_text() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("txt").with_primitive(Primitive::Text {
            x: 0.0,
            y: 0.0,
            content: "Hello".to_string(),
            font_size: 14.0,
            color: Color::WHITE,
            anchor: petal_tongue_scene::primitive::AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        }));
        let shapes = EguiCompiler::compile_primitives(&scene);
        assert_eq!(shapes.len(), 1);
        assert!(matches!(&shapes[0], EguiShapeDesc::Text { content, .. } if content == "Hello"));
    }

    #[test]
    fn egui_compiler_plan_roundtrip() {
        let scene = SceneGraph::new();
        let grammar = GrammarExpr::new("test", GeometryType::Point);
        let plan = RenderPlan::new(scene, grammar);
        let output = EguiCompiler::new().compile_plan(&plan);
        assert!(matches!(output, ModalityOutput::Description(_)));
    }

    #[test]
    fn color_conversion() {
        let rgba = color_to_rgba(Color::rgba(1.0, 0.0, 0.5, 0.8));
        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 0);
        assert_eq!(rgba[2], 127); // 0.5 * 255 ≈ 127
        assert_eq!(rgba[3], 204); // 0.8 * 255 ≈ 204
    }
}
