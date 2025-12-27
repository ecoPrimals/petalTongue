//! 2D Visual Renderer
//!
//! Renders graph topology as 2D graphics using egui.
//! Supports animation of flow particles and node pulses.

use egui::{Color32, Pos2, Stroke, Vec2};
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::graph_engine::Node;
use petal_tongue_core::graph_engine::Position;
use petal_tongue_core::{GraphEngine, PrimalHealthStatus};
use std::sync::{Arc, RwLock};

/// 2D Visual Renderer for graphs
pub struct Visual2DRenderer {
    /// Shared graph engine
    graph: Arc<RwLock<GraphEngine>>,
    /// Camera position (for panning)
    camera_offset: Vec2,
    /// Zoom level (1.0 = normal, 2.0 = 2x zoom)
    zoom: f32,
    /// Selected node ID
    selected_node: Option<String>,
    /// Is user currently dragging
    is_dragging: bool,
    /// Last mouse position (for drag delta)
    _last_mouse_pos: Option<Pos2>,
    /// Animation engine (optional, for flow visualization)
    animation_engine: Option<Arc<RwLock<AnimationEngine>>>,
    /// Animation enabled flag
    animation_enabled: bool,
}

impl Visual2DRenderer {
    /// Create a new visual 2D renderer
    pub fn new(graph: Arc<RwLock<GraphEngine>>) -> Self {
        Self {
            graph,
            camera_offset: Vec2::ZERO,
            zoom: 1.0,
            selected_node: None,
            is_dragging: false,
            _last_mouse_pos: None,
            animation_engine: None,
            animation_enabled: false,
        }
    }

    /// Set the animation engine
    ///
    /// Enables flow particles and node pulse visualization.
    pub fn set_animation_engine(&mut self, engine: Arc<RwLock<AnimationEngine>>) {
        self.animation_engine = Some(engine);
    }

    /// Enable or disable animation
    pub fn set_animation_enabled(&mut self, enabled: bool) {
        self.animation_enabled = enabled;
    }

    /// Check if animation is enabled
    #[must_use]
    pub fn is_animation_enabled(&self) -> bool {
        self.animation_enabled
    }

    /// Render the graph to egui
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Get available space
        let available_size = ui.available_size();
        let (response, mut painter) =
            ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        // Handle input (before we borrow graph)
        self.handle_input(&response);

        // Clip to available area
        let clip_rect = response.rect;
        painter.set_clip_rect(clip_rect);

        // Now borrow graph for reading
        let graph = self.graph.read().expect("graph lock poisoned");

        // Calculate center of screen for world-to-screen conversion
        let screen_center = clip_rect.center();

        // Render edges first (so they appear behind nodes)
        for edge in graph.edges() {
            if let (Some(from_node), Some(to_node)) =
                (graph.get_node(&edge.from), graph.get_node(&edge.to))
            {
                let from_pos = self.world_to_screen(from_node.position, screen_center);
                let to_pos = self.world_to_screen(to_node.position, screen_center);

                // Draw edge line
                painter.line_segment(
                    [from_pos, to_pos],
                    Stroke::new(2.0 * self.zoom, Color32::GRAY),
                );

                // Draw arrow head
                self.draw_arrow_head(&painter, from_pos, to_pos);
            }
        }

        // Render nodes
        for node in graph.nodes() {
            let screen_pos = self.world_to_screen(node.position, screen_center);

            // Skip if outside visible area (optimization)
            if !clip_rect.expand(100.0).contains(screen_pos) {
                continue;
            }

            // Determine if this node is selected
            let is_selected = self.selected_node.as_ref() == Some(&node.info.id);

            // Draw node
            self.draw_node(&painter, node, screen_pos, is_selected);
        }

        // Render animation (flow particles and pulses) if enabled
        if self.animation_enabled {
            if let Some(animation_engine) = &self.animation_engine {
                if let Ok(engine) = animation_engine.read() {
                    self.render_animation(&painter, &engine, &graph, screen_center);
                }
            }
        }

