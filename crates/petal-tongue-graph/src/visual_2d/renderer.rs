// SPDX-License-Identifier: AGPL-3.0-only
//! 2D Visual Renderer for graph topology.

use egui::{Color32, Pos2, Stroke, Vec2};
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::graph_engine::Position;
use petal_tongue_core::{GraphEngine, PrimalId};
use std::sync::{Arc, RwLock};

use super::animation;
use super::drawing;
use super::interaction;
use super::nodes;
use super::stats;
use super::types::EdgeDraft;

/// 2D Visual Renderer for graphs
pub struct Visual2DRenderer {
    /// Shared graph engine
    pub(crate) graph: Arc<RwLock<GraphEngine>>,
    /// Camera position (for panning)
    pub(crate) camera_offset: Vec2,
    /// Zoom level (1.0 = normal, 2.0 = 2x zoom)
    pub(crate) zoom: f32,
    /// Selected node ID
    pub(crate) selected_node: Option<PrimalId>,
    /// Is user currently dragging
    pub(crate) is_dragging: bool,
    /// Last mouse position (for drag delta)
    _last_mouse_pos: Option<Pos2>,
    /// Animation engine (optional, for flow visualization)
    animation_engine: Option<Arc<RwLock<AnimationEngine>>>,
    /// Animation enabled flag
    animation_enabled: bool,
    /// Show graph statistics window
    show_stats: bool,
    /// Interactive mode enabled (allows creating/editing nodes)
    pub(crate) interactive_mode: bool,
    /// Currently dragging a node (for moving)
    pub(crate) dragging_node: Option<PrimalId>,
    /// Edge being drawn (for connecting nodes)
    pub(crate) drawing_edge: Option<EdgeDraft>,
}

impl Visual2DRenderer {
    /// Create a new visual 2D renderer
    pub const fn new(graph: Arc<RwLock<GraphEngine>>) -> Self {
        Self {
            graph,
            camera_offset: Vec2::ZERO,
            zoom: 1.0,
            selected_node: None,
            is_dragging: false,
            _last_mouse_pos: None,
            animation_engine: None,
            animation_enabled: false,
            show_stats: true,
            interactive_mode: false,
            dragging_node: None,
            drawing_edge: None,
        }
    }

    /// Set the animation engine
    ///
    /// Enables flow particles and node pulse visualization.
    pub fn set_animation_engine(&mut self, engine: Arc<RwLock<AnimationEngine>>) {
        self.animation_engine = Some(engine);
    }

    /// Enable or disable animation
    pub const fn set_animation_enabled(&mut self, enabled: bool) {
        self.animation_enabled = enabled;
    }

    /// Check if animation is enabled
    #[must_use]
    pub const fn is_animation_enabled(&self) -> bool {
        self.animation_enabled
    }

    /// Enable or disable graph statistics window
    pub const fn set_show_stats(&mut self, show: bool) {
        self.show_stats = show;
    }

    /// Check if statistics window is enabled
    #[must_use]
    pub const fn show_stats(&self) -> bool {
        self.show_stats
    }

    /// Set the zoom level directly (motor efferent command).
    pub const fn set_zoom(&mut self, level: f32) {
        self.zoom = level.clamp(0.1, 10.0);
    }

