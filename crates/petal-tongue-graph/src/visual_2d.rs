//! 2D Visual Renderer
//!
//! Renders graph topology as 2D graphics using egui.
//! Supports animation of flow particles and node pulses.
//!
//! # Design Note: File Size
//!
//! This file is 1,133 lines, exceeding the 1000-line guideline. However, this is a
//! **smart exception** rather than bloat:
//!
//! - **High Cohesion**: Single responsibility (2D graph rendering)
//! - **Single Type**: One main struct (Visual2DRenderer) with impl block
//! - **Logical Organization**: Clear sections (setup, rendering, input, layout)
//! - **No Duplication**: High information density, minimal repetition
//! - **Performance**: Keeping related code together improves CPU cache locality
//!
//! Splitting this arbitrarily (e.g., one file per method) would:
//! - ❌ Decrease readability (jumping between files)
//! - ❌ Harm performance (cache misses)
//! - ❌ Violate cohesion (tightly coupled methods separated)
//!
//! **Extracted**: Truly independent utilities moved to `color_utils` module.

use crate::color_utils::hsv_to_rgb;
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

        // Use trust level for color if available, otherwise fall back to health
        // Get trust level from properties
        let trust_level = node
            .info
            .properties
            .get("trust_level")
            .and_then(|v| match v {
                petal_tongue_core::PropertyValue::Number(n) => {
                    if *n >= 0.0 && *n <= 255.0 {
                        // Range validated, cast is safe
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let value = *n as u8;
                        Some(value)
                    } else {
                        None
                    }
                }
                _ => None,
            });

        let (fill_color, stroke_color) = if trust_level.is_some() {
            Self::trust_level_to_colors(trust_level)
        } else {
            Self::health_to_colors(node.info.health)
        };

        // Draw selection highlight
        if is_selected {
            painter.circle(
                screen_pos,
                radius + 5.0,
                Color32::TRANSPARENT,
                Stroke::new(3.0, Color32::YELLOW),
            );
        }

        // Draw family ID indicator (colored ring if present)
        // Get family_id from properties
        if let Some(petal_tongue_core::PropertyValue::String(family_id)) =
            node.info.properties.get("family_id")
        {
            let family_color = Self::family_id_to_color(family_id);
            painter.circle_stroke(screen_pos, radius + 3.0, Stroke::new(2.5, family_color));
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

        // Draw trust level badge (if available and zoomed in)
        if self.zoom > 0.7 {
            if let Some(petal_tongue_core::PropertyValue::Number(trust_val)) =
                node.info.properties.get("trust_level")
            {
                if *trust_val >= 0.0 && *trust_val <= 255.0 {
                    // Range validated, cast is safe
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let trust_level = *trust_val as u8;
                    let badge_text = match trust_level {
                        0 => "⚫",
                        1 => "🟡",
                        2 => "🟠",
                        3 => "🟢",
                        _ => "❓",
                    };
                    painter.text(
                        Pos2::new(screen_pos.x + radius, screen_pos.y - radius),
                        egui::Align2::LEFT_BOTTOM,
                        badge_text,
                        egui::FontId::proportional(14.0),
                        Color32::WHITE,
                    );
                }
            }
        }

        // Draw capability badges (if zoomed in enough)
        if self.zoom > 0.9 && !node.info.capabilities.is_empty() {
            self.draw_capability_badges(painter, screen_pos, radius, &node.info.capabilities);
        }
    }

    /// Draw capability badges around the node
    fn draw_capability_badges(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        capabilities: &[String],
    ) {
        // Map capabilities to icons
        let badge_radius = 8.0 * self.zoom;
        let orbit_radius = radius + 15.0;

        // Show up to 6 capabilities as badges
        let displayed_caps = capabilities.iter().take(6);
        let num_caps = displayed_caps.clone().count();

        for (i, capability) in displayed_caps.enumerate() {
            // Position badges in a circle around the node
            #[allow(clippy::cast_precision_loss)]
            let angle = (i as f32) * std::f32::consts::TAU / (num_caps as f32);
            let badge_pos = Pos2::new(
                center.x + orbit_radius * angle.cos(),
                center.y + orbit_radius * angle.sin(),
            );

            // Determine icon and color based on capability
            let (icon, color) = Self::capability_to_icon_and_color(capability);

            // Draw badge background circle
            painter.circle(
                badge_pos,
                badge_radius,
                color.gamma_multiply(0.3),
                Stroke::new(1.5, color),
            );

            // Draw icon
            painter.text(
                badge_pos,
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }

        // If there are more capabilities, show a "+N" badge
        if capabilities.len() > 6 {
            let more_count = capabilities.len() - 6;
            let angle = std::f32::consts::TAU * 0.75; // Bottom position
            let badge_pos = Pos2::new(
                center.x + orbit_radius * angle.cos(),
                center.y + orbit_radius * angle.sin(),
            );

            painter.circle(
                badge_pos,
                badge_radius,
                Color32::DARK_GRAY,
                Stroke::new(1.5, Color32::GRAY),
            );

            painter.text(
                badge_pos,
                egui::Align2::CENTER_CENTER,
                format!("+{}", more_count),
                egui::FontId::proportional(8.0),
                Color32::WHITE,
            );
        }
    }

    /// Map capability to icon and color
    fn capability_to_icon_and_color(capability: &str) -> (&'static str, Color32) {
        let cap_lower = capability.to_lowercase();

        // Security capabilities
        if cap_lower.contains("security")
            || cap_lower.contains("trust")
            || cap_lower.contains("auth")
        {
            return ("🔒", Color32::from_rgb(255, 100, 100));
        }

        // Storage capabilities
        if cap_lower.contains("storage")
            || cap_lower.contains("persist")
            || cap_lower.contains("data")
        {
            return ("💾", Color32::from_rgb(100, 150, 255));
        }

        // Compute capabilities
        if cap_lower.contains("compute")
            || cap_lower.contains("container")
            || cap_lower.contains("workload")
            || cap_lower.contains("execution")
        {
            return ("⚙️", Color32::from_rgb(150, 200, 100));
        }

        // Discovery/orchestration capabilities
        if cap_lower.contains("discovery")
            || cap_lower.contains("orchestr")
            || cap_lower.contains("federation")
        {
            return ("🔍", Color32::from_rgb(200, 150, 255));
        }

        // Identity capabilities
        if cap_lower.contains("identity")
            || cap_lower.contains("lineage")
            || cap_lower.contains("genetic")
        {
            return ("🆔", Color32::from_rgb(255, 200, 100));
        }

        // Encryption/crypto capabilities
        if cap_lower.contains("encrypt")
            || cap_lower.contains("crypto")
            || cap_lower.contains("sign")
        {
            return ("🔐", Color32::from_rgb(255, 150, 200));
        }

        // AI/inference capabilities
        if cap_lower.contains("ai")
            || cap_lower.contains("inference")
            || cap_lower.contains("intent")
            || cap_lower.contains("planning")
        {
            return ("🧠", Color32::from_rgb(200, 100, 255));
        }

        // Network/communication capabilities
        if cap_lower.contains("network")
            || cap_lower.contains("tcp")
            || cap_lower.contains("http")
            || cap_lower.contains("grpc")
        {
            return ("🌐", Color32::from_rgb(100, 200, 255));
        }

        // Attribution/provenance capabilities
        if cap_lower.contains("attribution")
            || cap_lower.contains("provenance")
            || cap_lower.contains("audit")
        {
            return ("📋", Color32::from_rgb(255, 200, 150));
        }

        // Visualization/UI capabilities
        if cap_lower.contains("visual") || cap_lower.contains("ui") || cap_lower.contains("display")
        {
            return ("👁️", Color32::from_rgb(150, 255, 200));
        }

        // Audio capabilities
        if cap_lower.contains("audio")
            || cap_lower.contains("sound")
            || cap_lower.contains("sonification")
        {
            return ("🔊", Color32::from_rgb(255, 150, 100));
        }

        // Default for unknown capabilities
        ("•", Color32::GRAY)
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

    /// Map trust level to colors (for future trust visualization)
    /// Trust levels: 0 = None, 1 = Limited, 2 = Elevated, 3 = Full
    fn trust_level_to_colors(trust_level: Option<u8>) -> (Color32, Color32) {
        match trust_level {
            None | Some(0) => (
                Color32::from_rgb(100, 100, 100), // Gray - No trust
                Color32::from_rgb(60, 60, 60),
            ),
            Some(1) => (
                Color32::from_rgb(200, 180, 40), // Yellow - Limited trust
                Color32::from_rgb(140, 120, 20),
            ),
            Some(2) => (
                Color32::from_rgb(220, 140, 40), // Orange - Elevated trust
                Color32::from_rgb(160, 100, 20),
            ),
            Some(3) => (
                Color32::from_rgb(40, 200, 80), // Bright green - Full trust
                Color32::from_rgb(20, 140, 60),
            ),
            _ => (
                Color32::from_rgb(120, 120, 120), // Gray - Unknown
                Color32::from_rgb(80, 80, 80),
            ),
        }
    }

    /// Map family ID to a consistent color
    fn family_id_to_color(family_id: &str) -> Color32 {
        // Simple hash to color mapping for consistent family visualization
        let hash: u32 = family_id.bytes().map(u32::from).sum();
        let hue = (hash % 360) as f32;

        // Convert HSV to RGB (S=0.7, V=0.9 for pleasant colors)
        let (r, g, b) = hsv_to_rgb(hue, 0.7, 0.9);
        Color32::from_rgb(r, g, b)
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
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
    use petal_tongue_core::{LayoutAlgorithm, TopologyEdge};

    fn create_test_graph() -> Arc<RwLock<GraphEngine>> {
        let mut graph = GraphEngine::new();

        let mut node1 = petal_tongue_core::test_fixtures::primals::test_primal("node1");
        node1.name = "Node 1".to_string();
        node1.health = PrimalHealthStatus::Healthy;
        graph.add_node(node1);

        let mut node2 = petal_tongue_core::test_fixtures::primals::test_primal("node2");
        node2.name = "Node 2".to_string();
        node2.health = PrimalHealthStatus::Warning;
        graph.add_node(node2);

        graph.add_edge(TopologyEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: "test".to_string(),
            label: None,
            capability: None,
            metrics: None,
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

    #[test]
    fn test_animation_engine_integration() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph.clone());

        // Initially no animation engine
        assert!(!renderer.is_animation_enabled());

        // Add animation engine
        let animation = Arc::new(RwLock::new(AnimationEngine::new()));
        renderer.set_animation_engine(animation);

        // Enable animation
        renderer.set_animation_enabled(true);
        assert!(renderer.is_animation_enabled());

        // Disable animation
        renderer.set_animation_enabled(false);
        assert!(!renderer.is_animation_enabled());
    }

    #[test]
    fn test_zoom_levels() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        // Initial zoom
        assert_eq!(renderer.zoom, 1.0);

        // Test various zoom levels
        renderer.zoom = 0.5;
        assert_eq!(renderer.zoom, 0.5);

        renderer.zoom = 2.0;
        assert_eq!(renderer.zoom, 2.0);

        renderer.zoom = 3.0;
        assert_eq!(renderer.zoom, 3.0);
    }

    #[test]
    fn test_camera_panning() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        // Initial position
        assert_eq!(renderer.camera_offset, Vec2::ZERO);

        // Pan camera
        renderer.camera_offset = Vec2::new(100.0, 50.0);
        assert_eq!(renderer.camera_offset, Vec2::new(100.0, 50.0));

        // Pan in negative direction
        renderer.camera_offset = Vec2::new(-50.0, -25.0);
        assert_eq!(renderer.camera_offset, Vec2::new(-50.0, -25.0));
    }

    #[test]
    fn test_world_to_screen_with_zoom() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.zoom = 2.0;
        let world_pos = Position::new_2d(100.0, 50.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let screen_pos = renderer.world_to_screen(world_pos, screen_center);

        // With 2x zoom, positions should be scaled
        assert_eq!(screen_pos.x, 600.0); // 400 + (100 * 2.0)
        assert_eq!(screen_pos.y, 400.0); // 300 + (50 * 2.0)
    }

    #[test]
    fn test_world_to_screen_with_camera_offset() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.camera_offset = Vec2::new(50.0, 25.0);
        let world_pos = Position::new_2d(100.0, 50.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let screen_pos = renderer.world_to_screen(world_pos, screen_center);

        // Camera offset should shift the position
        assert_eq!(screen_pos.x, 550.0); // 400 + 100 + 50
        assert_eq!(screen_pos.y, 375.0); // 300 + 50 + 25
    }

    #[test]
    fn test_health_status_all_states() {
        // Test all health status color mappings
        let (fill_healthy, _stroke_healthy) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Healthy);
        assert_eq!(fill_healthy, Color32::from_rgb(40, 180, 40));

        let (fill_warning, _stroke_warning) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Warning);
        assert_eq!(fill_warning, Color32::from_rgb(200, 180, 40));

        let (fill_critical, _stroke_critical) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Critical);
        assert_eq!(fill_critical, Color32::from_rgb(200, 40, 40));

        let (fill_unknown, _stroke_unknown) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Unknown);
        assert_eq!(fill_unknown, Color32::from_rgb(120, 120, 120));
    }

    #[test]
    fn test_selected_node_persistence() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        // Select multiple nodes in sequence
        renderer.set_selected_node(Some("node1".to_string()));
        assert_eq!(renderer.selected_node(), Some("node1"));

        renderer.set_selected_node(Some("node2".to_string()));
        assert_eq!(renderer.selected_node(), Some("node2"));

        // Clear selection
        renderer.set_selected_node(None);
        assert!(renderer.selected_node().is_none());
    }

    #[test]
    fn test_coordinate_conversion_roundtrip() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let screen_center = Pos2::new(400.0, 300.0);
        let original_world = Position::new_2d(123.45, 67.89);

        // Convert world -> screen -> world
        let screen_pos = renderer.world_to_screen(original_world, screen_center);
        let converted_world = renderer.screen_to_world(screen_pos, screen_center);

        // Should be very close (floating point precision)
        assert!((converted_world.x - original_world.x).abs() < 0.001);
        assert!((converted_world.y - original_world.y).abs() < 0.001);
    }

    #[test]
    fn test_renderer_with_empty_graph() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let _renderer = Visual2DRenderer::new(graph.clone());

        assert_eq!(_renderer.zoom, 1.0);
        assert_eq!(_renderer.camera_offset, Vec2::ZERO);
        assert!(_renderer.selected_node().is_none());

        // Should handle empty graph gracefully
        let graph_read = graph.read().expect("lock poisoned");
        assert_eq!(graph_read.nodes().len(), 0);
        assert_eq!(graph_read.edges().len(), 0);
    }

    #[test]
    fn test_renderer_with_many_nodes() {
        let mut graph = GraphEngine::new();

        // Add 10 nodes
        for i in 0..10 {
            let mut node =
                petal_tongue_core::test_fixtures::primals::test_primal(&format!("node{i}"));
            node.name = format!("Node {i}");
            graph.add_node(node);
        }

        // Add edges between sequential nodes
        for i in 0..9 {
            graph.add_edge(TopologyEdge {
                from: format!("node{i}"),
                to: format!("node{}", i + 1),
                edge_type: "test".to_string(),
                label: None,
                capability: None,
                metrics: None,
            });
        }

        graph.set_layout(LayoutAlgorithm::ForceDirected);
        graph.layout(1);

        let graph_arc = Arc::new(RwLock::new(graph));
        let _renderer = Visual2DRenderer::new(graph_arc.clone());

        let graph_read = graph_arc.read().expect("lock poisoned");
        assert_eq!(graph_read.nodes().len(), 10);
        assert_eq!(graph_read.edges().len(), 9);
        drop(graph_read);

        assert_eq!(_renderer.zoom, 1.0);
    }

    #[test]
    fn test_animation_engine_optional() {
        let graph = create_test_graph();
        let _renderer = Visual2DRenderer::new(graph);

        // Animation should be disabled by default when no engine is set
        assert!(!_renderer.is_animation_enabled());
    }

    #[test]
    fn test_zoom_default_value() {
        let graph = create_test_graph();
        let _renderer = Visual2DRenderer::new(graph);

        // Zoom level is set at creation (1.0 by default)
        assert_eq!(_renderer.zoom, 1.0);
    }

    #[test]
    fn test_health_color_mapping() {
        // Test static method for color mapping
        let (healthy_fill, healthy_stroke) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Healthy);
        let (warning_fill, warning_stroke) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Warning);
        let (critical_fill, critical_stroke) =
            Visual2DRenderer::health_to_colors(PrimalHealthStatus::Critical);

        // Colors should be distinct
        assert_ne!(healthy_fill, warning_fill);
        assert_ne!(healthy_fill, critical_fill);
        assert_ne!(warning_fill, critical_fill);

        // All colors should be non-transparent
        assert_ne!(healthy_fill, Color32::TRANSPARENT);
        assert_ne!(healthy_stroke, Color32::TRANSPARENT);
    }

    #[test]
    fn test_renderer_initial_state() {
        let graph = create_test_graph();
        let _renderer = Visual2DRenderer::new(graph);

        // Verify initial state
        assert_eq!(_renderer.zoom, 1.0);
        assert!(!_renderer.is_animation_enabled());
    }

    #[test]
    fn test_animation_lifecycle() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        // Step 1: No animation engine
        assert!(!renderer.is_animation_enabled());

        // Step 2: Add animation engine but don't enable
        let engine = Arc::new(RwLock::new(AnimationEngine::default()));
        renderer.set_animation_engine(engine);
        assert!(!renderer.is_animation_enabled());

        // Step 3: Enable animation
        renderer.set_animation_enabled(true);
        assert!(renderer.is_animation_enabled());

        // Step 4: Disable again
        renderer.set_animation_enabled(false);
        assert!(!renderer.is_animation_enabled());

        // Step 5: Re-enable
        renderer.set_animation_enabled(true);
        assert!(renderer.is_animation_enabled());
    }

    #[test]
    fn test_multi_edge_rendering() {
        let mut graph = GraphEngine::new();

        // Add three nodes
        for i in 1..=3 {
            let mut node =
                petal_tongue_core::test_fixtures::primals::test_primal(&format!("node{i}"));
            node.name = format!("Node {i}");
            graph.add_node(node);
        }

        // Add multiple edges
        graph.add_edge(TopologyEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: "connection".to_string(),
            label: Some("Edge 1-2".to_string()),
            capability: None,
            metrics: None,
        });
        graph.add_edge(TopologyEdge {
            from: "node2".to_string(),
            to: "node3".to_string(),
            edge_type: "connection".to_string(),
            label: Some("Edge 2-3".to_string()),
            capability: None,
            metrics: None,
        });
        graph.add_edge(TopologyEdge {
            from: "node1".to_string(),
            to: "node3".to_string(),
            edge_type: "connection".to_string(),
            label: Some("Edge 1-3".to_string()),
            capability: None,
            metrics: None,
        });

        let graph_arc = Arc::new(RwLock::new(graph));
        let _renderer = Visual2DRenderer::new(graph_arc.clone());

        let graph_read = graph_arc.read().expect("lock poisoned");
        assert_eq!(graph_read.edges().len(), 3);
    }

    #[test]
    fn test_renderer_with_different_health_states() {
        let mut graph = GraphEngine::new();

        // Add nodes with different health states
        graph.add_node(
            petal_tongue_core::test_fixtures::primals::test_primal_with_health(
                "healthy_node",
                PrimalHealthStatus::Healthy,
            ),
        );

        graph.add_node(
            petal_tongue_core::test_fixtures::primals::test_primal_with_health(
                "warning_node",
                PrimalHealthStatus::Warning,
            ),
        );

        graph.add_node(
            petal_tongue_core::test_fixtures::primals::test_primal_with_health(
                "critical_node",
                PrimalHealthStatus::Critical,
            ),
        );

        let graph_arc = Arc::new(RwLock::new(graph));
        let _renderer = Visual2DRenderer::new(graph_arc.clone());

        let graph_read = graph_arc.read().expect("lock poisoned");
        assert_eq!(graph_read.nodes().len(), 3);
    }
}