        // Draw stats in corner
        self.draw_stats(ui, &graph);
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, world_pos: Position, screen_center: Pos2) -> Pos2 {
        Pos2::new(
            screen_center.x + (world_pos.x * self.zoom) + self.camera_offset.x,
            screen_center.y + (world_pos.y * self.zoom) + self.camera_offset.y,
        )
    }

    /// Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_pos: Pos2, screen_center: Pos2) -> Position {
        Position::new_2d(
            (screen_pos.x - screen_center.x - self.camera_offset.x) / self.zoom,
            (screen_pos.y - screen_center.y - self.camera_offset.y) / self.zoom,
        )
    }

    /// Draw a single node
    fn draw_node(&self, painter: &egui::Painter, node: &Node, screen_pos: Pos2, is_selected: bool) {
        let radius = 20.0 * self.zoom;

        // Get color based on health status
        let (fill_color, stroke_color) = Self::health_to_colors(node.info.health);

        // Draw selection highlight
        if is_selected {
            painter.circle(
                screen_pos,
                radius + 5.0,
                Color32::TRANSPARENT,
                Stroke::new(3.0, Color32::YELLOW),
            );
        }

        // Draw node circle
        painter.circle(
            screen_pos,
            radius,
            fill_color,
            Stroke::new(2.0, stroke_color),
        );

        // Draw node label (if zoomed in enough)
        if self.zoom > 0.5 {
            let text = &node.info.name;
            painter.text(
                Pos2::new(screen_pos.x, screen_pos.y + radius + 10.0),
                egui::Align2::CENTER_TOP,
                text,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        }
    }

    /// Draw arrow head on edge
    fn draw_arrow_head(&self, painter: &egui::Painter, from: Pos2, to: Pos2) {
        let arrow_size = 10.0 * self.zoom;
        let direction = (to - from).normalized();
        let perpendicular = Vec2::new(-direction.y, direction.x);

        // Arrow point (slightly back from 'to' position)
        let arrow_tip = to - direction * 20.0 * self.zoom;
        let arrow_left = arrow_tip - direction * arrow_size + perpendicular * arrow_size * 0.5;
        let arrow_right = arrow_tip - direction * arrow_size - perpendicular * arrow_size * 0.5;

        // Draw filled triangle
        painter.add(egui::Shape::convex_polygon(
            vec![arrow_tip, arrow_left, arrow_right],
            Color32::GRAY,
            Stroke::NONE,
        ));
    }

    /// Map health status to colors
    fn health_to_colors(health: PrimalHealthStatus) -> (Color32, Color32) {
        match health {
            PrimalHealthStatus::Healthy => (
                Color32::from_rgb(40, 180, 40), // Green fill
                Color32::from_rgb(20, 120, 20), // Dark green stroke
            ),
            PrimalHealthStatus::Warning => (
                Color32::from_rgb(200, 180, 40), // Yellow fill
                Color32::from_rgb(140, 120, 20), // Dark yellow stroke
            ),
            PrimalHealthStatus::Critical => (
                Color32::from_rgb(200, 40, 40), // Red fill
                Color32::from_rgb(140, 20, 20), // Dark red stroke
            ),
            PrimalHealthStatus::Unknown => (
                Color32::from_rgb(120, 120, 120), // Gray fill
                Color32::from_rgb(80, 80, 80),    // Dark gray stroke
            ),
        }
    }

    /// Handle user input (pan, zoom, click)
    fn handle_input(&mut self, response: &egui::Response) {
        // Handle zoom (scroll wheel)
        if response.hovered() {
            let scroll_delta = response.ctx.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = 1.0 + (scroll_delta * 0.001);
                self.zoom = (self.zoom * zoom_factor).clamp(0.1, 10.0);
            }
        }

        // Handle pan (drag)
        if response.dragged() {
            self.camera_offset += response.drag_delta();
            self.is_dragging = true;
        } else {
            self.is_dragging = false;
        }

        // Handle node selection (click)
        if response.clicked() && !self.is_dragging {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let screen_center = response.rect.center();
                let world_pos = self.screen_to_world(mouse_pos, screen_center);

                // Find node under cursor
                let graph = self.graph.read().expect("graph lock poisoned");
                let clicked_node = graph.nodes().iter().find(|node| {
                    let distance = node.position.distance_to(world_pos);
                    distance < 20.0 // Node radius in world coordinates
                });

                if let Some(node) = clicked_node {
                    self.selected_node = Some(node.info.id.clone());
                } else {
                    self.selected_node = None;
                }
            }
        }
    }

    /// Render animation (flow particles and node pulses)
    fn render_animation(
        &self,
        painter: &egui::Painter,
        animation_engine: &AnimationEngine,
        graph: &GraphEngine,
        screen_center: Pos2,
    ) {
        // Render flow particles from edge animations
        for edge_anim in &animation_engine.edge_animations {
            for particle in &edge_anim.particles {
                // Get source and target node positions
                if let (Some(source_node), Some(target_node)) = (
                    graph.get_node(&edge_anim.source),
                    graph.get_node(&edge_anim.target),
                ) {
                    // Interpolate position along edge based on progress
                    let source_pos = source_node.position;
                    let target_pos = target_node.position;

                    let x = source_pos.x + (target_pos.x - source_pos.x) * particle.progress;
                    let y = source_pos.y + (target_pos.y - source_pos.y) * particle.progress;

                    let world_pos = Position::new_2d(x, y);
                    let screen_pos = self.world_to_screen(world_pos, screen_center);

                    // Draw particle as a small circle
                    painter.circle_filled(
                        screen_pos,
                        4.0 * self.zoom,
                        Color32::from_rgb(100, 200, 255),
                    );
                }
            }
        }

        // Render node pulses
        for pulse in &animation_engine.node_pulses {
            if let Some(node) = graph.get_node(&pulse.node_id) {
                let screen_pos = self.world_to_screen(node.position, screen_center);

                // Pulse effect: expanding circle that fades out
                let pulse_radius = 25.0 * self.zoom * pulse.radius_multiplier();
                let alpha = (255.0 * pulse.alpha()) as u8;

                painter.circle_stroke(
                    screen_pos,
                    pulse_radius,
                    Stroke::new(2.0, Color32::from_rgba_premultiplied(100, 200, 255, alpha)),
                );
            }
        }
    }

    /// Draw statistics overlay
    fn draw_stats(&self, ui: &mut egui::Ui, graph: &GraphEngine) {
        let stats = graph.stats();

        egui::Window::new("📊 Graph Statistics")
            .fixed_pos([10.0, 10.0])
            .default_width(220.0)
            .collapsible(true)
            .frame(
                egui::Frame::window(&ui.ctx().style())
                    .fill(egui::Color32::from_rgba_premultiplied(40, 40, 45, 230)),
            )
            .show(ui.ctx(), |ui| {
                ui.label(egui::RichText::new(format!("Nodes: {}", stats.node_count)).strong());
                ui.label(egui::RichText::new(format!("Edges: {}", stats.edge_count)).strong());
                ui.label(
                    egui::RichText::new(format!("Avg Degree: {:.2}", stats.avg_degree)).strong(),
                );
                ui.label(
                    egui::RichText::new(format!("Zoom: {:.2}x", self.zoom))
                        .color(egui::Color32::from_rgb(150, 200, 255)),
                );

                if let Some(selected_id) = &self.selected_node {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Selected:").weak());
                    ui.label(
                        egui::RichText::new(selected_id)
                            .color(egui::Color32::from_rgb(255, 230, 150)),
                    );
                }
            });
    }

    /// Reset camera to default position
    pub fn reset_camera(&mut self) {
        self.camera_offset = Vec2::ZERO;
        self.zoom = 1.0;
    }

    /// Get selected node ID
    pub fn selected_node(&self) -> Option<&str> {
        self.selected_node.as_deref()
    }

    /// Set selected node
    pub fn set_selected_node(&mut self, node_id: Option<String>) {
        self.selected_node = node_id;
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Test assertions need exact equality
#[allow(dead_code)] // Test helper variables
mod tests {
    use super::*;
    use petal_tongue_core::{LayoutAlgorithm, PrimalInfo, TopologyEdge};

    fn create_test_graph() -> Arc<RwLock<GraphEngine>> {
        let mut graph = GraphEngine::new();

        graph.add_node(PrimalInfo {
            id: "node1".to_string(),
            name: "Node 1".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test".to_string()],
            health: PrimalHealthStatus::Healthy,
            last_seen: 0,
        });

        graph.add_node(PrimalInfo {
            id: "node2".to_string(),
            name: "Node 2".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec!["test".to_string()],
            health: PrimalHealthStatus::Warning,
            last_seen: 0,
        });

        graph.add_edge(TopologyEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });

        graph.set_layout(LayoutAlgorithm::Circular);
        graph.layout(1);

        Arc::new(RwLock::new(graph))
    }

    #[test]
    fn test_renderer_creation() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);
        assert_eq!(renderer.zoom, 1.0);
        assert_eq!(renderer.camera_offset, Vec2::ZERO);
    }

    #[test]
    fn test_world_to_screen_conversion() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let world_pos = Position::new_2d(100.0, 50.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let screen_pos = renderer.world_to_screen(world_pos, screen_center);

        assert_eq!(screen_pos.x, 500.0);
        assert_eq!(screen_pos.y, 350.0);
    }

    #[test]
    fn test_screen_to_world_conversion() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let screen_pos = Pos2::new(500.0, 350.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let world_pos = renderer.screen_to_world(screen_pos, screen_center);

        assert_eq!(world_pos.x, 100.0);
        assert_eq!(world_pos.y, 50.0);
    }

    #[test]
    fn test_health_to_colors() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let (fill, _stroke) = Visual2DRenderer::health_to_colors(PrimalHealthStatus::Healthy);
        assert_eq!(fill, Color32::from_rgb(40, 180, 40));

        let (fill, _stroke) = Visual2DRenderer::health_to_colors(PrimalHealthStatus::Critical);
        assert_eq!(fill, Color32::from_rgb(200, 40, 40));
    }

    #[test]
    fn test_node_selection() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        assert!(renderer.selected_node().is_none());

        renderer.set_selected_node(Some("node1".to_string()));
        assert_eq!(renderer.selected_node(), Some("node1"));

        renderer.set_selected_node(None);
        assert!(renderer.selected_node().is_none());
    }

    #[test]
    fn test_camera_reset() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.camera_offset = Vec2::new(100.0, 50.0);
        renderer.zoom = 2.5;

        renderer.reset_camera();

        assert_eq!(renderer.camera_offset, Vec2::ZERO);
        assert_eq!(renderer.zoom, 1.0);
    }
}
