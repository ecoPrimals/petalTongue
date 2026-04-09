// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use petal_tongue_core::graph_builder::{GraphNode, NodeType, Vec2};

fn make_node(id: &str, x: f32, y: f32) -> GraphNode {
    let mut n = GraphNode::new(NodeType::PrimalStart, Vec2::new(x, y));
    n.id = id.to_string();
    n
}

#[test]
fn hit_test_nodes_empty() {
    let nodes: Vec<GraphNode> = vec![];
    assert!(hit_test_nodes(0.0, 0.0, &nodes, 10.0, 10.0).is_none());
}

#[test]
fn hit_test_nodes_single_hit() {
    let nodes = vec![make_node("a", 100.0, 100.0)];
    assert_eq!(
        hit_test_nodes(100.0, 100.0, &nodes, 20.0, 20.0),
        Some("a".to_string())
    );
}

#[test]
fn hit_test_nodes_single_miss() {
    let nodes = vec![make_node("a", 100.0, 100.0)];
    assert!(hit_test_nodes(150.0, 150.0, &nodes, 20.0, 20.0).is_none());
}

#[test]
fn hit_test_nodes_boundary_inclusive() {
    let nodes = vec![make_node("a", 100.0, 100.0)];
    assert_eq!(
        hit_test_nodes(110.0, 110.0, &nodes, 15.0, 15.0),
        Some("a".to_string())
    );
    assert_eq!(
        hit_test_nodes(114.9, 114.9, &nodes, 15.0, 15.0),
        Some("a".to_string())
    );
}

#[test]
fn hit_test_nodes_first_wins() {
    let nodes = vec![
        make_node("first", 50.0, 50.0),
        make_node("second", 51.0, 51.0),
    ];
    assert_eq!(
        hit_test_nodes(50.5, 50.5, &nodes, 5.0, 5.0),
        Some("first".to_string())
    );
}

#[test]
fn nodes_in_rect_empty() {
    let nodes: Vec<GraphNode> = vec![];
    let cam = Vec2::zero();
    assert!(
        nodes_in_rect(
            [0.0, 0.0],
            [100.0, 100.0],
            &nodes,
            &cam,
            1.0,
            [0.0, 0.0],
            [800.0, 600.0]
        )
        .is_empty()
    );
}

#[test]
fn nodes_in_rect_single_inside() {
    let nodes = vec![make_node("n1", 0.0, 0.0)];
    let cam = Vec2::zero();
    let center_x = 400.0;
    let center_y = 300.0;
    let box_min = [center_x - 10.0, center_y - 10.0];
    let box_max = [center_x + 10.0, center_y + 10.0];
    let canvas_min = [0.0, 0.0];
    let canvas_size = [800.0, 600.0];
    let result = nodes_in_rect(box_min, box_max, &nodes, &cam, 1.0, canvas_min, canvas_size);
    assert_eq!(result, vec!["n1".to_string()]);
}

#[test]
fn nodes_in_rect_box_min_max_swapped() {
    let nodes = vec![make_node("n1", 0.0, 0.0)];
    let cam = Vec2::zero();
    let center_x = 400.0;
    let center_y = 300.0;
    let box_min = [center_x + 10.0, center_y + 10.0];
    let box_max = [center_x - 10.0, center_y - 10.0];
    let canvas_min = [0.0, 0.0];
    let canvas_size = [800.0, 600.0];
    let result = nodes_in_rect(box_min, box_max, &nodes, &cam, 1.0, canvas_min, canvas_size);
    assert_eq!(result, vec!["n1".to_string()]);
}

#[test]
fn nodes_in_rect_outside() {
    let nodes = vec![make_node("n1", 1000.0, 1000.0)];
    let cam = Vec2::zero();
    let box_min = [0.0, 0.0];
    let box_max = [100.0, 100.0];
    let canvas_min = [0.0, 0.0];
    let canvas_size = [800.0, 600.0];
    let result = nodes_in_rect(box_min, box_max, &nodes, &cam, 1.0, canvas_min, canvas_size);
    assert!(result.is_empty());
}

