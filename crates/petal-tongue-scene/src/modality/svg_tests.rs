// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::modality::test_utils::rich_test_scene;
use crate::primitive::{Color, Primitive};
use crate::scene_graph::{SceneGraph, SceneNode};
use crate::transform::Transform2D;
use proptest::prelude::*;

#[test]
fn svg_compiler_empty_scene() {
    let graph = SceneGraph::new();
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("</svg>"));
}

#[test]
fn svg_compiler_point_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Point {
        x: 10.0,
        y: 20.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("p").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<circle"));
}

#[test]
fn svg_compiler_line_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Line {
        points: vec![[10.0, 20.0], [50.0, 60.0]],
        stroke: crate::primitive::StrokeStyle::default(),
        closed: false,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("line").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<polyline"));
}

#[test]
fn svg_compiler_rect_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Rect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        fill: Some(Color::BLACK),
        stroke: None,
        corner_radius: 0.0,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("rect").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<rect"));
}

#[test]
fn svg_compiler_text_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Text {
        x: 0.0,
        y: 0.0,
        content: "Test".to_string(),
        font_size: 12.0,
        color: Color::BLACK,
        anchor: crate::primitive::AnchorPoint::TopLeft,
        bold: false,
        italic: false,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("t").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<text"));
    assert!(s.contains("Test"));
}

#[test]
fn svg_compiler_polygon_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Polygon {
        points: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        fill: Color::BLACK,
        stroke: None,
        fill_rule: crate::primitive::FillRule::NonZero,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("poly").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<polygon"));
}

#[test]
fn svg_compiler_arc_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Arc {
        cx: 100.0,
        cy: 100.0,
        radius: 50.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::PI,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("arc").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<path"));
}

#[test]
fn svg_compiler_bezier_primitive() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::BezierPath {
        start: [0.0, 0.0],
        segments: vec![crate::primitive::BezierSegment {
            cp1: [0.0, 50.0],
            cp2: [100.0, 50.0],
            end: [100.0, 0.0],
        }],
        stroke: crate::primitive::StrokeStyle::default(),
        fill: None,
        fill_rule: crate::primitive::FillRule::NonZero,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("bezier").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<path"));
}

#[test]
fn svg_compiler_mesh_skipped() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Mesh {
        vertices: vec![crate::primitive::MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            color: Color::WHITE,
        }],
        indices: vec![0, 1, 2],
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("mesh").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<svg"));
    assert!(!s.contains("<mesh"));
}

#[test]
fn svg_compiler_multiple_node_scene() {
    let scene = rich_test_scene();
    let out = SvgCompiler::new().compile(&scene);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<circle"));
    assert!(s.contains("<polyline"));
    assert!(s.contains("<rect"));
    assert!(s.contains("<text"));
    assert!(s.contains("<polygon"));
    assert!(s.contains("<path"));
}

#[test]
fn svg_compiler_scene_with_transforms() {
    let mut graph = SceneGraph::new();
    let node = SceneNode::new("transformed")
        .with_transform(Transform2D::translate(50.0, 100.0))
        .with_primitive(Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 10.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        });
    graph.add_to_root(node);
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<circle"));
    assert!(s.contains("50"));
    assert!(s.contains("100"));
}

#[test]
fn svg_compiler_produces_valid_svg() {
    let compiler = SvgCompiler::new();
    let graph = SceneGraph::new();
    let out = compiler.compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("</svg>"));
}

#[test]
fn svg_compiler_handles_point_as_circle() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Point {
        x: 10.0,
        y: 20.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("p").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<circle"));
}

#[test]
fn svg_compiler_handles_line_as_polyline() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Line {
        points: vec![[10.0, 20.0], [50.0, 60.0], [90.0, 30.0]],
        stroke: crate::primitive::StrokeStyle::default(),
        closed: false,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("line").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<polyline"));
    assert!(s.contains("10,20"));
    assert!(s.contains("90,30"));
}

#[test]
fn svg_compiler_handles_rect() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Rect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        fill: Some(Color::BLACK),
        stroke: None,
        corner_radius: 5.0,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("rect").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<rect"));
    assert!(s.contains("width=\"100\""));
    assert!(s.contains("height=\"50\""));
    assert!(s.contains("rx=\"5\""));
}

#[test]
fn svg_compiler_handles_polygon() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Polygon {
        points: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        fill: Color::BLACK,
        stroke: None,
        fill_rule: crate::primitive::FillRule::NonZero,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("poly").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<polygon"));
    assert!(s.contains("fill-rule=\"nonzero\""));
}

#[test]
fn svg_compiler_handles_arc() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Arc {
        cx: 100.0,
        cy: 100.0,
        radius: 50.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::PI,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("arc").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<path"));
    assert!(s.contains("d="));
}

#[test]
fn svg_compiler_handles_bezier_path() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::BezierPath {
        start: [0.0, 0.0],
        segments: vec![crate::primitive::BezierSegment {
            cp1: [0.0, 50.0],
            cp2: [100.0, 50.0],
            end: [100.0, 0.0],
        }],
        stroke: crate::primitive::StrokeStyle::default(),
        fill: None,
        fill_rule: crate::primitive::FillRule::NonZero,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("bezier").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("<path"));
    assert!(s.contains("d="));
}

#[test]
fn svg_compiler_escapes_text_content() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Text {
        x: 0.0,
        y: 0.0,
        content: "A & B < C > D".to_string(),
        font_size: 12.0,
        color: Color::BLACK,
        anchor: crate::primitive::AnchorPoint::TopLeft,
        bold: false,
        italic: false,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("t").with_primitive(prim));
    let out = SvgCompiler::new().compile(&graph);
    let ModalityOutput::Svg(b) = &out else {
        panic!("expected Svg");
    };
    let s = std::str::from_utf8(b.as_ref()).unwrap();
    assert!(s.contains("&amp;"));
    assert!(s.contains("&lt;"));
    assert!(s.contains("&gt;"));
}

proptest! {
    /// SVG compiler always produces valid XML wrapping: starts with <svg, ends with </svg>.
    #[test]
    fn prop_svg_valid_xml_wrapping(x in 0.0f64..800.0, y in 0.0f64..600.0, radius in 0.1f64..50.0) {
        let compiler = SvgCompiler::new();
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x,
            y,
            radius,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = compiler.compile(&graph);
        let ModalityOutput::Svg(b) = &out else {
            panic!("expected Svg output");
        };
        let s = std::str::from_utf8(b.as_ref()).unwrap();
        prop_assert!(s.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
        prop_assert!(s.ends_with("</svg>"));
    }
}
