// SPDX-License-Identifier: AGPL-3.0-only
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

// --- Interaction (uses egui) ---

impl GraphCanvas {
    /// Handle input (zoom, keyboard, hover, mouse)
    pub(super) fn handle_input(&mut self, ui: &mut Ui, response: &Response) {
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
    fn handle_mouse_interaction(&mut self, ui: &mut Ui, response: &Response, canvas_rect: Rect) {
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
                    // Click: Select this node only
                    self.clear_selection();
                    self.select_node(hovered);
                }
                // Click on already selected: Start potential drag
            } else {
                // Click on empty space: Clear selection
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
                    // Move node
                    let world_pos = layout::screen_to_world(
                        pointer_pos,
                        canvas_rect,
                        &self.camera.position,
                        self.camera.zoom,
                    );
                    let new_pos = Vec2::new(world_pos.x + offset.x, world_pos.y + offset.y);
                    let final_pos = if self.snap_to_grid {
                        new_pos.snap(self.grid_size)
                    } else {
                        new_pos
                    };

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

                    // Select nodes in box
                    let box_min = [start.x.min(pointer_pos.x), start.y.min(pointer_pos.y)];
                    let box_max = [start.x.max(pointer_pos.x), start.y.max(pointer_pos.y)];
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
                    // Pan camera
                    self.camera.position.x =
                        start_camera_pos.x - response.drag_delta().x / self.camera.zoom;
                    self.camera.position.y =
                        start_camera_pos.y - response.drag_delta().y / self.camera.zoom;
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
    fn update_hovered_node(&mut self, ui: &mut Ui, response: &Response) {
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