#[test]
fn compute_zoom_identity() {
    assert!((compute_zoom(1.0, 0.0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn compute_zoom_zoom_in() {
    let z = compute_zoom(1.0, 100.0);
    assert!(z > 1.0);
}

#[test]
fn compute_zoom_zoom_out() {
    let z = compute_zoom(1.0, -100.0);
    assert!(z < 1.0);
}

#[test]
fn compute_zoom_clamp_min() {
    let z = compute_zoom(0.25, -10000.0);
    assert_eq!(z, 0.25);
}

#[test]
fn compute_zoom_clamp_max() {
    let z = compute_zoom(3.0, 10000.0);
    assert_eq!(z, 3.0);
}

#[test]
fn test_selection_box_bounds() {
    let (min, max) = selection_box_bounds(10.0, 20.0, 100.0, 80.0);
    assert_eq!(min, [10.0, 20.0]);
    assert_eq!(max, [100.0, 80.0]);

    let (min, max) = selection_box_bounds(100.0, 80.0, 10.0, 20.0);
    assert_eq!(min, [10.0, 20.0]);
    assert_eq!(max, [100.0, 80.0]);
}

#[test]
fn test_compute_pan_camera_position() {
    let pos = compute_pan_camera_position(0.0, 0.0, 100.0, 50.0, 1.0);
    assert!((pos.x - (-100.0)).abs() < f32::EPSILON);
    assert!((pos.y - (-50.0)).abs() < f32::EPSILON);

    let pos = compute_pan_camera_position(10.0, 20.0, 20.0, 40.0, 2.0);
    assert!((pos.x - 0.0).abs() < f32::EPSILON);
    assert!((pos.y - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_compute_dragged_node_position_no_snap() {
    let world = Vec2::new(50.0, 60.0);
    let offset = Vec2::new(10.0, -5.0);
    let result = compute_dragged_node_position(world, offset, false, 20.0);
    assert!((result.x - 60.0).abs() < f32::EPSILON);
    assert!((result.y - 55.0).abs() < f32::EPSILON);
}

#[test]
fn test_compute_dragged_node_position_with_snap() {
    let world = Vec2::new(50.0, 60.0);
    let offset = Vec2::new(10.0, 20.0);
    let result = compute_dragged_node_position(world, offset, true, 20.0);
    assert_eq!(result.x, 60.0);
    assert_eq!(result.y, 80.0);
}

#[test]
fn hit_test_nodes_multiple_first_match() {
    let nodes = vec![
        make_node("a", 50.0, 50.0),
        make_node("b", 55.0, 55.0),
        make_node("c", 60.0, 60.0),
    ];
    let result = hit_test_nodes(52.0, 52.0, &nodes, 10.0, 10.0);
    assert_eq!(result, Some("a".to_string()));
}

#[test]
fn hit_test_nodes_exact_boundary() {
    let nodes = vec![make_node("n", 100.0, 100.0)];
    assert_eq!(
        hit_test_nodes(100.0, 100.0, &nodes, 0.1, 0.1),
        Some("n".to_string())
    );
    assert!(hit_test_nodes(100.2, 100.0, &nodes, 0.1, 0.1).is_none());
}

#[test]
fn nodes_in_rect_multiple_nodes() {
    let nodes = vec![
        make_node("n1", 0.0, 0.0),
        make_node("n2", 50.0, 0.0),
        make_node("n3", 100.0, 100.0),
    ];
    let cam = Vec2::zero();
    let center_x = 400.0;
    let center_y = 300.0;
    let box_min = [center_x - 100.0, center_y - 100.0];
    let box_max = [center_x + 100.0, center_y + 100.0];
    let canvas_min = [0.0, 0.0];
    let canvas_size = [800.0, 600.0];
    let result = nodes_in_rect(box_min, box_max, &nodes, &cam, 1.0, canvas_min, canvas_size);
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"n1".to_string()));
    assert!(result.contains(&"n2".to_string()));
    assert!(result.contains(&"n3".to_string()));
}

#[test]
fn compute_zoom_factor_identity() {
    let factor = 0.0f32.mul_add(0.001, 1.0);
    assert!((factor - 1.0).abs() < f32::EPSILON);
}

#[test]
fn compute_zoom_mid_range() {
    let z = compute_zoom(1.5, 50.0);
    assert!(z > 1.5);
    assert!(z < 3.0);
}

#[test]
fn selection_box_bounds_reversed() {
    let (min, max) = selection_box_bounds(100.0, 50.0, 10.0, 20.0);
    assert_eq!(min, [10.0, 20.0]);
    assert_eq!(max, [100.0, 50.0]);
}

#[test]
fn compute_pan_camera_position_with_zoom() {
    let pos = compute_pan_camera_position(100.0, 200.0, 50.0, 100.0, 2.0);
    assert!((pos.x - 75.0).abs() < f32::EPSILON);
    assert!((pos.y - 150.0).abs() < f32::EPSILON);
}

#[test]
fn compute_dragged_node_position_offset_only() {
    let world = Vec2::new(0.0, 0.0);
    let offset = Vec2::new(-10.0, 5.0);
    let result = compute_dragged_node_position(world, offset, false, 10.0);
    assert!((result.x - (-10.0)).abs() < f32::EPSILON);
    assert!((result.y - 5.0).abs() < f32::EPSILON);
}

#[test]
fn hit_test_nodes_multiple_all_miss() {
    let nodes = vec![make_node("a", 0.0, 0.0), make_node("b", 100.0, 100.0)];
    assert!(hit_test_nodes(50.0, 50.0, &nodes, 5.0, 5.0).is_none());
}

#[test]
fn hit_test_nodes_asymmetric_half_size() {
    let nodes = vec![make_node("n", 50.0, 50.0)];
    assert_eq!(
        hit_test_nodes(50.0, 50.0, &nodes, 100.0, 5.0),
        Some("n".to_string())
    );
    assert_eq!(
        hit_test_nodes(50.0, 50.0, &nodes, 5.0, 100.0),
        Some("n".to_string())
    );
}

#[test]
fn nodes_in_rect_partial_overlap() {
    let nodes = vec![
        make_node("n1", 0.0, 0.0),
        make_node("n2", 200.0, 0.0),
        make_node("n3", 400.0, 0.0),
    ];
    let cam = Vec2::zero();
    let center_x = 400.0;
    let center_y = 300.0;
    let box_min = [center_x - 150.0, center_y - 50.0];
    let box_max = [center_x + 50.0, center_y + 50.0];
    let canvas_min = [0.0, 0.0];
    let canvas_size = [800.0, 600.0];
    let result = nodes_in_rect(box_min, box_max, &nodes, &cam, 1.0, canvas_min, canvas_size);
    assert!(!result.is_empty());
}

#[test]
fn compute_zoom_large_scroll_in() {
    let z = compute_zoom(1.0, 500.0);
    assert!(z > 1.0);
    assert!(z <= 3.0);
}

#[test]
fn compute_zoom_large_scroll_out() {
    let z = compute_zoom(1.0, -500.0);
    assert!(z < 1.0);
    assert!(z >= 0.25);
}

#[test]
fn selection_box_bounds_same_point() {
    let (min, max) = selection_box_bounds(50.0, 50.0, 50.0, 50.0);
    assert_eq!(min, [50.0, 50.0]);
    assert_eq!(max, [50.0, 50.0]);
}

#[test]
fn compute_pan_camera_position_zero_delta() {
    let pos = compute_pan_camera_position(100.0, 200.0, 0.0, 0.0, 1.0);
    assert!((pos.x - 100.0).abs() < f32::EPSILON);
    assert!((pos.y - 200.0).abs() < f32::EPSILON);
}

#[test]
fn compute_dragged_node_position_snap_boundary() {
    let world = Vec2::new(50.0, 50.0);
    let offset = Vec2::new(0.0, 0.0);
    let result = compute_dragged_node_position(world, offset, true, 20.0);
    assert_eq!(result.x, 60.0);
    assert_eq!(result.y, 60.0);
}

#[test]
fn graph_canvas_render_smoke() {
    use crate::accessibility::{ColorPalette, ColorScheme};
    use crate::graph_canvas::GraphCanvas;

    let mut canvas = GraphCanvas::new("test");
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas.render(ui, &palette);
        });
    });
}

#[test]
fn graph_canvas_render_with_nodes() {
    use crate::accessibility::{ColorPalette, ColorScheme};
    use crate::graph_canvas::GraphCanvas;
    use egui::{Pos2, Rect, Vec2 as EguiVec2};
    use petal_tongue_core::graph_builder::NodeType;

    let mut canvas = GraphCanvas::new("test");
    let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));
    canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(400.0, 300.0), canvas_rect);
    canvas.add_node_at_screen(NodeType::Verification, Pos2::new(500.0, 350.0), canvas_rect);

    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas.render(ui, &palette);
        });
    });
}

