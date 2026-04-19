// SPDX-License-Identifier: AGPL-3.0-or-later

use super::geometry::{
    arrow_geometry, edge_color_rgb, edge_stroke_width, grid_color_alpha, grid_line_positions,
    grid_params, node_colors, node_text_layout,
};
use crate::graph_canvas::{DragState, EdgeDrawState, GraphCanvas};
use petal_tongue_core::graph_builder::{EdgeType, GraphEdge, NodeType};

#[test]
fn test_node_colors_selected() {
    let (fill, stroke) = node_colors(true, false, false);
    assert_eq!(fill, [245, 166, 35]);
    assert_eq!(stroke, [200, 130, 20]);
}

#[test]
fn test_node_colors_hovered() {
    let (fill, stroke) = node_colors(false, true, false);
    assert_eq!(fill, [100, 150, 255]);
    assert_eq!(stroke, [70, 120, 200]);
}

#[test]
fn test_node_colors_error() {
    let (fill, stroke) = node_colors(false, false, true);
    assert_eq!(fill, [208, 2, 27]);
    assert_eq!(stroke, [150, 0, 20]);
}

#[test]
fn test_node_colors_default() {
    let (fill, stroke) = node_colors(false, false, false);
    assert_eq!(fill, [74, 144, 226]);
    assert_eq!(stroke, [50, 100, 180]);
}

#[test]
fn test_node_colors_priority_selected_over_hovered() {
    let (fill, _) = node_colors(true, true, false);
    assert_eq!(fill, [245, 166, 35]);
}

#[test]
fn test_edge_color_rgb_dependency() {
    let accent = [100, 150, 200];
    let rgb = edge_color_rgb(&EdgeType::Dependency, accent);
    assert_eq!(rgb, accent);
}

#[test]
fn test_edge_color_rgb_data_flow() {
    let rgb = edge_color_rgb(&EdgeType::DataFlow, [0, 0, 0]);
    assert_eq!(rgb, [150, 150, 150]);
}

#[test]
fn test_arrow_geometry_normal() {
    let from = [0.0, 0.0];
    let to = [100.0, 0.0];
    let zoom = 1.0;
    let points = arrow_geometry(from, to, zoom);

    assert_eq!(points.tip, to);
    assert!((points.left[0] - 84.0).abs() < f32::EPSILON);
    assert!((points.left[1] - 8.0).abs() < f32::EPSILON);
    assert!((points.right[0] - 84.0).abs() < f32::EPSILON);
    assert!((points.right[1] - (-8.0)).abs() < f32::EPSILON);
}

#[test]
fn test_arrow_geometry_zero_length() {
    let from = [50.0, 50.0];
    let to = [50.0, 50.0];
    let points = arrow_geometry(from, to, 1.0);

    assert_eq!(points.tip, to);
    assert_eq!(points.left, to);
    assert_eq!(points.right, to);
}

#[test]
fn test_arrow_geometry_scales_with_zoom() {
    let from = [0.0, 0.0];
    let to = [100.0, 0.0];
    let points_zoom1 = arrow_geometry(from, to, 1.0);
    let points_zoom2 = arrow_geometry(from, to, 2.0);

    let width1 = (points_zoom1.left[1] - points_zoom1.right[1]).abs();
    let width2 = (points_zoom2.left[1] - points_zoom2.right[1]).abs();
    assert!(width2 > width1);
}

#[test]
fn test_grid_line_positions() {
    let positions = grid_line_positions(0.0, 100.0, 25.0, 0.0);
    assert!(!positions.is_empty());
    assert!((positions[0] - 0.0).abs() < f32::EPSILON);
    let last = positions.last().expect("non-empty");
    assert!(*last < 100.0);
    assert!(*last >= 75.0);
}

#[test]
fn test_grid_line_positions_with_offset() {
    let positions = grid_line_positions(0.0, 50.0, 20.0, 5.0);
    assert!(!positions.is_empty());
    assert!((positions[0] - (-5.0)).abs() < f32::EPSILON);
}

#[test]
fn test_node_colors_selected_over_error() {
    let (fill, _) = node_colors(true, false, true);
    assert_eq!(fill, [245, 166, 35]);
}

#[test]
fn test_node_colors_hovered_over_error() {
    let (fill, _) = node_colors(false, true, true);
    assert_eq!(fill, [100, 150, 255]);
}

