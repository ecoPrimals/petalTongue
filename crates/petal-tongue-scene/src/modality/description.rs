// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Write;

use bytes::Bytes;

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;

use super::{ModalityCompiler, ModalityOutput};

/// Compiles scene graph to text description for accessibility.
#[derive(Debug, Clone, Default)]
pub struct DescriptionCompiler;

impl DescriptionCompiler {
    /// Create a new description compiler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ModalityCompiler for DescriptionCompiler {
    fn name(&self) -> &'static str {
        "DescriptionCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let node_count = scene.node_count();
        let prim_count = scene.total_primitives();
        let flat = scene.flatten();
        let mut type_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for (_, prim) in &flat {
            let name = match prim {
                Primitive::Point { .. } => "Point",
                Primitive::Line { .. } => "Line",
                Primitive::Rect { .. } => "Rect",
                Primitive::Text { .. } => "Text",
                Primitive::Polygon { .. } => "Polygon",
                Primitive::Arc { .. } => "Arc",
                Primitive::BezierPath { .. } => "BezierPath",
                Primitive::Mesh { .. } => "Mesh",
            };
            *type_counts.entry(name).or_insert(0) += 1;
        }
        let type_desc: Vec<String> = type_counts
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect();
        let labels: Vec<&str> = flat
            .iter()
            .filter_map(|(_, p)| {
                if let Primitive::Text { content, .. } = p {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();
        let mut desc = format!(
            "Scene with {} nodes and {} primitives. Primitive types: {}.",
            node_count,
            prim_count,
            type_desc.join(", ")
        );
        if !labels.is_empty() {
            let _ = write!(desc, " Labels: {}.", labels.join(", "));
        }
        ModalityOutput::Description(Bytes::from(desc.into_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{AnchorPoint, Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn description_compiler_describes_node_count() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("a"));
        graph.add_to_root(SceneNode::new("b"));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("3 nodes")); // root + a + b
    }

    #[test]
    fn description_compiler_includes_labels() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Text {
            x: 0.0,
            y: 0.0,
            content: "Chart Title".to_string(),
            font_size: 16.0,
            color: Color::BLACK,
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("title").with_primitive(prim));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Chart Title"));
    }
}