#[test]
fn graph_canvas_toggle_selection() {
    use crate::graph_canvas::GraphCanvas;

    let mut canvas = GraphCanvas::new("test");
    canvas.toggle_node_selection("n1");
    assert!(canvas.selected_nodes.contains("n1"));
    canvas.toggle_node_selection("n1");
    assert!(!canvas.selected_nodes.contains("n1"));
}

#[test]
fn nodes_in_rect_with_camera_offset() {
    let nodes = vec![make_node("n1", 0.0, 0.0)];
    let cam = Vec2::new(100.0, 100.0);
    let canvas_min = [0.0, 0.0];
    let canvas_size = [800.0, 600.0];
    let center_x = canvas_min[0] + canvas_size[0] / 2.0;
    let center_y = canvas_min[1] + canvas_size[1] / 2.0;
    let screen_x = (0.0 - cam.x) * 1.0 + center_x;
    let screen_y = (0.0 - cam.y) * 1.0 + center_y;
    let box_min = [screen_x - 20.0, screen_y - 20.0];
    let box_max = [screen_x + 20.0, screen_y + 20.0];
    let result = nodes_in_rect(box_min, box_max, &nodes, &cam, 1.0, canvas_min, canvas_size);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "n1");
}

#[test]
fn hit_test_nodes_negative_coords() {
    let nodes = vec![make_node("n", -50.0, -50.0)];
    assert_eq!(
        hit_test_nodes(-50.0, -50.0, &nodes, 10.0, 10.0),
        Some("n".to_string())
    );
}
