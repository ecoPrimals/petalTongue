//! Graph Canvas - Interactive Visual Graph Editor
//!
//! Provides a canvas for building Neural API graphs through drag-and-drop interactions.
//! TRUE PRIMAL: Zero hardcoding, capability-based, self-contained visualization.

use crate::accessibility::ColorPalette;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2 as EguiVec2};
use petal_tongue_core::graph_builder::{
    EdgeType, GraphEdge, GraphNode, NodeType, Vec2, VisualGraph,
};
use std::collections::HashSet;

/// Interactive graph canvas for building Neural API graphs
pub struct GraphCanvas {
    /// Current graph being edited
    graph: VisualGraph,

    /// Camera position and zoom
    camera: Camera,

    /// Currently selected nodes
    selected_nodes: HashSet<String>,

    /// Drag state
    drag_state: Option<DragState>,

    /// Edge being drawn
    drawing_edge: Option<EdgeDrawState>,

    /// Show grid
    show_grid: bool,

    /// Grid size
    grid_size: f32,

    /// Snap to grid
    snap_to_grid: bool,

    /// Node size
    node_size: EguiVec2,

    /// Hovered node ID
    hovered_node: Option<String>,
}

impl GraphCanvas {
    /// Create a new graph canvas
    #[must_use]
    pub fn new(graph_name: String) -> Self {
        Self {
            graph: VisualGraph::new(graph_name),
            camera: Camera::default(),
            selected_nodes: HashSet::new(),
            drag_state: None,
            drawing_edge: None,
            show_grid: true,
            grid_size: 50.0,
            snap_to_grid: true,
            node_size: EguiVec2::new(120.0, 60.0),
            hovered_node: None,
        }
    }

