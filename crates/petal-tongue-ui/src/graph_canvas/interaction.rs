// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph canvas interaction handling - mouse, keyboard, drag, selection.

use egui::{Rect, Response, Ui};
use petal_tongue_core::graph_builder::{GraphEdge, GraphNode, Vec2};

use super::layout;
use super::{DragState, EdgeDrawState, GraphCanvas};

// --- Pure logic (testable, no egui) ---

/// Hit-test: find node at world position. Returns first matching node ID.
#[must_use]
pub fn hit_test_nodes(
    world_x: f32,
    world_y: f32,
    nodes: &[GraphNode],
    half_width: f32,
    half_height: f32,
) -> Option<String> {
    for node in nodes {
        let dx = (node.position.x - world_x).abs();
        let dy = (node.position.y - world_y).abs();
        if dx < half_width && dy < half_height {
            return Some(node.id.clone());
        }
    }
    None
}

/// Nodes whose screen positions fall inside the selection box.
#[must_use]
pub fn nodes_in_rect(
    box_min: [f32; 2],
    box_max: [f32; 2],
    nodes: &[GraphNode],
    camera_pos: &Vec2,
    zoom: f32,
    canvas_rect_min: [f32; 2],
    canvas_rect_size: [f32; 2],
) -> Vec<String> {
    let center_x = canvas_rect_min[0] + canvas_rect_size[0] / 2.0;
    let center_y = canvas_rect_min[1] + canvas_rect_size[1] / 2.0;

    let min_x = box_min[0].min(box_max[0]);
    let max_x = box_min[0].max(box_max[0]);
    let min_y = box_min[1].min(box_max[1]);
    let max_y = box_min[1].max(box_max[1]);

    let mut result = Vec::new();
    for node in nodes {
        let screen_x = (node.position.x - camera_pos.x).mul_add(zoom, center_x);
        let screen_y = (node.position.y - camera_pos.y).mul_add(zoom, center_y);
        if screen_x >= min_x && screen_x <= max_x && screen_y >= min_y && screen_y <= max_y {
            result.push(node.id.clone());
        }
    }
    result
}

/// Compute new zoom from scroll delta.
#[must_use]
pub fn compute_zoom(current_zoom: f32, scroll_delta: f32) -> f32 {
    let zoom_factor = scroll_delta.mul_add(0.001, 1.0);
    (current_zoom * zoom_factor).clamp(0.25, 3.0)
}

