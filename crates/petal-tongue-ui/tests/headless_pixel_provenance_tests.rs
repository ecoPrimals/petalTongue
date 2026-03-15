// SPDX-License-Identifier: AGPL-3.0-only
//! Headless pixel verification tests using ProvenanceBuffer.
//!
//! Verifies that rendered scene primitives produce provenance data
//! at expected pixel locations.

use petal_tongue_scene::primitive::{
    AnchorPoint, Color, FillRule, LineCap, LineJoin, Primitive, StrokeStyle,
};
use petal_tongue_scene::provenance::ProvenanceRenderer;
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
use petal_tongue_scene::transform::Transform2D;

fn point_scene(x: f64, y: f64, radius: f64, data_id: &str) -> SceneGraph {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("point").with_primitive(Primitive::Point {
        x,
        y,
        radius,
        fill: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        stroke: None,
        data_id: Some(data_id.to_string()),
    }));
    scene
}

#[test]
fn provenance_records_point_at_center() {
    let scene = point_scene(50.0, 50.0, 10.0, "test-point");
    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let prov = buf.get(50, 50);
    assert!(
        prov.is_some(),
        "Expected provenance at (50,50) for a point centered there"
    );
    let p = prov.unwrap();
    assert_eq!(p.data_id.as_deref(), Some("test-point"));
    assert_eq!(p.node_id, "point");
}

#[test]
fn provenance_empty_outside_primitive() {
    let scene = point_scene(10.0, 10.0, 3.0, "small-pt");
    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let far_prov = buf.get(90, 90);
    assert!(
        far_prov.is_none(),
        "No provenance expected at (90,90) for a point at (10,10) with r=3"
    );
}