    /// Fit all graph nodes into the viewport (motor efferent command).
    pub fn fit_to_view(&mut self, graph: &Arc<RwLock<GraphEngine>>) {
        let Ok(g) = graph.read() else { return };
        let nodes: Vec<_> = g.nodes().iter().collect();
        if nodes.is_empty() {
            return;
        }
        let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
        let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
        for node in &nodes {
            let p = node.position;
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        let cx = f32::midpoint(min_x, max_x);
        let cy = f32::midpoint(min_y, max_y);
        let span_x = (max_x - min_x).max(100.0);
        let span_y = (max_y - min_y).max(100.0);
        let span = span_x.max(span_y);
        self.zoom = (400.0 / span).clamp(0.1, 5.0);
        self.camera_offset = egui::Vec2::new(-cx * self.zoom, -cy * self.zoom);
    }

    /// Center on a specific node by ID (motor efferent command).
    pub fn navigate_to_node(&mut self, node_id: &str, graph: &Arc<RwLock<GraphEngine>>) {
        let Ok(g) = graph.read() else { return };
        if let Some(node) = g.get_node(node_id) {
            let p = node.position;
            self.camera_offset = egui::Vec2::new(-p.x * self.zoom, -p.y * self.zoom);
        }
    }

    /// Select a node by ID, or deselect all (motor efferent command).
    pub fn select_node(&mut self, node_id: Option<&str>) {
        self.selected_node = node_id.map(PrimalId::new);
    }

    /// Enable or disable interactive mode (create/edit nodes)
    pub const fn set_interactive_mode(&mut self, enabled: bool) {
        self.interactive_mode = enabled;
    }

    /// Check if interactive mode is enabled
    #[must_use]
    pub const fn is_interactive(&self) -> bool {
        self.interactive_mode
    }

    /// Render the graph to egui
    pub fn render(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (response, mut painter) =
            ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        let clip_rect = response.rect;
        let screen_center = clip_rect.center();
        interaction::handle_input(self, &response, screen_center);

        painter.set_clip_rect(clip_rect);

        let Ok(graph) = self.graph.read() else {
            tracing::error!("graph lock poisoned");
            return;
        };

        // Render edges first (so they appear behind nodes)
        for edge in graph.edges() {
            if let (Some(from_node), Some(to_node)) = (
                graph.get_node(edge.from.as_str()),
                graph.get_node(edge.to.as_str()),
            ) {
                let from_pos = self.world_to_screen(from_node.position, screen_center);
                let to_pos = self.world_to_screen(to_node.position, screen_center);

                painter.line_segment(
                    [from_pos, to_pos],
                    Stroke::new(2.0 * self.zoom, Color32::GRAY),
                );

                drawing::draw_arrow_head(&painter, from_pos, to_pos, self.zoom);
            }
        }

        // Render nodes
        for node in graph.nodes() {
            let screen_pos = self.world_to_screen(node.position, screen_center);

            if !clip_rect.expand(100.0).contains(screen_pos) {
                continue;
            }

            let is_selected = self
                .selected_node
                .as_ref()
                .is_some_and(|id| id.as_str() == node.info.id.as_str());

            nodes::draw_node(&painter, node, screen_pos, is_selected, self.zoom);
        }

        // Render animation (flow particles and pulses) if enabled
        if self.animation_enabled
            && let Some(animation_engine) = &self.animation_engine
            && let Ok(engine) = animation_engine.read()
        {
            animation::render_animation(self, &painter, &engine, &graph, screen_center);
        }

        // Draw edge being drafted (if in interactive mode)
        if let Some(ref edge_draft) = self.drawing_edge {
            if let Some(source_node) = graph.get_node(edge_draft.from.as_str()) {
                painter.line_segment(
                    [
                        self.world_to_screen(source_node.position, screen_center),
                        edge_draft.current_pos,
                    ],
                    Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                );
            } else {
                self.drawing_edge = None;
            }
        }

        if self.show_stats {
            stats::draw_stats(self, ui, &graph);
        }
    }

    /// Convert world coordinates to screen coordinates
    pub(crate) fn world_to_screen(&self, world_pos: Position, screen_center: Pos2) -> Pos2 {
        Pos2::new(
            world_pos.x.mul_add(self.zoom, screen_center.x) + self.camera_offset.x,
            world_pos.y.mul_add(self.zoom, screen_center.y) + self.camera_offset.y,
        )
    }

    /// Convert screen coordinates to world coordinates
    pub(crate) fn screen_to_world(&self, screen_pos: Pos2, screen_center: Pos2) -> Position {
        Position::new_2d(
            (screen_pos.x - screen_center.x - self.camera_offset.x) / self.zoom,
            (screen_pos.y - screen_center.y - self.camera_offset.y) / self.zoom,
        )
    }

    /// Reset camera to default position
    pub const fn reset_camera(&mut self) {
        self.camera_offset = Vec2::ZERO;
        self.zoom = 1.0;
    }

    /// Get selected node ID
    pub fn selected_node(&self) -> Option<&str> {
        self.selected_node.as_ref().map(PrimalId::as_str)
    }

    /// Set selected node
    pub fn set_selected_node(&mut self, node_id: Option<String>) {
        self.selected_node = node_id.map(PrimalId::from);
    }
}