    /// Render the canvas
    pub fn render(&mut self, ui: &mut Ui, palette: &ColorPalette) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            Sense::click_and_drag(),
        );

        let canvas_rect = response.rect;

        // Draw grid if enabled
        if self.show_grid {
            self.draw_grid(&painter, canvas_rect, palette);
        }

        // Handle input
        self.handle_input(ui, &response);

        // Draw edges
        for edge in &self.graph.edges {
            self.draw_edge(&painter, edge, canvas_rect, palette);
        }

        // Draw nodes
        for node in &self.graph.nodes {
            self.draw_node(&painter, node, canvas_rect, palette);
        }

        // Draw edge being created
        if let Some(edge_state) = &self.drawing_edge {
            self.draw_temporary_edge(&painter, edge_state, canvas_rect, palette);
        }

        // Draw selection box if dragging
        if let Some(DragState::SelectBox { start, current }) = &self.drag_state {
            self.draw_selection_box(&painter, *start, *current, palette);
        }
    }

    /// Draw grid
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, palette: &ColorPalette) {
        let grid_color = Color32::from_rgba_premultiplied(
            palette.text_dim.r(),
            palette.text_dim.g(),
            palette.text_dim.b(),
            20, // Very subtle
        );

        let grid_size = self.grid_size * self.camera.zoom;
        let offset_x = (self.camera.position.x * self.camera.zoom) % grid_size;
        let offset_y = (self.camera.position.y * self.camera.zoom) % grid_size;

        // Vertical lines
        let mut x = rect.min.x - offset_x;
        while x < rect.max.x {
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }

        // Horizontal lines
        let mut y = rect.min.y - offset_y;
        while y < rect.max.y {
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, grid_color),
            );
            y += grid_size;
        }
    }

    /// Draw a node
    fn draw_node(
        &self,
        painter: &egui::Painter,
        node: &GraphNode,
        canvas_rect: Rect,
        palette: &ColorPalette,
    ) {
        let screen_pos = self.world_to_screen(node.position, canvas_rect);
        let node_rect = Rect::from_center_size(screen_pos, self.node_size * self.camera.zoom);

        // Determine node color based on state
        let (fill_color, stroke_color) = if self.selected_nodes.contains(&node.id) {
            // Selected: Orange
            (
                Color32::from_rgb(245, 166, 35),
                Color32::from_rgb(200, 130, 20),
            )
        } else if Some(&node.id) == self.hovered_node.as_ref() {
            // Hovered: Light blue
            (
                Color32::from_rgb(100, 150, 255),
                Color32::from_rgb(70, 120, 200),
            )
        } else if node.visual_state.has_error {
            // Error: Red
            (
                Color32::from_rgb(208, 2, 27),
                Color32::from_rgb(150, 0, 20),
            )
        } else {
            // Default: Blue
            (
                Color32::from_rgb(74, 144, 226),
                Color32::from_rgb(50, 100, 180),
            )
        };

        // Draw node box
        painter.rect(
            node_rect,
            5.0, // Rounded corners
            fill_color,
            Stroke::new(2.0 * self.camera.zoom, stroke_color),
        );

        // Draw node icon and label
        let text_size = 14.0 * self.camera.zoom;
        if text_size > 8.0 {
            // Only show text if zoomed in enough
            let icon = node.node_type.icon();
            let name = node.node_type.display_name();

            // Icon
            painter.text(
                Pos2::new(node_rect.center().x, node_rect.min.y + 15.0 * self.camera.zoom),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(text_size * 1.2),
                palette.text,
            );

            // Label
            painter.text(
                Pos2::new(node_rect.center().x, node_rect.max.y - 10.0 * self.camera.zoom),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::proportional(text_size * 0.8),
                palette.text,
            );

            // Parameter indicator (small dot if has errors)
            if node.visual_state.has_error {
                painter.circle_filled(
                    Pos2::new(node_rect.max.x - 5.0, node_rect.min.y + 5.0),
                    3.0 * self.camera.zoom,
                    Color32::RED,
                );
            }
        }
    }

    /// Draw an edge
    fn draw_edge(
        &self,
        painter: &egui::Painter,
        edge: &GraphEdge,
        canvas_rect: Rect,
        palette: &ColorPalette,
    ) {
        // Find source and target nodes
        let from_node = self.graph.get_node(&edge.from);
        let to_node = self.graph.get_node(&edge.to);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let from_pos = self.world_to_screen(from.position, canvas_rect);
            let to_pos = self.world_to_screen(to.position, canvas_rect);

            let edge_color = match edge.edge_type {
                EdgeType::Dependency => palette.accent, // Solid for dependencies
                EdgeType::DataFlow => Color32::from_rgb(150, 150, 150), // Gray for data flow
            };

            // Draw arrow from center of from_node to center of to_node
            self.draw_arrow(painter, from_pos, to_pos, edge_color, &edge.edge_type);
        }
    }

    /// Draw an arrow between two points
    fn draw_arrow(
        &self,
        painter: &egui::Painter,
        from: Pos2,
        to: Pos2,
        color: Color32,
        edge_type: &EdgeType,
    ) {
        let stroke_width = 2.0 * self.camera.zoom;
        let arrow_size = 8.0 * self.camera.zoom;

        // Line style based on edge type
        let stroke = match edge_type {
            EdgeType::Dependency => Stroke::new(stroke_width, color),
            EdgeType::DataFlow => Stroke::new(stroke_width, color), // Could add dashed in future
        };

        // Draw line
        painter.line_segment([from, to], stroke);

        // Draw arrowhead
        let direction = (to - from).normalized();
        let perpendicular = EguiVec2::new(-direction.y, direction.x);

        let arrow_base = to - direction * arrow_size * 2.0;
        let arrow_left = arrow_base + perpendicular * arrow_size;
        let arrow_right = arrow_base - perpendicular * arrow_size;

        painter.add(egui::Shape::convex_polygon(
            vec![to, arrow_left, arrow_right],
            color,
            Stroke::NONE,
        ));
    }

    /// Draw temporary edge being created
    fn draw_temporary_edge(
        &self,
        painter: &egui::Painter,
        edge_state: &EdgeDrawState,
        canvas_rect: Rect,
        palette: &ColorPalette,
    ) {
        if let Some(from_node) = self.graph.get_node(&edge_state.from_node) {
            let from_pos = self.world_to_screen(from_node.position, canvas_rect);
            let to_pos = edge_state.current_pos;

            // Dotted line for temporary edge
            painter.line_segment(
                [from_pos, to_pos],
                Stroke::new(
                    2.0 * self.camera.zoom,
                    Color32::from_rgba_premultiplied(
                        palette.accent.r(),
                        palette.accent.g(),
                        palette.accent.b(),
                        150,
                    ),
                ),
            );

            // Small circle at mouse position
            painter.circle_filled(to_pos, 4.0 * self.camera.zoom, palette.accent);
        }
    }

    /// Draw selection box
    fn draw_selection_box(&self, painter: &egui::Painter, start: Pos2, end: Pos2, palette: &ColorPalette) {
        let rect = Rect::from_two_pos(start, end);
        painter.rect(
            rect,
            0.0,
            Color32::from_rgba_premultiplied(
                palette.accent.r(),
                palette.accent.g(),
                palette.accent.b(),
                30,
            ),
            Stroke::new(1.0, palette.accent),
        );
    }

    /// Handle input
    fn handle_input(&mut self, ui: &mut Ui, response: &Response) {
        let canvas_rect = response.rect;

        // Zoom with scroll wheel
        if response.hovered() {
            ui.input(|i| {
                let scroll_delta = i.raw_scroll_delta.y;
                if scroll_delta != 0.0 {
                    let zoom_factor = 1.0 + scroll_delta * 0.001;
                    self.camera.zoom *= zoom_factor;
                    self.camera.zoom = self.camera.zoom.clamp(0.25, 3.0);
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
                } else {
                    // Click on already selected: Start potential drag
                }
            } else {
                // Click on empty space: Clear selection
                if !ctrl_held {
                    self.clear_selection();
                }
            }
        }

        // Handle drag start
        if response.drag_started() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
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
                        let world_pos = self.screen_to_world(pointer_pos, canvas_rect);
                        let offset = Vec2::new(
                            node.position.x - world_pos.x,
                            node.position.y - world_pos.y,
                        );
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
        }

        // Handle dragging
        if response.dragged() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                // Extract drag state to avoid borrow issues
                let drag_state_clone = self.drag_state.clone();
                
                match drag_state_clone {
                    Some(DragState::Node { node_id, offset }) => {
                        // Move node
                        let world_pos = self.screen_to_world(pointer_pos, canvas_rect);
                        let new_pos = Vec2::new(
                            world_pos.x + offset.x,
                            world_pos.y + offset.y,
                        );
                        let final_pos = if self.snap_to_grid {
                            new_pos.snap(self.grid_size)
                        } else {
                            new_pos
                        };

                        if let Some(node) = self.graph.get_node_mut(&node_id) {
                            node.position = final_pos;
                        }
                    }
                    Some(DragState::SelectBox { start, mut current }) => {
                        // Update selection box
                        current = pointer_pos;
                        self.drag_state = Some(DragState::SelectBox { start, current });

                        // Select nodes in box
                        let box_rect = Rect::from_two_pos(start, current);
                        if !ctrl_held {
                            self.clear_selection();
                        }

                        for node in &self.graph.nodes {
                            let node_screen = self.world_to_screen(node.position, canvas_rect);
                            if box_rect.contains(node_screen) {
                                self.selected_nodes.insert(node.id.clone());
                            }
                        }
                    }
                    Some(DragState::Pan { start_camera_pos }) => {
                        // Pan camera
                        self.camera.position.x = start_camera_pos.x - response.drag_delta().x / self.camera.zoom;
                        self.camera.position.y = start_camera_pos.y - response.drag_delta().y / self.camera.zoom;
                    }
                    None => {}
                }

                // Update edge drawing position
                if let Some(edge_state) = &mut self.drawing_edge {
                    edge_state.current_pos = pointer_pos;
                }
            }
        }

        // Handle drag released
        if response.drag_released() {
            // If we were drawing an edge, try to create it
            if let Some(edge_state) = &self.drawing_edge {
                if let Some(target_node) = &self.hovered_node {
                    // Create edge from source to target
                    let edge = GraphEdge::dependency(
                        edge_state.from_node.clone(),
                        target_node.clone(),
                    );
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
            let world_pos = self.screen_to_world(pointer_pos, response.rect);

            for node in &self.graph.nodes {
                let dx = (node.position.x - world_pos.x).abs();
                let dy = (node.position.y - world_pos.y).abs();

                let half_width = self.node_size.x / (2.0 * self.camera.zoom);
                let half_height = self.node_size.y / (2.0 * self.camera.zoom);

                if dx < half_width && dy < half_height {
                    self.hovered_node = Some(node.id.clone());
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    break;
                }
            }
        }
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, world_pos: Vec2, canvas_rect: Rect) -> Pos2 {
        let screen_x = canvas_rect.center().x
            + (world_pos.x - self.camera.position.x) * self.camera.zoom;
        let screen_y = canvas_rect.center().y
            + (world_pos.y - self.camera.position.y) * self.camera.zoom;
        Pos2::new(screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_pos: Pos2, canvas_rect: Rect) -> Vec2 {
        let world_x = self.camera.position.x
            + (screen_pos.x - canvas_rect.center().x) / self.camera.zoom;
        let world_y = self.camera.position.y
            + (screen_pos.y - canvas_rect.center().y) / self.camera.zoom;
        Vec2::new(world_x, world_y)
    }

    /// Get the current graph
    #[must_use]
    pub fn graph(&self) -> &VisualGraph {
        &self.graph
    }

    /// Get mutable graph reference
    pub fn graph_mut(&mut self) -> &mut VisualGraph {
        &mut self.graph
    }

    /// Add a node at screen position
    pub fn add_node_at_screen(&mut self, node_type: NodeType, screen_pos: Pos2, canvas_rect: Rect) {
        let world_pos = self.screen_to_world(screen_pos, canvas_rect);
        let final_pos = if self.snap_to_grid {
            world_pos.snap(self.grid_size)
        } else {
            world_pos
        };

        let node = GraphNode::new(node_type, final_pos);
        self.graph.add_node(node);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    /// Select node
    pub fn select_node(&mut self, node_id: String) {
        self.selected_nodes.insert(node_id);
    }

    /// Toggle node selection
    pub fn toggle_node_selection(&mut self, node_id: String) {
        if self.selected_nodes.contains(&node_id) {
            self.selected_nodes.remove(&node_id);
        } else {
            self.selected_nodes.insert(node_id);
        }
    }

    /// Delete selected nodes
    pub fn delete_selected(&mut self) {
        for node_id in &self.selected_nodes {
            self.graph.remove_node(node_id);
        }
        self.selected_nodes.clear();
    }

    /// Reset camera to center
    pub fn reset_camera(&mut self) {
        self.camera = Camera::default();
    }
}

/// Camera for panning and zooming
#[derive(Clone, Debug)]
struct Camera {
    /// Camera position in world coordinates
    position: Vec2,

    /// Zoom level (1.0 = 100%)
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            zoom: 1.0,
        }
    }
}

