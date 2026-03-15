// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(test)]
pub fn rich_test_scene() -> crate::scene_graph::SceneGraph {
    use crate::primitive::{
        AnchorPoint, BezierSegment, Color, FillRule, MeshVertex, Primitive, StrokeStyle,
    };
    use crate::scene_graph::{SceneGraph, SceneNode};
    use crate::transform::Transform2D;

    let mut graph = SceneGraph::new();
    graph.add_to_root(SceneNode::new("point").with_primitive(Primitive::Point {
        x: 100.0,
        y: 200.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: Some("pt".to_string()),
    }));
    graph.add_to_root(SceneNode::new("line").with_primitive(Primitive::Line {
        points: vec![[10.0, 20.0], [50.0, 60.0]],
        stroke: StrokeStyle::default(),
        closed: false,
        data_id: Some("ln".to_string()),
    }));
    graph.add_to_root(SceneNode::new("rect").with_primitive(Primitive::Rect {
        x: 200.0,
        y: 100.0,
        width: 80.0,
        height: 40.0,
        fill: Some(Color::BLACK),
        stroke: None,
        corner_radius: 4.0,
        data_id: Some("rect".to_string()),
    }));
    graph.add_to_root(SceneNode::new("text").with_primitive(Primitive::Text {
        x: 50.0,
        y: 50.0,
        content: "Hello".to_string(),
        font_size: 12.0,
        color: Color::BLACK,
        anchor: AnchorPoint::TopLeft,
        bold: false,
        italic: false,
        data_id: Some("txt".to_string()),
    }));
    graph.add_to_root(
        SceneNode::new("polygon").with_primitive(Primitive::Polygon {
            points: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("poly".to_string()),
        }),
    );
    graph.add_to_root(SceneNode::new("arc").with_primitive(Primitive::Arc {
        cx: 300.0,
        cy: 300.0,
        radius: 50.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::PI,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: Some("arc".to_string()),
    }));
    graph.add_to_root(
        SceneNode::new("bezier").with_primitive(Primitive::BezierPath {
            start: [0.0, 0.0],
            segments: vec![BezierSegment {
                cp1: [0.0, 50.0],
                cp2: [100.0, 50.0],
                end: [100.0, 0.0],
            }],
            stroke: StrokeStyle::default(),
            fill: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("bez".to_string()),
        }),
    );
    graph.add_to_root(SceneNode::new("mesh").with_primitive(Primitive::Mesh {
        vertices: vec![MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            color: Color::WHITE,
        }],
        indices: vec![0, 1, 2],
        data_id: Some("mesh".to_string()),
    }));
    let root_id: String = graph.root_id().into();
    let child = SceneNode::new("child")
        .with_transform(Transform2D::translate(10.0, 20.0))
        .with_primitive(Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 2.0,
            fill: None,
            stroke: None,
            data_id: Some("child_pt".to_string()),
        });
    graph.add_node(child, root_id.as_str());
    graph
}