#[test]
fn provenance_line_has_data() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("line-node").with_primitive(Primitive::Line {
        points: vec![[10.0, 10.0], [90.0, 10.0]],
        stroke: StrokeStyle {
            color: Color::rgba(0.0, 1.0, 0.0, 1.0),
            width: 3.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        },
        closed: false,
        data_id: Some("horiz-line".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let mid = buf.get(50, 10);
    assert!(mid.is_some(), "Expected provenance at midpoint of line");
    assert_eq!(mid.unwrap().data_id.as_deref(), Some("horiz-line"));
}

#[test]
fn provenance_rect_fills_region() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("rect-node").with_primitive(Primitive::Rect {
        x: 20.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
        fill: Some(Color::rgba(0.0, 0.0, 1.0, 1.0)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("blue-rect".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let inside = buf.get(30, 30);
    assert!(inside.is_some(), "Expected provenance inside rect");
    assert_eq!(inside.unwrap().data_id.as_deref(), Some("blue-rect"));

    let outside = buf.get(5, 5);
    assert!(outside.is_none(), "No provenance outside rect");
}

#[test]
fn provenance_multiple_nodes_distinct() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("node-a").with_primitive(Primitive::Point {
        x: 20.0,
        y: 20.0,
        radius: 5.0,
        fill: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        stroke: None,
        data_id: Some("data-a".to_string()),
    }));
    scene.add_to_root(SceneNode::new("node-b").with_primitive(Primitive::Point {
        x: 80.0,
        y: 80.0,
        radius: 5.0,
        fill: Some(Color::rgba(0.0, 1.0, 0.0, 1.0)),
        stroke: None,
        data_id: Some("data-b".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let a = buf.get(20, 20);
    let b = buf.get(80, 80);
    assert!(a.is_some());
    assert!(b.is_some());
    assert_eq!(a.unwrap().data_id.as_deref(), Some("data-a"));
    assert_eq!(b.unwrap().data_id.as_deref(), Some("data-b"));
}

#[test]
fn provenance_arc_at_center() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("arc-node").with_primitive(Primitive::Arc {
        cx: 50.0,
        cy: 50.0,
        radius: 20.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::TAU,
        fill: Some(Color::rgba(0.0, 0.0, 1.0, 1.0)),
        stroke: None,
        data_id: Some("arc-data".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let prov = buf.get(50, 50);
    assert!(prov.is_some(), "Expected provenance at arc center (50,50)");
    assert_eq!(prov.unwrap().data_id.as_deref(), Some("arc-data"));
    assert_eq!(prov.unwrap().node_id, "arc-node");
}

#[test]
fn provenance_text_at_position() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("text-node").with_primitive(Primitive::Text {
        x: 25.0,
        y: 40.0,
        content: "Label".to_string(),
        font_size: 12.0,
        color: Color::rgba(0.0, 0.0, 0.0, 1.0),
        anchor: AnchorPoint::TopLeft,
        bold: false,
        italic: false,
        data_id: Some("text-data".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let prov = buf.get(25, 40);
    assert!(
        prov.is_some(),
        "Expected provenance at text position (25,40)"
    );
    assert_eq!(prov.unwrap().data_id.as_deref(), Some("text-data"));
}

#[test]
fn provenance_polygon_triangle_inside() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(
        SceneNode::new("poly-node").with_primitive(Primitive::Polygon {
            points: vec![[10.0, 10.0], [50.0, 10.0], [30.0, 50.0]],
            fill: Color::rgba(0.0, 1.0, 0.0, 1.0),
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("triangle".to_string()),
        }),
    );

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let prov = buf.get(30, 25);
    assert!(
        prov.is_some(),
        "Expected provenance inside triangle at (30,25)"
    );
    assert_eq!(prov.unwrap().data_id.as_deref(), Some("triangle"));
}

#[test]
fn provenance_transformed_node() {
    let mut scene = SceneGraph::new();
    let root_id = scene.root_id().to_string();
    let child = SceneNode::new("translated")
        .with_transform(Transform2D::translate(60.0, 70.0))
        .with_primitive(Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            fill: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            stroke: None,
            data_id: Some("translated-pt".to_string()),
        });
    scene.add_node(child, &root_id);

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 200, 200);

    let prov = buf.get(60, 70);
    assert!(
        prov.is_some(),
        "Expected provenance at transformed position (60,70)"
    );
    assert_eq!(prov.unwrap().data_id.as_deref(), Some("translated-pt"));
}

#[test]
fn provenance_empty_scene_no_data() {
    let scene = SceneGraph::new();
    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 50, 50);

    assert!(buf.get(0, 0).is_none());
    assert!(buf.get(25, 25).is_none());
    assert!(buf.get(49, 49).is_none());
}

#[test]
fn provenance_multi_node_topmost_wins() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("bottom").with_primitive(Primitive::Rect {
        x: 0.0,
        y: 0.0,
        width: 60.0,
        height: 60.0,
        fill: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("bottom-rect".to_string()),
    }));
    scene.add_to_root(SceneNode::new("top").with_primitive(Primitive::Rect {
        x: 20.0,
        y: 20.0,
        width: 40.0,
        height: 40.0,
        fill: Some(Color::rgba(0.0, 0.0, 1.0, 1.0)),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("top-rect".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let overlap = buf.get(40, 40);
    assert!(overlap.is_some());
    assert_eq!(
        overlap.unwrap().node_id,
        "top",
        "Last (topmost) node should win at overlapping pixel"
    );
    assert_eq!(overlap.unwrap().data_id.as_deref(), Some("top-rect"));
}

#[test]
fn provenance_rect_with_corner_radius() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("rounded").with_primitive(Primitive::Rect {
        x: 15.0,
        y: 15.0,
        width: 50.0,
        height: 50.0,
        fill: Some(Color::rgba(0.5, 0.5, 0.5, 1.0)),
        stroke: None,
        corner_radius: 8.0,
        data_id: Some("rounded-rect".to_string()),
    }));

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let inside = buf.get(40, 40);
    assert!(inside.is_some());
    assert_eq!(inside.unwrap().data_id.as_deref(), Some("rounded-rect"));
}

#[test]
fn provenance_nested_transforms() {
    let mut scene = SceneGraph::new();
    let root_id = scene.root_id().to_string();
    let child = SceneNode::new("child")
        .with_transform(Transform2D::translate(30.0, 20.0))
        .with_primitive(Primitive::Point {
            x: 10.0,
            y: 10.0,
            radius: 8.0,
            fill: Some(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            stroke: None,
            data_id: Some("nested-pt".to_string()),
        });
    scene.add_node(child, &root_id);

    let renderer = ProvenanceRenderer;
    let buf = renderer.render(&scene, 100, 100);

    let prov = buf.get(40, 30);
    assert!(prov.is_some());
    assert_eq!(prov.unwrap().data_id.as_deref(), Some("nested-pt"));
}