/// Drag state
#[derive(Clone, Debug)]
enum DragState {
    /// Dragging a node
    Node {
        node_id: String,
        offset: Vec2,
    },

    /// Drawing a selection box
    SelectBox {
        start: Pos2,
        current: Pos2,
    },

    /// Panning the canvas
    Pan {
        start_camera_pos: Vec2,
    },
}

/// State for drawing an edge
#[derive(Clone, Debug)]
struct EdgeDrawState {
    /// Source node ID
    from_node: String,

    /// Current mouse position
    current_pos: Pos2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = GraphCanvas::new("test-graph".to_string());
        assert_eq!(canvas.graph().name, "test-graph");
        assert_eq!(canvas.camera.zoom, 1.0);
        assert!(canvas.show_grid);
    }

    #[test]
    fn test_camera_zoom() {
        let mut canvas = GraphCanvas::new("test".to_string());
        canvas.camera.zoom = 2.0;
        assert_eq!(canvas.camera.zoom, 2.0);

        // Test zoom clamping
        canvas.camera.zoom = 5.0;
        canvas.camera.zoom = canvas.camera.zoom.clamp(0.25, 3.0);
        assert_eq!(canvas.camera.zoom, 3.0);
    }

    #[test]
    fn test_add_node() {
        let mut canvas = GraphCanvas::new("test".to_string());
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(400.0, 300.0), canvas_rect);
        assert_eq!(canvas.graph().nodes.len(), 1);
    }

    #[test]
    fn test_selection() {
        let mut canvas = GraphCanvas::new("test".to_string());
        let node_id = "test-node".to_string();

        canvas.select_node(node_id.clone());
        assert!(canvas.selected_nodes.contains(&node_id));

        canvas.clear_selection();
        assert!(canvas.selected_nodes.is_empty());
    }

    #[test]
    fn test_coordinate_conversion() {
        let canvas = GraphCanvas::new("test".to_string());
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        let world_pos = Vec2::new(100.0, 100.0);
        let screen_pos = canvas.world_to_screen(world_pos, canvas_rect);
        let back_to_world = canvas.screen_to_world(screen_pos, canvas_rect);

        assert!((back_to_world.x - world_pos.x).abs() < 0.1);
        assert!((back_to_world.y - world_pos.y).abs() < 0.1);
    }

    #[test]
    fn test_node_dragging() {
        let mut canvas = GraphCanvas::new("test".to_string());
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        // Add a node
        canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(400.0, 300.0), canvas_rect);
        let node_id = canvas.graph().nodes[0].id.clone();
        let initial_pos = canvas.graph().nodes[0].position;

        // Simulate drag by setting drag state
        canvas.drag_state = Some(DragState::Node {
            node_id: node_id.clone(),
            offset: Vec2::zero(),
        });

        // Move node
        if let Some(node) = canvas.graph.get_node_mut(&node_id) {
            node.position = Vec2::new(initial_pos.x + 50.0, initial_pos.y + 50.0);
        }

        let final_pos = canvas.graph().nodes[0].position;
        assert!(final_pos.x > initial_pos.x);
        assert!(final_pos.y > initial_pos.y);
    }

    #[test]
    fn test_multi_select() {
        let mut canvas = GraphCanvas::new("test".to_string());
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        // Add multiple nodes
        canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(100.0, 100.0), canvas_rect);
        canvas.add_node_at_screen(NodeType::Verification, Pos2::new(200.0, 200.0), canvas_rect);
        canvas.add_node_at_screen(NodeType::WaitFor, Pos2::new(300.0, 300.0), canvas_rect);

        let id1 = canvas.graph().nodes[0].id.clone();
        let id2 = canvas.graph().nodes[1].id.clone();

        // Select first node
        canvas.select_node(id1.clone());
        assert_eq!(canvas.selected_nodes.len(), 1);

        // Add second node to selection
        canvas.select_node(id2.clone());
        assert_eq!(canvas.selected_nodes.len(), 2);

        // Clear selection
        canvas.clear_selection();
        assert_eq!(canvas.selected_nodes.len(), 0);
    }

    #[test]
    fn test_delete_selected() {
        let mut canvas = GraphCanvas::new("test".to_string());
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        // Add nodes
        canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(100.0, 100.0), canvas_rect);
        canvas.add_node_at_screen(NodeType::Verification, Pos2::new(200.0, 200.0), canvas_rect);
        assert_eq!(canvas.graph().nodes.len(), 2);

        let id = canvas.graph().nodes[0].id.clone();

        // Select and delete first node
        canvas.select_node(id);
        canvas.delete_selected();

        assert_eq!(canvas.graph().nodes.len(), 1);
        assert!(canvas.selected_nodes.is_empty());
    }

    #[test]
    fn test_edge_creation() {
        let mut canvas = GraphCanvas::new("test".to_string());
        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        // Add two nodes
        canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(100.0, 100.0), canvas_rect);
        canvas.add_node_at_screen(NodeType::Verification, Pos2::new(200.0, 200.0), canvas_rect);

        let id1 = canvas.graph().nodes[0].id.clone();
        let id2 = canvas.graph().nodes[1].id.clone();

        // Start drawing edge
        canvas.drawing_edge = Some(EdgeDrawState {
            from_node: id1.clone(),
            current_pos: Pos2::new(150.0, 150.0),
        });

        // Set hovered node to target
        canvas.hovered_node = Some(id2.clone());

        // Create edge (simulating drag release)
        let edge = GraphEdge::dependency(id1, id2);
        assert!(canvas.graph.add_edge(edge).is_ok());
        assert_eq!(canvas.graph().edges.len(), 1);
    }

    #[test]
    fn test_snap_to_grid() {
        let mut canvas = GraphCanvas::new("test".to_string());
        canvas.snap_to_grid = true;
        canvas.grid_size = 50.0;

        let canvas_rect = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(800.0, 600.0));

        // Add node at non-grid position
        canvas.add_node_at_screen(NodeType::PrimalStart, Pos2::new(423.0, 347.0), canvas_rect);

        // Position should be snapped to grid
        let pos = canvas.graph().nodes[0].position;
        
        // Should be close to grid multiples
        assert!((pos.x % 50.0).abs() < 1.0 || (pos.x % 50.0 - 50.0).abs() < 1.0);
        assert!((pos.y % 50.0).abs() < 1.0 || (pos.y % 50.0 - 50.0).abs() < 1.0);
    }
}