#[test]
fn test_arrow_geometry_vertical() {
    let from = [50.0, 0.0];
    let to = [50.0, 100.0];
    let points = arrow_geometry(from, to, 1.0);
    assert_eq!(points.tip, to);
    assert!((points.left[0] - 42.0).abs() < 1.0);
    assert!((points.left[1] - 84.0).abs() < 1.0);
    assert!((points.right[0] - 58.0).abs() < 1.0);
    assert!((points.right[1] - 84.0).abs() < 1.0);
}

#[test]
fn test_grid_line_positions_empty_range() {
    let positions = grid_line_positions(100.0, 50.0, 25.0, 0.0);
    assert!(positions.is_empty());
}

#[test]
fn test_grid_params() {
    let (gs, ox, oy) = grid_params(20.0, 0.0, 0.0, 1.0);
    assert!((gs - 20.0).abs() < f32::EPSILON);
    assert!((ox - 0.0).abs() < f32::EPSILON);
    assert!((oy - 0.0).abs() < f32::EPSILON);

    let (gs, ox, _) = grid_params(20.0, 10.0, 0.0, 2.0);
    assert!((gs - 40.0).abs() < f32::EPSILON);
    assert!((0.0..40.0).contains(&ox));
}

#[test]
fn test_node_text_layout() {
    let (text_size, icon_y, name_y) = node_text_layout(1.0, 100.0, 150.0);
    assert!((text_size - 14.0).abs() < f32::EPSILON);
    assert!((icon_y - 115.0).abs() < f32::EPSILON);
    assert!((name_y - 140.0).abs() < f32::EPSILON);

    let (text_size, _, _) = node_text_layout(2.0, 0.0, 50.0);
    assert!((text_size - 28.0).abs() < f32::EPSILON);
}

#[test]
fn test_grid_color_alpha() {
    assert_eq!(grid_color_alpha(), 20);
}

#[test]
fn test_edge_stroke_width() {
    assert!((edge_stroke_width(1.0) - 2.0).abs() < f32::EPSILON);
    assert!((edge_stroke_width(2.0) - 4.0).abs() < f32::EPSILON);
}

#[test]
fn test_node_text_layout_small_zoom() {
    let (text_size, _, _) = node_text_layout(0.5, 0.0, 50.0);
    assert!((text_size - 7.0).abs() < f32::EPSILON);
}

#[test]
fn test_arrow_points_struct() {
    let from = [0.0, 0.0];
    let to = [100.0, 0.0];
    let points = arrow_geometry(from, to, 1.0);
    assert_eq!(points.tip, to);
    assert_ne!(points.left, points.right);
}

#[test]
fn test_render_canvas_with_nodes_and_edges() {
    use crate::accessibility::ColorScheme;
    use egui::Pos2;

    let mut canvas = GraphCanvas::new("test-graph");
    let canvas_rect = egui::Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0));
    canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(100.0, 100.0), canvas_rect);
    canvas.add_node_at_screen(NodeType::Verification, Pos2::new(200.0, 200.0), canvas_rect);
    let id1 = canvas.graph().nodes[0].id.clone();
    let id2 = canvas.graph().nodes[1].id.clone();
    let _ = canvas.graph_mut().add_edge(GraphEdge::dependency(id1, id2));

    let palette = crate::accessibility::ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas.render(ui, &palette);
        });
    });
}

#[test]
fn test_render_canvas_empty() {
    use crate::accessibility::ColorScheme;

    let mut canvas = GraphCanvas::new("empty-graph");
    let palette = crate::accessibility::ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas.render(ui, &palette);
        });
    });
}

#[test]
fn test_render_canvas_with_drawing_edge() {
    use crate::accessibility::ColorScheme;
    use egui::Pos2;

    let mut canvas = GraphCanvas::new("test-graph");
    let canvas_rect = egui::Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0));
    canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(100.0, 100.0), canvas_rect);
    let from_id = canvas.graph().nodes[0].id.clone();
    canvas.drawing_edge = Some(EdgeDrawState {
        from_node: from_id,
        current_pos: Pos2::new(150.0, 150.0),
    });

    let palette = crate::accessibility::ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas.render(ui, &palette);
        });
    });
}

#[test]
fn test_render_canvas_with_selection_box() {
    use crate::accessibility::ColorScheme;
    use egui::Pos2;

    let mut canvas = GraphCanvas::new("test-graph");
    canvas.drag_state = Some(DragState::SelectBox {
        start: Pos2::new(50.0, 50.0),
        current: Pos2::new(200.0, 200.0),
    });

    let palette = crate::accessibility::ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            canvas.render(ui, &palette);
        });
    });
}
