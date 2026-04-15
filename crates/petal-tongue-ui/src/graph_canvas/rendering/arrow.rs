// SPDX-License-Identifier: AGPL-3.0-or-later

//! Arrowhead stroke and fill for directed edges.

use egui::{Color32, Pos2, Stroke};

use petal_tongue_core::graph_builder::EdgeType;

use super::geometry::{arrow_geometry, edge_stroke_width};

pub fn draw_arrow(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    edge_type: &EdgeType,
    zoom: f32,
) {
    let stroke_width = edge_stroke_width(zoom);
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
