// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::modality::{ModalityCompiler, ModalityOutput};
use crate::primitive::{AnchorPoint, Color, FillRule, Primitive, StrokeStyle};
use crate::scene_graph::{SceneGraph, SceneNode};

use super::raster::BrailleCompiler;
use super::types::BrailleCell;

#[test]
fn braille_compiler_produces_grid() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Point {
        x: 400.0,
        y: 300.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: Some("d1".to_string()),
    };
    graph.add_to_root(SceneNode::new("p").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    assert_eq!(grid.len(), 12);
    assert_eq!(grid[0].len(), 40);
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised, "Grid should have at least one raised dot");
}

#[test]
fn braille_cell_to_char_blank() {
    assert_eq!(BrailleCell::BLANK.to_char(), '\u{2800}');
}

#[test]
fn braille_cell_to_char_dot1() {
    let cell = BrailleCell { dots: 1 };
    assert_eq!(cell.to_char(), '\u{2801}');
}

#[test]
fn braille_compiler_with_viewport() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Point {
        x: 400.0,
        y: 300.0,
        radius: 5.0,
        fill: None,
        stroke: None,
        data_id: Some("d1".to_string()),
    };
    graph.add_to_root(SceneNode::new("p").with_primitive(prim));
    let compiler = BrailleCompiler::new(40, 12).with_viewport(1600.0, 1200.0);
    let out = compiler.compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_skips_non_data_primitives() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Point {
        x: 400.0,
        y: 300.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: None,
    };
    graph.add_to_root(SceneNode::new("p").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(!has_raised);
}

#[test]
fn braille_compiler_line() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Line {
        points: vec![[0.0, 0.0], [800.0, 600.0]],
        stroke: StrokeStyle::default(),
        closed: false,
        data_id: Some("line1".to_string()),
    };
    graph.add_to_root(SceneNode::new("l").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_text() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Text {
        x: 100.0,
        y: 100.0,
        content: "Hello".to_string(),
        font_size: 16.0,
        color: Color::BLACK,
        anchor: AnchorPoint::TopLeft,
        bold: false,
        italic: false,
        data_id: Some("text1".to_string()),
    };
    graph.add_to_root(SceneNode::new("t").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_polygon() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Polygon {
        points: vec![[100.0, 100.0], [200.0, 100.0], [150.0, 200.0]],
        fill: Color::BLACK,
        stroke: None,
        fill_rule: FillRule::NonZero,
        data_id: Some("poly1".to_string()),
    };
    graph.add_to_root(SceneNode::new("p").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_arc() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Arc {
        cx: 400.0,
        cy: 300.0,
        radius: 100.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::FRAC_PI_2,
        fill: None,
        stroke: None,
        data_id: Some("arc1".to_string()),
    };
    graph.add_to_root(SceneNode::new("a").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_bezier_path() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::BezierPath {
        start: [100.0, 100.0],
        segments: vec![crate::primitive::BezierSegment {
            cp1: [150.0, 50.0],
            cp2: [250.0, 150.0],
            end: [300.0, 100.0],
        }],
        stroke: StrokeStyle::default(),
        fill: None,
        fill_rule: FillRule::NonZero,
        data_id: Some("path1".to_string()),
    };
    graph.add_to_root(SceneNode::new("b").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_mesh() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Mesh {
        vertices: vec![
            crate::primitive::MeshVertex {
                position: [100.0, 100.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: Color::BLACK,
            },
            crate::primitive::MeshVertex {
                position: [200.0, 100.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: Color::BLACK,
            },
            crate::primitive::MeshVertex {
                position: [150.0, 200.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: Color::BLACK,
            },
        ],
        indices: vec![0, 1, 2],
        data_id: Some("mesh1".to_string()),
    };
    graph.add_to_root(SceneNode::new("m").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}

#[test]
fn braille_compiler_rect() {
    let mut graph = SceneGraph::new();
    let prim = Primitive::Rect {
        x: 100.0,
        y: 100.0,
        width: 200.0,
        height: 150.0,
        fill: Some(Color::BLACK),
        stroke: None,
        corner_radius: 0.0,
        data_id: Some("rect1".to_string()),
    };
    graph.add_to_root(SceneNode::new("r").with_primitive(prim));
    let out = BrailleCompiler::new(40, 12).compile(&graph);
    let ModalityOutput::BrailleCells(grid) = &out else {
        panic!("expected BrailleCells");
    };
    let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
    assert!(has_raised);
}
