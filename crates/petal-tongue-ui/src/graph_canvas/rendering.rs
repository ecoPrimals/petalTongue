// SPDX-License-Identifier: AGPL-3.0-only
//! Graph canvas rendering - grid, nodes, edges, selection box.

use crate::accessibility::ColorPalette;
use egui::{Color32, Pos2, Rect, Stroke};
use petal_tongue_core::graph_builder::{EdgeType, GraphEdge, GraphNode};

use super::layout;
use super::{EdgeDrawState, GraphCanvas};

// --- Pure logic (testable, no egui) ---

/// Node fill and stroke colors based on state.
/// Returns (fill_rgb, stroke_rgb).
#[must_use]
pub const fn node_colors(selected: bool, hovered: bool, has_error: bool) -> ([u8; 3], [u8; 3]) {
    if selected {
        ([245, 166, 35], [200, 130, 20])
    } else if hovered {
        ([100, 150, 255], [70, 120, 200])
    } else if has_error {
        ([208, 2, 27], [150, 0, 20])
    } else {
        ([74, 144, 226], [50, 100, 180])
    }
}

/// Edge color as RGB based on edge type and accent.
#[must_use]
pub const fn edge_color_rgb(edge_type: &EdgeType, accent: [u8; 3]) -> [u8; 3] {
    match edge_type {
        EdgeType::Dependency => accent,
        EdgeType::DataFlow => [150, 150, 150],
    }
}

/// Arrow triangle geometry for directed edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowPoints {
    pub tip: [f32; 2],
    pub left: [f32; 2],
    pub right: [f32; 2],
}

/// Compute arrow triangle vertices from line segment and zoom.
#[must_use]
pub fn arrow_geometry(from: [f32; 2], to: [f32; 2], zoom: f32) -> ArrowPoints {
    let arrow_size = 8.0 * zoom;
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let len = dx.hypot(dy);
    if len < f32::EPSILON {
        return ArrowPoints {
            tip: to,
            left: to,
            right: to,
        };
    }
    let dir_x = dx / len;
    let dir_y = dy / len;
    let perp_x = -dir_y;
    let perp_y = dir_x;
    let base_x = (dir_x * arrow_size).mul_add(-2.0, to[0]);
    let base_y = (dir_y * arrow_size).mul_add(-2.0, to[1]);
    ArrowPoints {
        tip: to,
        left: [
            perp_x.mul_add(arrow_size, base_x),
            perp_y.mul_add(arrow_size, base_y),
        ],
        right: [
            perp_x.mul_add(-arrow_size, base_x),
            perp_y.mul_add(-arrow_size, base_y),
        ],
    }
}

/// Grid line positions along one axis.
#[must_use]
pub fn grid_line_positions(rect_min: f32, rect_max: f32, grid_size: f32, offset: f32) -> Vec<f32> {
    let mut positions = Vec::new();
    let mut x = rect_min - offset;
    while x < rect_max {
        positions.push(x);
        x += grid_size;
    }
    positions
}

// --- Rendering (uses egui) ---

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

        let text_size = 14.0 * self.camera.zoom;
        if text_size > 8.0 {
            let icon = node.node_type.icon();
            let name = node.node_type.display_name();

            painter.text(
                Pos2::new(
                    node_rect.center().x,
                    15.0f32.mul_add(self.camera.zoom, node_rect.min.y),
                ),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(text_size * 1.2),
                palette.text,
            );

            painter.text(
                Pos2::new(
                    node_rect.center().x,
                    10.0f32.mul_add(-self.camera.zoom, node_rect.max.y),
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
    let stroke = match edge_type {
        EdgeType::Dependency => Stroke::new(stroke_width, color),
        EdgeType::DataFlow => Stroke::new(stroke_width, color),
    };

    painter.line_segment([from, to], stroke);

    let from_arr = [from.x, from.y];
    let to_arr = [to.x, to.y];
    let points = arrow_geometry(from_arr, to_arr, zoom);

    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(points.tip[0], points.tip[1]),
            Pos2::new(points.left[0], points.left[1]),
            Pos2::new(points.right[0], points.right[1]),
        ],
        color,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::graph_builder::EdgeType;

    #[test]
    fn test_node_colors_selected() {
        let (fill, stroke) = node_colors(true, false, false);
        assert_eq!(fill, [245, 166, 35]);
        assert_eq!(stroke, [200, 130, 20]);
    }

    #[test]
    fn test_node_colors_hovered() {
        let (fill, stroke) = node_colors(false, true, false);
        assert_eq!(fill, [100, 150, 255]);
        assert_eq!(stroke, [70, 120, 200]);
    }

    #[test]
    fn test_node_colors_error() {
        let (fill, stroke) = node_colors(false, false, true);
        assert_eq!(fill, [208, 2, 27]);
        assert_eq!(stroke, [150, 0, 20]);
    }

    #[test]
    fn test_node_colors_default() {
        let (fill, stroke) = node_colors(false, false, false);
        assert_eq!(fill, [74, 144, 226]);
        assert_eq!(stroke, [50, 100, 180]);
    }

    #[test]
    fn test_node_colors_priority_selected_over_hovered() {
        let (fill, _) = node_colors(true, true, false);
        assert_eq!(fill, [245, 166, 35]);
    }

    #[test]
    fn test_edge_color_rgb_dependency() {
        let accent = [100, 150, 200];
        let rgb = edge_color_rgb(&EdgeType::Dependency, accent);
        assert_eq!(rgb, accent);
    }

    #[test]
    fn test_edge_color_rgb_data_flow() {
        let rgb = edge_color_rgb(&EdgeType::DataFlow, [0, 0, 0]);
        assert_eq!(rgb, [150, 150, 150]);
    }

    #[test]
    fn test_arrow_geometry_normal() {
        let from = [0.0, 0.0];
        let to = [100.0, 0.0];
        let zoom = 1.0;
        let points = arrow_geometry(from, to, zoom);

        assert_eq!(points.tip, to);
        assert!((points.left[0] - 84.0).abs() < f32::EPSILON);
        assert!((points.left[1] - 8.0).abs() < f32::EPSILON);
        assert!((points.right[0] - 84.0).abs() < f32::EPSILON);
        assert!((points.right[1] - (-8.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_arrow_geometry_zero_length() {
        let from = [50.0, 50.0];
        let to = [50.0, 50.0];
        let points = arrow_geometry(from, to, 1.0);

        assert_eq!(points.tip, to);
        assert_eq!(points.left, to);
        assert_eq!(points.right, to);
    }

    #[test]
    fn test_arrow_geometry_scales_with_zoom() {
        let from = [0.0, 0.0];
        let to = [100.0, 0.0];
        let points_zoom1 = arrow_geometry(from, to, 1.0);
        let points_zoom2 = arrow_geometry(from, to, 2.0);

        let width1 = (points_zoom1.left[1] - points_zoom1.right[1]).abs();
        let width2 = (points_zoom2.left[1] - points_zoom2.right[1]).abs();
        assert!(width2 > width1);
    }

    #[test]
    fn test_grid_line_positions() {
        let positions = grid_line_positions(0.0, 100.0, 25.0, 0.0);
        assert!(!positions.is_empty());
        assert!((positions[0] - 0.0).abs() < f32::EPSILON);
        let last = positions.last().expect("non-empty");
        assert!(*last < 100.0);
        assert!(*last >= 75.0);
    }

    #[test]
    fn test_grid_line_positions_with_offset() {
        let positions = grid_line_positions(0.0, 50.0, 20.0, 5.0);
        assert!(!positions.is_empty());
        assert!((positions[0] - (-5.0)).abs() < f32::EPSILON);
    }
}
