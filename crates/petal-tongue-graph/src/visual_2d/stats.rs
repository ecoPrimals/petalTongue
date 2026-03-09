// SPDX-License-Identifier: AGPL-3.0-only
//! Statistics overlay for 2D graph visualization.

use petal_tongue_core::GraphEngine;

use super::renderer::Visual2DRenderer;

/// Draws the graph statistics overlay window.
pub fn draw_stats(renderer: &Visual2DRenderer, ui: &mut egui::Ui, graph: &GraphEngine) {
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
            ui.label(egui::RichText::new(format!("Avg Degree: {:.2}", stats.avg_degree)).strong());
            ui.label(
                egui::RichText::new(format!("Zoom: {:.2}x", renderer.zoom))
                    .color(egui::Color32::from_rgb(150, 200, 255)),
            );

            if let Some(selected_id) = &renderer.selected_node {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Selected:").weak());
                ui.label(
                    egui::RichText::new(selected_id.as_str())
                        .color(egui::Color32::from_rgb(255, 230, 150)),
                );
            }
        });
}
