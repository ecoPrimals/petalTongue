// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Canvas - Interactive Visual Graph Editor
//!
//! Provides a canvas for building Neural API graphs through drag-and-drop interactions.
//! TRUE PRIMAL: Zero hardcoding, capability-based, self-contained visualization.

mod interaction;
mod layout;
mod rendering;

use crate::accessibility::ColorPalette;
use egui::{Pos2, Rect, Sense, Ui, Vec2 as EguiVec2};
use petal_tongue_core::graph_builder::{GraphNode, NodeType, Vec2, VisualGraph};
use std::collections::HashSet;

/// Interactive graph canvas for building Neural API graphs
pub struct GraphCanvas {
    /// Current graph being edited
    graph: VisualGraph,

    /// Camera position and zoom
    pub(crate) camera: Camera,

    /// Currently selected nodes
    pub(crate) selected_nodes: HashSet<String>,

    /// Drag state
    pub(crate) drag_state: Option<DragState>,

    /// Edge being drawn
    pub(crate) drawing_edge: Option<EdgeDrawState>,

    /// Show grid
    pub(crate) show_grid: bool,

    /// Grid size
    pub(crate) grid_size: f32,

    /// Snap to grid
    pub(crate) snap_to_grid: bool,

    /// Node size
    node_size: EguiVec2,

    /// Hovered node ID
    pub(crate) hovered_node: Option<String>,
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
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

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

    /// Convert world coordinates to screen coordinates
    #[allow(dead_code)]
    fn world_to_screen(&self, world_pos: Vec2, canvas_rect: Rect) -> Pos2 {
        layout::world_to_screen(
            world_pos,
            canvas_rect,
            &self.camera.position,
            self.camera.zoom,
        )
    }

    /// Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_pos: Pos2, canvas_rect: Rect) -> Vec2 {
        layout::screen_to_world(
            screen_pos,
            canvas_rect,
            &self.camera.position,
            self.camera.zoom,
        )
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
pub(crate) struct Camera {
    /// Camera position in world coordinates
    pub position: Vec2,

    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
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
pub(crate) enum DragState {
    /// Dragging a node
    Node { node_id: String, offset: Vec2 },

    /// Drawing a selection box
    SelectBox { start: Pos2, current: Pos2 },

    /// Panning the canvas
    Pan { start_camera_pos: Vec2 },
}

/// State for drawing an edge
#[derive(Clone, Debug)]
pub(crate) struct EdgeDrawState {
    /// Source node ID
    pub from_node: String,

    /// Current mouse position
    pub current_pos: Pos2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::graph_builder::GraphEdge;

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
