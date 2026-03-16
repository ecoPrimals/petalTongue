// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Write;

use bytes::Bytes;

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

use super::{ModalityCompiler, ModalityOutput};

/// Compiles scene graph to text description for accessibility.
#[derive(Debug, Clone, Default)]
pub struct DescriptionCompiler;

impl DescriptionCompiler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn primitive_position(prim: &Primitive, transform: &Transform2D) -> (f64, f64) {
        let (x, y) = match prim {
            Primitive::Point { x, y, .. }
            | Primitive::Rect { x, y, .. }
            | Primitive::Text { x, y, .. } => (*x, *y),
            Primitive::Arc { cx, cy, .. } => (*cx, *cy),
            Primitive::Line { points, .. } => points.first().map_or((0.0, 0.0), |p| (p[0], p[1])),
            Primitive::Polygon { points, .. } => {
                points.first().map_or((0.0, 0.0), |p| (p[0], p[1]))
            }
            Primitive::BezierPath { start, .. } => (start[0], start[1]),
            Primitive::Mesh { vertices, .. } => vertices
                .first()
                .map_or((0.0, 0.0), |v| (v.position[0], v.position[1])),
        };
        transform.apply(x, y)
    }

    fn primitive_bounds(prim: &Primitive, transform: &Transform2D) -> Option<(f64, f64, f64, f64)> {
        let (min_x, min_y, max_x, max_y) = match prim {
            Primitive::Point { x, y, radius, .. } => {
                (*x - radius, *y - radius, *x + radius, *y + radius)
            }
            Primitive::Rect {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, x + width, y + height),
            Primitive::Text { x, y, .. } => (*x, *y, *x + 1.0, *y + 1.0),
            Primitive::Line { points, .. } | Primitive::Polygon { points, .. } => {
                if points.is_empty() {
                    return None;
                }
                let mut min_x = points[0][0];
                let mut min_y = points[0][1];
                let mut max_x = points[0][0];
                let mut max_y = points[0][1];
                for p in points.iter().skip(1) {
                    min_x = min_x.min(p[0]);
                    min_y = min_y.min(p[1]);
                    max_x = max_x.max(p[0]);
                    max_y = max_y.max(p[1]);
                }
                (min_x, min_y, max_x, max_y)
            }
            Primitive::Arc { cx, cy, radius, .. } => {
                (cx - radius, cy - radius, cx + radius, cy + radius)
            }
            Primitive::BezierPath {
                start, segments, ..
            } => {
                let mut min_x = start[0];
                let mut min_y = start[1];
                let mut max_x = start[0];
                let mut max_y = start[1];
                for s in segments {
                    for p in [s.cp1, s.cp2, s.end] {
                        min_x = min_x.min(p[0]);
                        min_y = min_y.min(p[1]);
                        max_x = max_x.max(p[0]);
                        max_y = max_y.max(p[1]);
                    }
                }
                (min_x, min_y, max_x, max_y)
            }
            Primitive::Mesh { vertices, .. } => {
                if vertices.is_empty() {
                    return None;
                }
                let mut min_x = vertices[0].position[0];
                let mut min_y = vertices[0].position[1];
                let mut max_x = vertices[0].position[0];
                let mut max_y = vertices[0].position[1];
                for v in vertices.iter().skip(1) {
                    min_x = min_x.min(v.position[0]);
                    min_y = min_y.min(v.position[1]);
                    max_x = max_x.max(v.position[0]);
                    max_y = max_y.max(v.position[1]);
                }
                (min_x, min_y, max_x, max_y)
            }
        };
        let (a_x, a_y) = transform.apply(min_x, min_y);
        let (b_x, b_y) = transform.apply(max_x, max_y);
        Some((a_x.min(b_x), a_y.min(b_y), a_x.max(b_x), a_y.max(b_y)))
    }

    #[expect(
        clippy::missing_const_for_fn,
        reason = "match on &Primitive not const in stable"
    )]
    fn primitive_type_name(prim: &Primitive) -> &'static str {
        match prim {
            Primitive::Point { .. } => "Point",
            Primitive::Line { .. } => "Line",
            Primitive::Rect { .. } => "Rect",
            Primitive::Text { .. } => "Text",
            Primitive::Polygon { .. } => "Polygon",
            Primitive::Arc { .. } => "Arc",
            Primitive::BezierPath { .. } => "BezierPath",
            Primitive::Mesh { .. } => "Mesh",
        }
    }
}

impl ModalityCompiler for DescriptionCompiler {
    fn name(&self) -> &'static str {
        "DescriptionCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let node_count = scene.node_count();
        let prim_count = scene.total_primitives();
        let flat = scene.flatten_with_ids();
        let mut type_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        let mut node_prim_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        let mut bounds: Option<(f64, f64, f64, f64)> = None;
        let mut data_primitives: Vec<((f64, f64), String)> = Vec::new();

        for (transform, prim, node_id) in &flat {
            let name = Self::primitive_type_name(prim);
            *type_counts.entry(name).or_insert(0) += 1;
            *node_prim_counts.entry(node_id.as_str()).or_insert(0) += 1;

            if let Some(b) = Self::primitive_bounds(prim, transform) {
                bounds = Some(bounds.map_or(b, |prev| {
                    (
                        prev.0.min(b.0),
                        prev.1.min(b.1),
                        prev.2.max(b.2),
                        prev.3.max(b.3),
                    )
                }));
            }

            if prim.carries_data() {
                let (x, y) = Self::primitive_position(prim, transform);
                let mut desc = format!(
                    "{} at ({:.0}, {:.0})",
                    Self::primitive_type_name(prim),
                    x,
                    y
                );
                if let Some(id) = prim.data_id() {
                    desc.push_str(" id:");
                    desc.push_str(id);
                }
                data_primitives.push(((x, y), desc));
            }
        }