/// Selection box screen bounds from start and end points.
#[must_use]
pub const fn selection_box_bounds(
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> ([f32; 2], [f32; 2]) {
    (
        [start_x.min(end_x), start_y.min(end_y)],
        [start_x.max(end_x), start_y.max(end_y)],
    )
}

/// Compute pan camera position from start and drag delta.
#[must_use]
pub fn compute_pan_camera_position(
    start_x: f32,
    start_y: f32,
    delta_x: f32,
    delta_y: f32,
    zoom: f32,
) -> Vec2 {
    Vec2::new(start_x - delta_x / zoom, start_y - delta_y / zoom)
}

/// Compute final node position after drag, with optional grid snap.
#[must_use]
pub fn compute_dragged_node_position(
    world_pos: Vec2,
    offset: Vec2,
    snap_to_grid: bool,
    grid_size: f32,
) -> Vec2 {
    let new_pos = Vec2::new(world_pos.x + offset.x, world_pos.y + offset.y);
    if snap_to_grid {
        new_pos.snap(grid_size)
    } else {
        new_pos
    }
}

// --- Interaction (uses egui) ---

impl GraphCanvas {
    /// Handle input (zoom, keyboard, hover, mouse)
    pub(super) fn handle_input(&mut self, ui: &Ui, response: &Response) {
        let canvas_rect = response.rect;

        // Zoom with scroll wheel
        if response.hovered() {
            ui.input(|i| {
                let scroll_delta = i.raw_scroll_delta.y;
                if scroll_delta != 0.0 {
                    self.camera.zoom = compute_zoom(self.camera.zoom, scroll_delta);
                }
            });
        }

        // Handle keyboard shortcuts
        ui.input(|i| {
            // Delete selected nodes (Delete key)
            if i.key_pressed(egui::Key::Delete) && !self.selected_nodes.is_empty() {
                self.delete_selected();
            }

            // Select all (Ctrl+A)
            if i.key_pressed(egui::Key::A) && i.modifiers.command {
                for node in &self.graph.nodes {
                    self.selected_nodes.insert(node.id.clone());
                }
            }

            // Deselect all (Escape)
            if i.key_pressed(egui::Key::Escape) {
                self.clear_selection();
                self.drawing_edge = None;
            }
        });

        // Update hovered node
        self.update_hovered_node(ui, response);

        // Handle mouse clicks and drags
        self.handle_mouse_interaction(ui, response, canvas_rect);
    }

    /// Handle mouse interactions (clicks, drags)
    fn handle_mouse_interaction(&mut self, ui: &Ui, response: &Response, canvas_rect: Rect) {
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
        let shift_held = ui.input(|i| i.modifiers.shift);

        // Left click to select/deselect nodes
        if response.clicked() {
            let hovered_clone = self.hovered_node.clone();
            if let Some(hovered) = hovered_clone {
                if ctrl_held {
                    // Ctrl+Click: Toggle selection
                    self.toggle_node_selection(hovered);
                } else if !self.selected_nodes.contains(&hovered) {
                    // Activate: Select this node only
                    self.clear_selection();
                    self.select_node(hovered);
                }
                // Activate on already selected: Start potential drag
            } else {
                // Activate on empty space: Clear selection
                if !ctrl_held {
                    self.clear_selection();
                }
            }
        }

        // Handle drag start
        if response.drag_started()
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            if ctrl_held {
                // Ctrl+Drag: Start drawing edge
                if let Some(hovered) = &self.hovered_node {
                    self.drawing_edge = Some(EdgeDrawState {
                        from_node: hovered.clone(),
                        current_pos: pointer_pos,
                    });
                }
            } else if let Some(hovered) = &self.hovered_node {
                // Drag node
                if let Some(node) = self.graph.get_node(hovered) {
                    let world_pos = layout::screen_to_world(
                        pointer_pos,
                        canvas_rect,
                        &self.camera.position,
                        self.camera.zoom,
                    );
                    let offset =
                        Vec2::new(node.position.x - world_pos.x, node.position.y - world_pos.y);
                    self.drag_state = Some(DragState::Node {
                        node_id: hovered.clone(),
                        offset,
                    });
                }
            } else if shift_held {
                // Shift+Drag: Pan camera
                self.drag_state = Some(DragState::Pan {
                    start_camera_pos: self.camera.position,
                });
            } else {
                // Drag on empty space: Selection box
                self.drag_state = Some(DragState::SelectBox {
                    start: pointer_pos,
                    current: pointer_pos,
                });
            }
        }

        // Handle dragging
        if response.dragged()
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            // Extract drag state to avoid borrow issues
            let drag_state_clone = self.drag_state.clone();

            match drag_state_clone {
                Some(DragState::Node { node_id, offset }) => {
                    let world_pos = layout::screen_to_world(
                        pointer_pos,
                        canvas_rect,
                        &self.camera.position,
                        self.camera.zoom,
                    );
                    let final_pos = compute_dragged_node_position(
                        world_pos,
                        offset,
                        self.snap_to_grid,
                        self.grid_size,
                    );

                    if let Some(node) = self.graph.get_node_mut(&node_id) {
                        node.position = final_pos;
                    }
                }
                Some(DragState::SelectBox { start, current: _ }) => {
                    // Update selection box
                    self.drag_state = Some(DragState::SelectBox {
                        start,
                        current: pointer_pos,
                    });

                    let (box_min, box_max) =
                        selection_box_bounds(start.x, start.y, pointer_pos.x, pointer_pos.y);
                    let canvas_min = [canvas_rect.min.x, canvas_rect.min.y];
                    let canvas_size = [canvas_rect.width(), canvas_rect.height()];
                    let node_ids = nodes_in_rect(
                        box_min,
                        box_max,
                        &self.graph.nodes,
                        &self.camera.position,
                        self.camera.zoom,
                        canvas_min,
                        canvas_size,
                    );
                    if !ctrl_held {
                        self.clear_selection();
                    }
                    for id in node_ids {
                        self.selected_nodes.insert(id);
                    }
                }
                Some(DragState::Pan { start_camera_pos }) => {
                    let delta = response.drag_delta();
                    self.camera.position = compute_pan_camera_position(
                        start_camera_pos.x,
                        start_camera_pos.y,
                        delta.x,
                        delta.y,
                        self.camera.zoom,
                    );
                }
                None => {}
            }

            // Update edge drawing position
            if let Some(edge_state) = &mut self.drawing_edge {
                edge_state.current_pos = pointer_pos;
            }
        }

        // Handle drag released
        if response.drag_stopped() {
            // If we were drawing an edge, try to create it
            if let Some(edge_state) = &self.drawing_edge {
                if let Some(target_node) = &self.hovered_node {
                    // Create edge from source to target
                    let edge =
                        GraphEdge::dependency(edge_state.from_node.clone(), target_node.clone());
                    if let Err(e) = self.graph.add_edge(edge) {
                        tracing::warn!("Failed to create edge: {}", e);
                    }
                }
                self.drawing_edge = None;
            }

            // Clear drag state
            self.drag_state = None;
        }
    }

    /// Update which node is being hovered
    fn update_hovered_node(&mut self, ui: &Ui, response: &Response) {
        self.hovered_node = None;

        if let Some(pointer_pos) = response.hover_pos() {
            let world_pos = layout::screen_to_world(
                pointer_pos,
                response.rect,
                &self.camera.position,
                self.camera.zoom,
            );

            let half_width = self.node_size.x / (2.0 * self.camera.zoom);
            let half_height = self.node_size.y / (2.0 * self.camera.zoom);

            if let Some(id) = hit_test_nodes(
                world_pos.x,
                world_pos.y,
                &self.graph.nodes,
                half_width,
                half_height,
            ) {
                self.hovered_node = Some(id);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
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
}
