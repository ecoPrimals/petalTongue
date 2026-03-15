// SPDX-License-Identifier: AGPL-3.0-only
//! Statistics overlay for 2D graph visualization.

use petal_tongue_core::graph_engine::GraphStats;
use petal_tongue_core::{GraphEngine, PrimalId};

use super::renderer::Visual2DRenderer;

#[must_use]
pub fn format_stats_lines(stats: &GraphStats, zoom: f32, selected_id: Option<&str>) -> Vec<String> {
    let mut lines = vec![
        format!("Nodes: {}", stats.node_count),
        format!("Edges: {}", stats.edge_count),
        format!("Avg Degree: {:.2}", stats.avg_degree),
        format!("Zoom: {:.2}x", zoom),
    ];
    if let Some(id) = selected_id {
        lines.push("Selected:".to_string());
        lines.push(id.to_string());
    }
    lines
}

/// Draws the graph statistics overlay window.
pub fn draw_stats(renderer: &Visual2DRenderer, ui: &mut egui::Ui, graph: &GraphEngine) {
    let stats = graph.stats();
    let lines = format_stats_lines(
        &stats,
        renderer.zoom,
        renderer.selected_node.as_ref().map(PrimalId::as_str),
    );

    egui::Window::new("📊 Graph Statistics")
        .fixed_pos([10.0, 10.0])
        .default_width(220.0)
        .collapsible(true)
        .frame(
            egui::Frame::window(&ui.ctx().style())
                .fill(egui::Color32::from_rgba_premultiplied(40, 40, 45, 230)),
        )
        .show(ui.ctx(), |ui| {
            ui.label(egui::RichText::new(&lines[0]).strong());
            ui.label(egui::RichText::new(&lines[1]).strong());
            ui.label(egui::RichText::new(&lines[2]).strong());
            ui.label(egui::RichText::new(&lines[3]).color(egui::Color32::from_rgb(150, 200, 255)));

            if lines.len() > 4 {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Selected:").weak());
                ui.label(
                    egui::RichText::new(&lines[5]).color(egui::Color32::from_rgb(255, 230, 150)),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::graph_engine::GraphStats;

    #[test]
    fn format_stats_lines_basic() {
        let stats = GraphStats {
            node_count: 5,
            edge_count: 8,
            avg_degree: 1.6,
        };
        let lines = format_stats_lines(&stats, 1.5, None);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "Nodes: 5");
        assert_eq!(lines[1], "Edges: 8");
        assert_eq!(lines[2], "Avg Degree: 1.60");
        assert_eq!(lines[3], "Zoom: 1.50x");
    }

    #[test]
    fn format_stats_lines_with_selected() {
        let stats = GraphStats {
            node_count: 1,
            edge_count: 0,
            avg_degree: 0.0,
        };
        let lines = format_stats_lines(&stats, 2.0, Some("node-1"));
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[4], "Selected:");
        assert_eq!(lines[5], "node-1");
    }

    #[test]
    fn format_stats_lines_empty_graph() {
        let stats = GraphStats {
            node_count: 0,
            edge_count: 0,
            avg_degree: 0.0,
        };
        let lines = format_stats_lines(&stats, 1.0, None);
        assert_eq!(lines[0], "Nodes: 0");
        assert_eq!(lines[1], "Edges: 0");
    }
}