        data_primitives.sort_by(|a, b| {
            let ((ax, ay), _) = a;
            let ((bx, by), _) = b;
            ay.partial_cmp(by)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| ax.partial_cmp(bx).unwrap_or(std::cmp::Ordering::Equal))
        });

        let type_desc: Vec<String> = type_counts
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect();
        let labels: Vec<&str> = flat
            .iter()
            .filter_map(|(_, p, _)| {
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

        for node_id in scene.node_ids() {
            if let Some(node) = scene.get(node_id)
                && let Some(label) = &node.label
            {
                let count = node_prim_counts.get(node_id).copied().unwrap_or(0);
                let _ = write!(desc, " Node '{label}': {count} primitives.");
            }
        }

        if let Some((min_x, min_y, max_x, max_y)) = bounds {
            let _ = write!(
                desc,
                " Layout spans from ({min_x:.0}, {min_y:.0}) to ({max_x:.0}, {max_y:.0})."
            );
        }

        for (_, prim_desc) in &data_primitives {
            let _ = write!(desc, " {prim_desc}.");
        }

        ModalityOutput::Description(Bytes::from(desc.into_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{AnchorPoint, Color, FillRule, Primitive, StrokeStyle};
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
        assert!(s.contains("3 nodes"));
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

    #[test]
    fn description_compiler_per_node_descriptions() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(
            SceneNode::new("chart")
                .with_label("Chart")
                .with_primitive(Primitive::Point {
                    x: 10.0,
                    y: 20.0,
                    radius: 5.0,
                    fill: None,
                    stroke: None,
                    data_id: None,
                })
                .with_primitive(Primitive::Point {
                    x: 30.0,
                    y: 40.0,
                    radius: 5.0,
                    fill: None,
                    stroke: None,
                    data_id: None,
                }),
        );
        graph.add_to_root(SceneNode::new("axes").with_label("Axes"));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Node 'Chart': 2 primitives"));
        assert!(s.contains("Node 'Axes': 0 primitives"));
    }

    #[test]
    fn description_compiler_spatial_summary() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("r").with_primitive(Primitive::Rect {
            x: 50.0,
            y: 100.0,
            width: 200.0,
            height: 150.0,
            fill: None,
            stroke: None,
            corner_radius: 0.0,
            data_id: None,
        }));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Layout spans from (50, 100) to (250, 250)"));
    }

    #[test]
    fn description_compiler_data_carrying_primitives() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("p").with_primitive(Primitive::Point {
            x: 123.0,
            y: 456.0,
            radius: 5.0,
            fill: None,
            stroke: None,
            data_id: Some("pt-1".to_string()),
        }));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Point at (123, 456)"));
        assert!(s.contains("id:pt-1"));
    }

    #[test]
    fn description_compiler_spatial_ordering() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(
            SceneNode::new("n")
                .with_primitive(Primitive::Point {
                    x: 200.0,
                    y: 100.0,
                    radius: 1.0,
                    fill: None,
                    stroke: None,
                    data_id: Some("b".to_string()),
                })
                .with_primitive(Primitive::Point {
                    x: 100.0,
                    y: 50.0,
                    radius: 1.0,
                    fill: None,
                    stroke: None,
                    data_id: Some("a".to_string()),
                }),
        );
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        let pos_a = s.find("Point at (100, 50)");
        let pos_b = s.find("Point at (200, 100)");
        assert!(pos_a.is_some());
        assert!(pos_b.is_some());
        assert!(pos_a.unwrap() < pos_b.unwrap());
    }

    #[test]
    fn description_compiler_skips_nodes_without_label() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(
            SceneNode::new("unlabeled").with_primitive(Primitive::Point {
                x: 0.0,
                y: 0.0,
                radius: 1.0,
                fill: None,
                stroke: None,
                data_id: None,
            }),
        );
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(!s.contains("Node 'unlabeled'"));
    }

    #[test]
    fn description_compiler_line_data_primitive() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("l").with_primitive(Primitive::Line {
            points: vec![[10.0, 20.0], [100.0, 200.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: Some("line-1".to_string()),
        }));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Line at (10, 20)"));
        assert!(s.contains("id:line-1"));
    }

    #[test]
    fn description_compiler_text_data_primitive() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("t").with_primitive(Primitive::Text {
            x: 50.0,
            y: 75.0,
            content: "Label".to_string(),
            font_size: 12.0,
            color: Color::BLACK,
            anchor: AnchorPoint::Center,
            bold: false,
            italic: false,
            data_id: Some("text-1".to_string()),
        }));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Text at (50, 75)"));
        assert!(s.contains("id:text-1"));
    }

    #[test]
    fn description_compiler_arc_data_primitive() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("a").with_primitive(Primitive::Arc {
            cx: 100.0,
            cy: 200.0,
            radius: 50.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::PI,
            fill: None,
            stroke: None,
            data_id: Some("arc-1".to_string()),
        }));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Arc at (100, 200)"));
        assert!(s.contains("id:arc-1"));
    }

    #[test]
    fn description_compiler_polygon_data_primitive() {
        let mut graph = SceneGraph::new();
        graph.add_to_root(SceneNode::new("p").with_primitive(Primitive::Polygon {
            points: vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("poly-1".to_string()),
        }));
        let out = DescriptionCompiler::new().compile(&graph);
        let ModalityOutput::Description(b) = &out else {
            panic!("expected Description");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        assert!(s.contains("Polygon at (0, 0)"));
        assert!(s.contains("id:poly-1"));
    }
}
