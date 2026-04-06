// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph Canvas Widget
//!
//! Interactive egui-based canvas for graph editing.

use egui::{Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::collections::HashSet;

use super::edge::GraphEdge;
use super::graph::Graph;
use super::node::GraphNode;

// --- Pure logic (testable, no egui) ---

/// World to screen coordinate conversion.
#[must_use]
pub fn world_to_screen(world_pos: Pos2, rect: Rect, pan: egui::Vec2, zoom: f32) -> Pos2 {
    let screen_x = world_pos.x.mul_add(zoom, rect.left()) + pan.x;
    let screen_y = world_pos.y.mul_add(zoom, rect.top()) + pan.y;
    Pos2::new(screen_x, screen_y)
}

/// Screen to world coordinate conversion.
#[must_use]
pub fn screen_to_world(screen_pos: Pos2, rect: Rect, pan: egui::Vec2, zoom: f32) -> Pos2 {
    let world_x = (screen_pos.x - rect.left() - pan.x) / zoom;
    let world_y = (screen_pos.y - rect.top() - pan.y) / zoom;
    Pos2::new(world_x, world_y)
}

#[must_use]
pub fn editor_grid_params(zoom: f32, base_size: f32) -> (f32, f32) {
    let grid_size = base_size * zoom;
    (grid_size, grid_size)
}

#[must_use]
pub fn editor_edge_stroke_width(zoom: f32) -> f32 {
    2.0 * zoom
}

#[must_use]
pub const fn editor_selected_stroke_width(selected: bool) -> f32 {
    if selected { 3.0 } else { 1.0 }
}

#[must_use]
pub fn editor_arrow_vertices(
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    zoom: f32,
) -> ((f32, f32), (f32, f32), (f32, f32)) {
    let arrow_size = 10.0 * zoom;
    let arrow_angle = std::f32::consts::PI / 6.0;
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let len = dx.hypot(dy);
    if len < f32::EPSILON {
        return ((to_x, to_y), (to_x, to_y), (to_x, to_y));
    }
    let dir_x = dx / len;
    let dir_y = dy / len;
    let perp_x = -dir_y;
    let perp_y = dir_x;
    let half = arrow_size * arrow_angle.sin();
    let tip = (to_x, to_y);
    let left = (perp_x.mul_add(-half, to_x), perp_y.mul_add(-half, to_y));
    let right = (perp_x.mul_add(half, to_x), perp_y.mul_add(half, to_y));
    (tip, left, right)
}

#[must_use]
pub fn format_node_label(icon: &str, name: &str) -> String {
    format!("{icon} {name}")
}

/// Editor node fill and stroke colors from base color and state.
/// Returns (`fill_rgb`, `stroke_rgb`). Uses multiply factors: selected=0.8, else 0.6.
#[must_use]
pub(crate) fn editor_node_colors(
    base_rgb: [u8; 3],
    selected: bool,
    _hovered: bool,
) -> ([u8; 3], [u8; 3]) {
    let factor = if selected { 0.8 } else { 0.6 };
    let fill_rgb = [
        (f32::from(base_rgb[0]) * factor).round() as u8,
        (f32::from(base_rgb[1]) * factor).round() as u8,
        (f32::from(base_rgb[2]) * factor).round() as u8,
    ];
    let stroke_rgb = if selected { [255, 255, 255] } else { base_rgb };
    (fill_rgb, stroke_rgb)
}

// --- Rendering (uses egui) ---

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
    pub const fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get mutable reference to current graph
    pub const fn graph_mut(&mut self) -> &mut Graph {
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
                self.zoom *= scroll_delta.mul_add(0.001, 1.0);
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }
        }

        // Draw grid
        self.draw_grid(&painter, rect);

        // Draw edges
        for edge in &self.graph.edges {
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
        let (grid_size, _) = editor_grid_params(self.zoom, 50.0);
        let color = Color32::from_gray(40);

        // Vertical lines
        let start_x = (rect.left() - self.pan.x) % grid_size;
        let count_x = ((rect.right() - start_x) / grid_size).ceil().max(0.0) as usize;
        for i in 0..count_x {
            let x = (i as f32).mul_add(grid_size, start_x);
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, color),
            );
        }

        // Horizontal lines
        let start_y = (rect.top() - self.pan.y) % grid_size;
        let count_y = ((rect.bottom() - start_y) / grid_size).ceil().max(0.0) as usize;
        for i in 0..count_y {
            let y = (i as f32).mul_add(grid_size, start_y);
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, color),
            );
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
        let screen_pos = world_to_screen(
            Pos2::new(node.position.0, node.position.1),
            rect,
            self.pan,
            self.zoom,
        );

        // Node size
        let node_size = Vec2::new(120.0, 60.0) * self.zoom;
        let node_rect = Rect::from_center_size(screen_pos, node_size);

        // Check if node is selected
        let is_selected = self.selected_nodes.contains(&node.id);
        let is_hovered = node_rect.contains(response.hover_pos().unwrap_or(Pos2::ZERO));

        // Node color based on state (pure logic)
        let base_rgb = node.display_color();
        let (fill_rgb, stroke_rgb) = editor_node_colors(base_rgb, is_selected, is_hovered);
        let fill_color = Color32::from_rgb(fill_rgb[0], fill_rgb[1], fill_rgb[2]);
        let stroke_color = Color32::from_rgb(stroke_rgb[0], stroke_rgb[1], stroke_rgb[2]);

        // Draw node rectangle
        painter.rect_filled(node_rect, 5.0, fill_color);

        painter.rect_stroke(
            node_rect,
            5.0,
            Stroke::new(editor_selected_stroke_width(is_selected), stroke_color),
        );

        // Draw node label
        let font_id = FontId::proportional(12.0 * self.zoom);
        painter.text(
            screen_pos,
            egui::Align2::CENTER_CENTER,
            format_node_label(node.display_icon(), &node.name),
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
        if let Some(dragging_id) = &self.dragging_node
            && dragging_id == &node.id
            && response.dragged()
        {
            // Update node position
            if let Some(node_mut) = self.graph.nodes.get_mut(&node.id) {
                let delta = response.drag_delta() / self.zoom;
                node_mut.position.0 += delta.x;
                node_mut.position.1 += delta.y;
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
            let from_pos = world_to_screen(
                Pos2::new(from_node.position.0, from_node.position.1),
                rect,
                self.pan,
                self.zoom,
            );
            let to_pos = world_to_screen(
                Pos2::new(to_node.position.0, to_node.position.1),
                rect,
                self.pan,
                self.zoom,
            );

            // Edge color
            let rgb = edge.display_color();
            let edge_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);

            painter.line_segment(
                [from_pos, to_pos],
                Stroke::new(editor_edge_stroke_width(self.zoom), edge_color),
            );

            let (_tip, left, right) =
                editor_arrow_vertices(from_pos.x, from_pos.y, to_pos.x, to_pos.y, self.zoom);
            let stroke_w = editor_edge_stroke_width(self.zoom);
            painter.line_segment(
                [Pos2::new(to_pos.x, to_pos.y), Pos2::new(left.0, left.1)],
                Stroke::new(stroke_w, edge_color),
            );
            painter.line_segment(
                [Pos2::new(to_pos.x, to_pos.y), Pos2::new(right.0, right.1)],
                Stroke::new(stroke_w, edge_color),
            );
        }
    }

    /// Add a node at screen position
    pub fn add_node_at_screen_pos(&mut self, screen_pos: Pos2, rect: Rect, node_type: String) {
        let world_pos = screen_to_world(screen_pos, rect, self.pan, self.zoom);

        let node_id = format!("node-{}", uuid::Uuid::new_v4());
        let mut node = GraphNode::new(node_id, node_type);
        node.position = (world_pos.x, world_pos.y);

        let _ = self.graph.add_node(node);
    }

    /// Get selected nodes
    #[must_use]
    pub const fn selected_nodes(&self) -> &HashSet<String> {
        &self.selected_nodes
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    /// Reset view
    pub const fn reset_view(&mut self) {
        self.pan = Vec2::ZERO;
        self.zoom = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::{Graph, GraphEdge, GraphNode};
    use super::{
        GraphCanvas, editor_arrow_vertices, editor_edge_stroke_width, editor_grid_params,
        editor_node_colors, editor_selected_stroke_width, format_node_label, screen_to_world,
        world_to_screen,
    };
    use crate::graph_editor::edge::DependencyType;
    use egui::{Pos2, Rect, Vec2};

    #[test]
    fn test_world_to_screen_origin() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let pan = Vec2::ZERO;
        let zoom = 1.0;
        let screen = world_to_screen(Pos2::new(0.0, 0.0), rect, pan, zoom);
        assert!((screen.x - 0.0).abs() < f32::EPSILON);
        assert!((screen.y - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_world_to_screen_with_zoom() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let pan = Vec2::ZERO;
        let screen = world_to_screen(Pos2::new(100.0, 50.0), rect, pan, 2.0);
        assert!((screen.x - 200.0).abs() < f32::EPSILON);
        assert!((screen.y - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_world_to_screen_with_pan() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let pan = Vec2::new(50.0, 25.0);
        let screen = world_to_screen(Pos2::new(0.0, 0.0), rect, pan, 1.0);
        assert!((screen.x - 50.0).abs() < f32::EPSILON);
        assert!((screen.y - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_screen_to_world_roundtrip() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let pan = Vec2::new(10.0, 20.0);
        let zoom = 1.5;
        let world = Pos2::new(100.0, 200.0);
        let screen = world_to_screen(world, rect, pan, zoom);
        let back = screen_to_world(screen, rect, pan, zoom);
        assert!((back.x - world.x).abs() < 0.01);
        assert!((back.y - world.y).abs() < 0.01);
    }

    #[test]
    fn test_editor_node_colors_selected() {
        let base = [100, 150, 200];
        let (fill, stroke) = editor_node_colors(base, true, false);
        assert_eq!(fill, [80, 120, 160]); // 0.8 * each
        assert_eq!(stroke, [255, 255, 255]);
    }

    #[test]
    fn test_editor_node_colors_not_selected() {
        let base = [100, 150, 200];
        let (fill, stroke) = editor_node_colors(base, false, false);
        assert_eq!(fill, [60, 90, 120]); // 0.6 * each
        assert_eq!(stroke, base);
    }

    #[test]
    fn test_editor_node_colors_hovered_unused() {
        let base = [128, 128, 128];
        let (fill, _) = editor_node_colors(base, false, true);
        assert_eq!(fill, [77, 77, 77]); // 0.6 * 128 rounded
    }

    #[test]
    fn test_editor_grid_params() {
        let (gs, _) = editor_grid_params(1.0, 50.0);
        assert!((gs - 50.0).abs() < f32::EPSILON);
        let (gs, _) = editor_grid_params(2.0, 50.0);
        assert!((gs - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_editor_edge_stroke_width() {
        assert!((editor_edge_stroke_width(1.0) - 2.0).abs() < f32::EPSILON);
        assert!((editor_edge_stroke_width(2.0) - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_editor_selected_stroke_width() {
        assert_eq!(editor_selected_stroke_width(true), 3.0);
        assert_eq!(editor_selected_stroke_width(false), 1.0);
    }

    #[test]
    fn test_editor_arrow_vertices() {
        let (tip, left, right) = editor_arrow_vertices(0.0, 0.0, 100.0, 0.0, 1.0);
        assert!((tip.0 - 100.0).abs() < 0.01);
        assert!((tip.1 - 0.0).abs() < 0.01);
        assert!(left.1 != right.1);
    }

    #[test]
    fn test_format_node_label() {
        assert_eq!(format_node_label("●", "node1"), "● node1");
    }

    #[test]
    fn test_editor_arrow_vertices_zero_length() {
        let (tip, left, right) = editor_arrow_vertices(50.0, 50.0, 50.0, 50.0, 1.0);
        assert!((tip.0 - 50.0).abs() < 0.01);
        assert!((tip.1 - 50.0).abs() < 0.01);
        assert_eq!(left, right);
    }

    #[test]
    fn test_graph_canvas_new() {
        let graph = Graph::new("g1".to_string(), "Test Graph".to_string());
        let canvas = GraphCanvas::new(graph);
        assert_eq!(canvas.graph().id, "g1");
        assert!(canvas.selected_nodes().is_empty());
    }

    #[test]
    fn test_graph_canvas_show() {
        let graph = Graph::new("g1".to_string(), "Test".to_string());
        let mut canvas = GraphCanvas::new(graph);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = canvas.show(ui);
            });
        });
    }

    #[test]
    fn test_graph_canvas_show_with_nodes() {
        let mut graph = Graph::new("g1".to_string(), "Test".to_string());
        let node1 =
            GraphNode::new("n1".to_string(), "start".to_string()).with_position(100.0, 100.0);
        let node2 =
            GraphNode::new("n2".to_string(), "task".to_string()).with_position(200.0, 200.0);
        graph.add_node(node1).expect("add node1");
        graph.add_node(node2).expect("add node2");
        let edge = GraphEdge::new(
            "e1".to_string(),
            "n1".to_string(),
            "n2".to_string(),
            DependencyType::Sequential,
        );
        graph.edges.push(edge);

        let mut canvas = GraphCanvas::new(graph);
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = canvas.show(ui);
            });
        });
    }

    #[test]
    fn test_graph_canvas_add_node_at_screen_pos() {
        let graph = Graph::new("g1".to_string(), "Test".to_string());
        let mut canvas = GraphCanvas::new(graph);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

        canvas.add_node_at_screen_pos(Pos2::new(400.0, 300.0), rect, "start".to_string());
        assert_eq!(canvas.graph().nodes.len(), 1);
    }

    #[test]
    fn test_graph_canvas_clear_selection() {
        let graph = Graph::new("g1".to_string(), "Test".to_string());
        let mut canvas = GraphCanvas::new(graph);
        canvas.clear_selection();
        assert!(canvas.selected_nodes().is_empty());
    }

    #[test]
    fn test_graph_canvas_reset_view() {
        let graph = Graph::new("g1".to_string(), "Test".to_string());
        let mut canvas = GraphCanvas::new(graph);
        canvas.pan = Vec2::new(100.0, 50.0);
        canvas.zoom = 2.0;
        canvas.reset_view();
        assert!((canvas.pan.x - 0.0).abs() < f32::EPSILON);
        assert!((canvas.pan.y - 0.0).abs() < f32::EPSILON);
        assert!((canvas.zoom - 1.0).abs() < f32::EPSILON);
    }
}
