// SPDX-License-Identifier: AGPL-3.0-or-later

//! egui painting for [`crate::graph_canvas::GraphCanvas`].

use crate::accessibility::ColorPalette;
use crate::graph_canvas::layout;
use crate::graph_canvas::{EdgeDrawState, GraphCanvas};
use egui::{Color32, Pos2, Rect, Stroke};
use petal_tongue_core::graph_builder::{GraphEdge, GraphNode};

use super::arrow::draw_arrow;
use super::geometry::{
    edge_color_rgb, grid_color_alpha, grid_line_positions, grid_params, node_colors,
    node_text_layout,
};

impl GraphCanvas {
    /// Draw grid
    pub(in crate::graph_canvas) fn draw_grid(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        palette: &ColorPalette,
    ) {
        let grid_color = Color32::from_rgba_premultiplied(
            palette.text_dim.r(),
            palette.text_dim.g(),
            palette.text_dim.b(),
            grid_color_alpha(),
        );

        let (grid_size, offset_x, offset_y) = grid_params(
            self.grid_size,
            self.camera.position.x,
            self.camera.position.y,
            self.camera.zoom,
        );

        for x in grid_line_positions(rect.min.x, rect.max.x, grid_size, offset_x) {
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, grid_color),
            );
        }

        for y in grid_line_positions(rect.min.y, rect.max.y, grid_size, offset_y) {
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, grid_color),
            );
        }
    }

    /// Draw a node
    pub(in crate::graph_canvas) fn draw_node(
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

        let (fill_rgb, stroke_rgb) = node_colors(
            self.selected_nodes.contains(&node.id),
            Some(&node.id) == self.hovered_node.as_ref(),
            node.visual_state.has_error,
        );
        let fill_color = Color32::from_rgb(fill_rgb[0], fill_rgb[1], fill_rgb[2]);
        let stroke_color = Color32::from_rgb(stroke_rgb[0], stroke_rgb[1], stroke_rgb[2]);

        painter.rect(
            node_rect,
            5.0,
            fill_color,
            Stroke::new(2.0 * self.camera.zoom, stroke_color),
        );

        let (text_size, icon_y, name_y) =
            node_text_layout(self.camera.zoom, node_rect.min.y, node_rect.max.y);
        if text_size > 8.0 {
            let icon = node.node_type.icon();
            let name = node.node_type.display_name();

            painter.text(
                Pos2::new(node_rect.center().x, icon_y),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(text_size * 1.2),
                palette.text,
            );

            painter.text(
                Pos2::new(node_rect.center().x, name_y),
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
    pub(in crate::graph_canvas) fn draw_edge(
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

            let accent_rgb = [palette.accent.r(), palette.accent.g(), palette.accent.b()];
            let edge_rgb = edge_color_rgb(&edge.edge_type, accent_rgb);
            let edge_color = Color32::from_rgb(edge_rgb[0], edge_rgb[1], edge_rgb[2]);

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
    pub(in crate::graph_canvas) fn draw_temporary_edge(
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
    pub(in crate::graph_canvas) fn draw_selection_box(
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
