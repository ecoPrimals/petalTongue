//! Graph Metrics Plotter Integration
//!
//! Visualizes graph metrics over time (node count, edge count, updates).
//! Demonstrates using petalTongue's own data as input to a tool.

#![allow(clippy::cast_precision_loss)]

use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use std::collections::VecDeque;
use std::time::Instant;

/// Metrics snapshot
#[derive(Clone, Debug)]
struct MetricsSnapshot {
    #[allow(dead_code)] // TODO: Use for time axis labels
    timestamp: Instant,
    node_count: usize,
    edge_count: usize,
}

/// Graph Metrics Plotter tool
///
/// Plots petalTongue's own graph metrics over time.
/// This demonstrates a tool that uses the host application's data.
pub struct GraphMetricsPlotter {
    show_panel: bool,
    history: VecDeque<MetricsSnapshot>,
    max_history: usize,
    #[allow(dead_code)] // TODO: Use for time-based x-axis
    start_time: Instant,
}

impl Default for GraphMetricsPlotter {
    fn default() -> Self {
        Self {
            show_panel: false,
            history: VecDeque::new(),
            max_history: 100, // Last 100 samples
            start_time: Instant::now(),
        }
    }
}

impl GraphMetricsPlotter {
    /// Add a metrics snapshot
    pub fn add_snapshot(&mut self, node_count: usize, edge_count: usize) {
        let snapshot = MetricsSnapshot {
            timestamp: Instant::now(),
            node_count,
            edge_count,
        };

        self.history.push_back(snapshot);

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Render metrics chart
    fn render_chart(&self, ui: &mut egui::Ui) {
        if self.history.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No data yet. Graph metrics will appear as data flows.")
                        .color(egui::Color32::GRAY),
                );
            });
            return;
        }

        ui.heading("📊 Graph Topology Metrics");
        ui.add_space(10.0);

        // Summary stats
        if let (Some(_first), Some(last)) = (self.history.front(), self.history.back()) {
            ui.horizontal(|ui| {
                ui.label("Nodes:");
                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{}", last.node_count));
                ui.separator();
                ui.label("Edges:");
                ui.colored_label(egui::Color32::LIGHT_GREEN, format!("{}", last.edge_count));
                ui.separator();
                ui.label("Samples:");
                ui.colored_label(egui::Color32::GRAY, format!("{}", self.history.len()));
            });
        }

        ui.add_space(10.0);

        // Node count chart
        Self::render_line_chart(
            ui,
            "Node Count Over Time",
            &self
                .history
                .iter()
                .map(|s| s.node_count as f32)
                .collect::<Vec<_>>(),
            egui::Color32::from_rgb(100, 150, 255),
        );

        ui.add_space(10.0);

        // Edge count chart
        Self::render_line_chart(
            ui,
            "Edge Count Over Time",
            &self
                .history
                .iter()
                .map(|s| s.edge_count as f32)
                .collect::<Vec<_>>(),
            egui::Color32::from_rgb(100, 255, 150),
        );
    }

    /// Render a line chart
    fn render_line_chart(ui: &mut egui::Ui, title: &str, data: &[f32], color: egui::Color32) {
        ui.label(egui::RichText::new(title).strong());

        let height = 100.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        if data.len() < 2 {
            return;
        }

        // Find min/max for scaling
        let max_value = data.iter().copied().fold(0.0f32, f32::max).max(1.0);
        let min_value = data.iter().copied().fold(max_value, f32::min).min(0.0);
        let range = (max_value - min_value).max(1.0);

        // Draw background
        painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(20, 20, 25));

        // Draw grid lines
        for i in 0..5 {
            let y = rect.top() + (i as f32 / 4.0) * rect.height();
            painter.line_segment(
                [
                    egui::Pos2::new(rect.left(), y),
                    egui::Pos2::new(rect.right(), y),
                ],
                egui::Stroke::new(0.5, egui::Color32::from_rgb(40, 40, 45)),
            );
        }

        // Draw data line
        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let x = rect.left() + (i as f32 / (data.len() - 1).max(1) as f32) * rect.width();
                let normalized = (value - min_value) / range;
                let y = rect.bottom() - normalized * rect.height();
                egui::Pos2::new(x, y)
            })
            .collect();

        painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));

        // Draw labels
        let label_text = format!("Max: {max_value:.0} | Min: {min_value:.0}");
        painter.text(
            egui::Pos2::new(rect.left() + 5.0, rect.top() + 5.0),
            egui::Align2::LEFT_TOP,
            label_text,
            egui::FontId::proportional(10.0),
            egui::Color32::GRAY,
        );
    }
}

impl ToolPanel for GraphMetricsPlotter {
    fn metadata(&self) -> &ToolMetadata {
        static METADATA: std::sync::OnceLock<ToolMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            name: "Graph Metrics".to_string(),
            description: "Visualize graph topology metrics over time".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![
                ToolCapability::Visual,
                ToolCapability::Custom("Plotting".to_string()),
                ToolCapability::Custom("Metrics".to_string()),
            ],
            icon: "📈".to_string(),
            source: None, // Built-in tool
        })
    }

    fn is_visible(&self) -> bool {
        self.show_panel
    }

    fn toggle_visibility(&mut self) {
        self.show_panel = !self.show_panel;
    }

    fn render_panel(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("📈 Graph Metrics Plotter").size(24.0));
            ui.label(
                egui::RichText::new("Real-time visualization of graph topology changes")
                    .size(14.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
        });

        ui.separator();
        ui.add_space(10.0);

        // Main content
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 30, 35))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    self.render_chart(ui);
                });
        });

        // Request repaint
        ui.ctx().request_repaint();
    }

    fn status_message(&self) -> Option<String> {
        if let Some(last) = self.history.back() {
            Some(format!("N:{} E:{}", last.node_count, last.edge_count))
        } else {
            Some("No data".to_string())
        }
    }
}
