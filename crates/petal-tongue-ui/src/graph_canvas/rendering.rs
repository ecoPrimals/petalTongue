// SPDX-License-Identifier: AGPL-3.0-only
//! Graph canvas rendering - grid, nodes, edges, selection box.

use crate::accessibility::ColorPalette;
use egui::{Color32, Pos2, Rect, Stroke, Vec2 as EguiVec2};
use petal_tongue_core::graph_builder::{EdgeType, GraphEdge, GraphNode};

use super::layout;
use super::{EdgeDrawState, GraphCanvas};

impl GraphCanvas {
    /// Draw grid
    pub(super) fn draw_grid(&self, painter: &egui::Painter, rect: Rect, palette: &ColorPalette) {
        let grid_color = Color32::from_rgba_premultiplied(
            palette.text_dim.r(),
            palette.text_dim.g(),
            palette.text_dim.b(),
            20,
        );

        let grid_size = self.grid_size * self.camera.zoom;
        let offset_x = (self.camera.position.x * self.camera.zoom) % grid_size;
        let offset_y = (self.camera.position.y * self.camera.zoom) % grid_size;

        let mut x = rect.min.x - offset_x;
        while x < rect.max.x {
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }

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
    pub(super) fn draw_node(
        &self,
        painter: &egui::Painter,
        node: &GraphNode,
        canvas_rect: Rect,
        palette: &ColorPalette,
    ) {
        let screen_pos = layout::world_to_screen(
            node.position,
            canvas_rect,
            &self.camera.position,
            self.camera.zoom,
        );
        let node_rect = Rect::from_center_size(screen_pos, self.node_size * self.camera.zoom);

        let (fill_color, stroke_color) = if self.selected_nodes.contains(&node.id) {
            (
                Color32::from_rgb(245, 166, 35),
                Color32::from_rgb(200, 130, 20),
            )
        } else if Some(&node.id) == self.hovered_node.as_ref() {
            (
                Color32::from_rgb(100, 150, 255),
                Color32::from_rgb(70, 120, 200),
            )
        } else if node.visual_state.has_error {
            (Color32::from_rgb(208, 2, 27), Color32::from_rgb(150, 0, 20))
        } else {
            (
                Color32::from_rgb(74, 144, 226),
                Color32::from_rgb(50, 100, 180),
            )
        };

        painter.rect(
            node_rect,
            5.0,
            fill_color,
            Stroke::new(2.0 * self.camera.zoom, stroke_color),
        );

        let text_size = 14.0 * self.camera.zoom;
        if text_size > 8.0 {
            let icon = node.node_type.icon();
            let name = node.node_type.display_name();

            painter.text(
                Pos2::new(
                    node_rect.center().x,
                    node_rect.min.y + 15.0 * self.camera.zoom,
                ),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(text_size * 1.2),
                palette.text,
            );

            painter.text(
                Pos2::new(
                    node_rect.center().x,
                    node_rect.max.y - 10.0 * self.camera.zoom,
                ),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::proportional(text_size * 0.8),
                palette.text,
            );

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
    pub(super) fn draw_edge(
        &self,
        painter: &egui::Painter,
        edge: &GraphEdge,
        canvas_rect: Rect,
        palette: &ColorPalette,
    ) {
        let from_node = self.graph.get_node(&edge.from);
        let to_node = self.graph.get_node(&edge.to);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let from_pos = layout::world_to_screen(
                from.position,
                canvas_rect,
                &self.camera.position,
                self.camera.zoom,
            );
            let to_pos = layout::world_to_screen(
                to.position,
                canvas_rect,
                &self.camera.position,
                self.camera.zoom,
            );

            let edge_color = match edge.edge_type {
                EdgeType::Dependency => palette.accent,
                EdgeType::DataFlow => Color32::from_rgb(150, 150, 150),
            };

            draw_arrow(
                painter,
                from_pos,
                to_pos,
                edge_color,
                &edge.edge_type,
                self.camera.zoom,
            );
        }
    }

    /// Draw temporary edge being created
    pub(super) fn draw_temporary_edge(
        &self,
        painter: &egui::Painter,
        edge_state: &EdgeDrawState,
        canvas_rect: Rect,
        palette: &ColorPalette,
    ) {
        if let Some(from_node) = self.graph.get_node(&edge_state.from_node) {
            let from_pos = layout::world_to_screen(
                from_node.position,
                canvas_rect,
                &self.camera.position,
                self.camera.zoom,
            );
            let to_pos = edge_state.current_pos;

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

            painter.circle_filled(to_pos, 4.0 * self.camera.zoom, palette.accent);
        }
    }

    /// Draw selection box
    pub(super) fn draw_selection_box(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        palette: &ColorPalette,
    ) {
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
}

fn draw_arrow(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    edge_type: &EdgeType,
    zoom: f32,
) {
    let stroke_width = 2.0 * zoom;
    let arrow_size = 8.0 * zoom;

    let stroke = match edge_type {
        EdgeType::Dependency => Stroke::new(stroke_width, color),
        EdgeType::DataFlow => Stroke::new(stroke_width, color),
    };

    painter.line_segment([from, to], stroke);

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
