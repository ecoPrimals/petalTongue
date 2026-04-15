// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure layout and color math for graph canvas rendering (no egui painter calls).

use petal_tongue_core::graph_builder::EdgeType;

/// Node fill and stroke colors based on state.
/// Returns (`fill_rgb`, `stroke_rgb`).
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

/// Grid parameters for drawing: (`grid_size`, `offset_x`, `offset_y`).
#[must_use]
pub fn grid_params(
    base_grid_size: f32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    zoom: f32,
) -> (f32, f32, f32) {
    let grid_size = base_grid_size * zoom;
    let offset_x = (camera_pos_x * zoom) % grid_size;
    let offset_y = (camera_pos_y * zoom) % grid_size;
    (grid_size, offset_x, offset_y)
}

/// Node text layout: (`text_size`, `icon_y`, `name_y`) from zoom and node rect bounds.
#[must_use]
pub fn node_text_layout(zoom: f32, node_rect_min_y: f32, node_rect_max_y: f32) -> (f32, f32, f32) {
    let text_size = 14.0 * zoom;
    let icon_y = 15.0f32.mul_add(zoom, node_rect_min_y);
    let name_y = (-10.0f32).mul_add(zoom, node_rect_max_y);
    (text_size, icon_y, name_y)
}

#[must_use]
pub const fn grid_color_alpha() -> u8 {
    20
}

#[must_use]
pub fn edge_stroke_width(zoom: f32) -> f32 {
    2.0 * zoom
}

/// Grid line positions along one axis.
#[must_use]
pub fn grid_line_positions(rect_min: f32, rect_max: f32, grid_size: f32, offset: f32) -> Vec<f32> {
    let start = rect_min - offset;
    let count = ((rect_max - start) / grid_size).ceil().max(0.0) as usize;
    (0..count)
        .map(|i| (i as f32).mul_add(grid_size, start))
        .collect()
}
