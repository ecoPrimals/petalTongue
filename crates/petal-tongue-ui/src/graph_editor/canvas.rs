//! Graph Canvas Widget
//!
//! Interactive egui-based canvas for graph editing.

use egui::{Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::collections::HashSet;

use super::edge::GraphEdge;
use super::graph::Graph;
use super::node::GraphNode;

/// Graph Canvas - Interactive graph visualization and editing
///
/// Provides drag-and-drop, zoom, pan, and node manipulation.
pub struct GraphCanvas {
    /// Current graph being edited
    graph: Graph,

    /// Canvas pan offset
    pan: Vec2,

    /// Canvas zoom level
    zoom: f32,

    /// Selected nodes
    selected_nodes: HashSet<String>,

    /// Node being dragged (if any)
    dragging_node: Option<String>,

    /// Drag start position
    drag_start_pos: Option<Pos2>,
}

impl GraphCanvas {
    /// Create a new canvas
    #[must_use]
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            pan: Vec2::ZERO,
            zoom: 1.0,
            selected_nodes: HashSet::new(),
            dragging_node: None,
            drag_start_pos: None,
        }
    }

    /// Get reference to current graph
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get mutable reference to current graph
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Render the canvas
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        let rect = response.rect;

        // Handle pan
        if response.dragged() && self.dragging_node.is_none() {
            self.pan += response.drag_delta();
        }

        // Handle zoom (mouse wheel)
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom *= 1.0 + scroll_delta * 0.001;
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }
        }

        // Draw grid
        self.draw_grid(&painter, rect);

        // Draw edges
        for edge in self.graph.edges.iter() {
            self.draw_edge(&painter, rect, edge);
        }

        // Draw nodes (collect to avoid borrow checker issues)
        let nodes: Vec<GraphNode> = self.graph.nodes.values().cloned().collect();
        for node in &nodes {
            self.draw_node(&painter, rect, node, &response);
        }

        response
    }

    /// Draw background grid
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 50.0 * self.zoom;
        let color = Color32::from_gray(40);

        // Vertical lines
        let mut x = (rect.left() - self.pan.x) % grid_size;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, color),
            );
            x += grid_size;
        }

        // Horizontal lines
        let mut y = (rect.top() - self.pan.y) % grid_size;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, color),
            );
            y += grid_size;
        }
    }

    /// Draw a single node
    fn draw_node(
        &mut self,
        painter: &egui::Painter,
        rect: Rect,
        node: &GraphNode,
        response: &Response,
    ) {
        let screen_pos = self.world_to_screen(Pos2::new(node.position.0, node.position.1), rect);

        // Node size
        let node_size = Vec2::new(120.0, 60.0) * self.zoom;
        let node_rect = Rect::from_center_size(screen_pos, node_size);

        // Check if node is selected
        let is_selected = self.selected_nodes.contains(&node.id);

        // Node color based on state
        let rgb = node.display_color();
        let node_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);

        // Draw node rectangle
        painter.rect_filled(
            node_rect,
            5.0,
            if is_selected {
                node_color.linear_multiply(0.8)
            } else {
                node_color.linear_multiply(0.6)
            },
        );

        painter.rect_stroke(
            node_rect,
            5.0,
            Stroke::new(
                if is_selected { 3.0 } else { 1.0 },
                if is_selected {
                    Color32::WHITE
                } else {
                    node_color
                },
            ),
        );

        // Draw node label
        let font_id = FontId::proportional(12.0 * self.zoom);
        painter.text(
            screen_pos,
            egui::Align2::CENTER_CENTER,
            format!("{} {}", node.display_icon(), node.name),
            font_id,
            Color32::WHITE,
        );

        // Handle node interaction
        if node_rect.contains(response.hover_pos().unwrap_or(Pos2::ZERO)) {
            if response.clicked() {
                // Toggle selection
                if is_selected {
                    self.selected_nodes.remove(&node.id);
                } else {
                    self.selected_nodes.insert(node.id.clone());
                }
            }

            if response.drag_started() {
                self.dragging_node = Some(node.id.clone());
                self.drag_start_pos = Some(screen_pos);
            }
        }

        // Handle node dragging
        if let Some(dragging_id) = &self.dragging_node {
            if dragging_id == &node.id && response.dragged() {
                // Update node position
                if let Some(node_mut) = self.graph.nodes.get_mut(&node.id) {
                    let delta = response.drag_delta() / self.zoom;
                    node_mut.position.0 += delta.x;
                    node_mut.position.1 += delta.y;
                }
            }
        }

        if response.drag_stopped() {
            self.dragging_node = None;
            self.drag_start_pos = None;
        }
    }

    /// Draw an edge between nodes
    fn draw_edge(&self, painter: &egui::Painter, rect: Rect, edge: &GraphEdge) {
        if let (Some(from_node), Some(to_node)) = (
            self.graph.nodes.get(&edge.from),
            self.graph.nodes.get(&edge.to),
        ) {
            let from_pos =
                self.world_to_screen(Pos2::new(from_node.position.0, from_node.position.1), rect);
            let to_pos =
                self.world_to_screen(Pos2::new(to_node.position.0, to_node.position.1), rect);

            // Edge color
            let rgb = edge.display_color();
            let edge_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);

            // Draw line
            painter.line_segment([from_pos, to_pos], Stroke::new(2.0 * self.zoom, edge_color));

            // Draw arrow head
            let direction = (to_pos - from_pos).normalized();
            let arrow_size = 10.0 * self.zoom;
            let arrow_angle = std::f32::consts::PI / 6.0; // 30 degrees

            let arrow_left = to_pos - direction.rot90() * arrow_size * arrow_angle.sin();
            let arrow_right = to_pos + direction.rot90() * arrow_size * arrow_angle.sin();

            painter.line_segment(
                [to_pos, arrow_left],
                Stroke::new(2.0 * self.zoom, edge_color),
            );
            painter.line_segment(
                [to_pos, arrow_right],
                Stroke::new(2.0 * self.zoom, edge_color),
            );
        }
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, world_pos: Pos2, rect: Rect) -> Pos2 {
        let screen_x = rect.left() + (world_pos.x * self.zoom) + self.pan.x;
        let screen_y = rect.top() + (world_pos.y * self.zoom) + self.pan.y;
        Pos2::new(screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    #[allow(dead_code)]
    fn screen_to_world(&self, screen_pos: Pos2, rect: Rect) -> Pos2 {
        let world_x = (screen_pos.x - rect.left() - self.pan.x) / self.zoom;
        let world_y = (screen_pos.y - rect.top() - self.pan.y) / self.zoom;
        Pos2::new(world_x, world_y)
    }

    /// Add a node at screen position
    pub fn add_node_at_screen_pos(&mut self, screen_pos: Pos2, rect: Rect, node_type: String) {
        let world_pos = self.screen_to_world(screen_pos, rect);

        let node_id = format!("node-{}", uuid::Uuid::new_v4());
        let mut node = GraphNode::new(node_id, node_type);
        node.position = (world_pos.x, world_pos.y);

        let _ = self.graph.add_node(node);
    }

    /// Get selected nodes
    #[must_use]
    pub fn selected_nodes(&self) -> &HashSet<String> {
        &self.selected_nodes
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    /// Reset view
    pub fn reset_view(&mut self) {
        self.pan = Vec2::ZERO;
        self.zoom = 1.0;
    }
}
