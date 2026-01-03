//! Traffic View - Flow Analysis and Sankey Diagram
//!
//! Visualizes data flow and traffic patterns between primals.
//! Implements Phase 4 of the UI specification.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::collections::HashMap;

/// Traffic metrics for an edge
#[derive(Clone, Debug, Default)]
pub struct TrafficMetrics {
    /// Bytes transferred per second
    pub bytes_per_second: u64,
    /// Request count per second
    pub requests_per_second: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

/// Traffic flow visualization
#[derive(Clone, Debug)]
pub struct TrafficFlow {
    /// Source primal ID
    pub from: String,
    /// Destination primal ID
    pub to: String,
    /// Traffic metrics
    pub metrics: TrafficMetrics,
    /// Flow color (based on health/volume)
    pub color: Color32,
}

/// Traffic View - Sankey-style flow visualization
pub struct TrafficView {
    /// Traffic flows to display
    flows: Vec<TrafficFlow>,
    /// Primals in the view
    primals: HashMap<String, PrimalInfo>,
    /// Selected flow (for detail panel)
    selected_flow: Option<(String, String)>,
    /// Show metrics overlay
    show_metrics: bool,
    /// Color scheme (volume, latency, errors)
    color_scheme: ColorScheme,
    /// Minimum flow width
    min_flow_width: f32,
    /// Maximum flow width
    max_flow_width: f32,
}

/// Color scheme for traffic visualization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    /// Color by traffic volume
    Volume,
    /// Color by latency
    Latency,
    /// Color by error rate
    ErrorRate,
}

impl Default for TrafficView {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficView {
    /// Create a new traffic view
    #[must_use]
    pub fn new() -> Self {
        Self {
            flows: Vec::new(),
            primals: HashMap::new(),
            selected_flow: None,
            show_metrics: true,
            color_scheme: ColorScheme::Volume,
            min_flow_width: 2.0,
            max_flow_width: 40.0,
        }
    }

    /// Add a traffic flow
    pub fn add_flow(&mut self, flow: TrafficFlow) {
        self.flows.push(flow);
    }

    /// Set primals for the view
    pub fn set_primals(&mut self, primals: Vec<PrimalInfo>) {
        self.primals.clear();
        for primal in primals {
            self.primals.insert(primal.id.clone(), primal);
        }
    }

    /// Clear all flows
    pub fn clear(&mut self) {
        self.flows.clear();
        self.selected_flow = None;
    }

    /// Update traffic metrics from topology edges
    pub fn update_from_topology(&mut self, edges: &[TopologyEdge]) {
        self.flows.clear();

        for edge in edges {
            // Create flow with default metrics
            // In production, these would come from telemetry
            let flow = TrafficFlow {
                from: edge.from.clone(),
                to: edge.to.clone(),
                metrics: TrafficMetrics {
                    bytes_per_second: 1000 + (edge.from.len() * 100) as u64,
                    requests_per_second: 10.0 + (edge.to.len() as f64 * 0.5),
                    avg_latency_ms: 5.0 + (edge.from.len() as f64 * 0.2),
                    error_rate: 0.01, // 1% default
                },
                color: Self::calculate_flow_color(&TrafficMetrics::default(), ColorScheme::Volume),
            };

            self.add_flow(flow);
        }
    }

    /// Calculate flow color based on metrics and scheme
    fn calculate_flow_color(metrics: &TrafficMetrics, scheme: ColorScheme) -> Color32 {
        match scheme {
            ColorScheme::Volume => {
                // Green to red based on volume (blue channel = 32 for distinction)
                let normalized = (metrics.bytes_per_second as f32 / 100_000.0).min(1.0);
                Color32::from_rgb(
                    (255.0 * normalized) as u8,
                    (255.0 * (1.0 - normalized)) as u8,
                    32,
                )
            }
            ColorScheme::Latency => {
                // Green (fast) to red (slow) (blue channel = 64 for distinction)
                let normalized = (metrics.avg_latency_ms as f32 / 100.0).min(1.0);
                Color32::from_rgb(
                    (255.0 * normalized) as u8,
                    (255.0 * (1.0 - normalized)) as u8,
                    64,
                )
            }
            ColorScheme::ErrorRate => {
                // Green (no errors) to red (many errors) (blue channel = 96 for distinction)
                let normalized = (metrics.error_rate as f32 * 10.0).min(1.0);
                Color32::from_rgb(
                    (255.0 * normalized) as u8,
                    (255.0 * (1.0 - normalized) * 0.8) as u8, // Different green scaling
                    96,
                )
            }
        }
    }

    /// Calculate flow width based on volume
    fn calculate_flow_width(&self, metrics: &TrafficMetrics) -> f32 {
        let max_volume = self
            .flows
            .iter()
            .map(|f| f.metrics.bytes_per_second)
            .max()
            .unwrap_or(1);

        let normalized = metrics.bytes_per_second as f32 / max_volume as f32;
        self.min_flow_width + (self.max_flow_width - self.min_flow_width) * normalized
    }

    /// Render the traffic view
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Top control bar
        ui.horizontal(|ui| {
            ui.heading("🌊 Traffic View");
            ui.separator();

            // Color scheme selector
            ui.label("Color by:");
            if ui
                .selectable_label(self.color_scheme == ColorScheme::Volume, "Volume")
                .clicked()
            {
                self.color_scheme = ColorScheme::Volume;
                self.update_flow_colors();
            }
            if ui
                .selectable_label(self.color_scheme == ColorScheme::Latency, "Latency")
                .clicked()
            {
                self.color_scheme = ColorScheme::Latency;
                self.update_flow_colors();
            }
            if ui
                .selectable_label(self.color_scheme == ColorScheme::ErrorRate, "Errors")
                .clicked()
            {
                self.color_scheme = ColorScheme::ErrorRate;
                self.update_flow_colors();
            }

            ui.separator();

            // Metrics toggle
            if ui
                .checkbox(&mut self.show_metrics, "Show Metrics")
                .clicked()
            {
                // Toggle handled by checkbox
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    self.clear();
                }
                ui.label(format!("Flows: {}", self.flows.len()));
            });
        });

        ui.separator();

        // Main visualization
        if self.show_metrics && self.selected_flow.is_some() {
            ui.horizontal(|ui| {
                // Traffic diagram (70%)
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width() * 0.7, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.render_traffic_diagram(ui);
                    },
                );

                ui.separator();

                // Metrics panel (30%)
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.render_metrics_panel(ui);
                    },
                );
            });
        } else {
            // Full-width traffic diagram
            self.render_traffic_diagram(ui);
        }
    }

    /// Render the main traffic diagram (Sankey-style)
    fn render_traffic_diagram(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());

        let rect = response.rect;
        painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 25));

        if self.flows.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No traffic data to display",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
            return;
        }

        // Get unique primals
        let mut primal_ids: Vec<_> = self.primals.keys().cloned().collect();
        if primal_ids.is_empty() {
            // Fallback: extract from flows
            for flow in &self.flows {
                if !primal_ids.contains(&flow.from) {
                    primal_ids.push(flow.from.clone());
                }
                if !primal_ids.contains(&flow.to) {
                    primal_ids.push(flow.to.clone());
                }
            }
        }
        primal_ids.sort();

        if primal_ids.is_empty() {
            return;
        }

        // Layout: primals on left and right, flows in middle
        let node_width = 120.0;
        let margin = 20.0;
        let flow_area_width = rect.width() - 2.0 * (node_width + margin);
        let node_height = (rect.height() - margin * 2.0) / primal_ids.len() as f32;

        // Build primal positions
        let primal_positions: HashMap<_, _> = primal_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let y = rect.min.y + margin + node_height * (i as f32 + 0.5);
                let left_pos = Pos2::new(rect.min.x + margin + node_width / 2.0, y);
                let right_pos = Pos2::new(rect.max.x - margin - node_width / 2.0, y);
                (id.clone(), (left_pos, right_pos))
            })
            .collect();

        // Draw flows
        for flow in &self.flows {
            if let (Some((_from_left, from_right)), Some((to_left, _to_right))) = (
                primal_positions.get(&flow.from),
                primal_positions.get(&flow.to),
            ) {
                let width = self.calculate_flow_width(&flow.metrics);

                // Use right position of source and left position of target
                let start = *from_right;
                let end = *to_left;

                // Draw curved flow line (Bezier)
                let control_offset = flow_area_width * 0.3;
                let ctrl1 = Pos2::new(start.x + control_offset, start.y);
                let ctrl2 = Pos2::new(end.x - control_offset, end.y);

                // Draw flow path
                self.draw_bezier_flow(&painter, start, ctrl1, ctrl2, end, width, flow.color);

                // Check for click
                let click_rect = Rect::from_center_size(
                    Pos2::new(f32::midpoint(start.x, end.x), f32::midpoint(start.y, end.y)),
                    Vec2::splat(30.0),
                );
                if response.clicked()
                    && let Some(pointer_pos) = response.interact_pointer_pos()
                        && click_rect.contains(pointer_pos) {
                            self.selected_flow = Some((flow.from.clone(), flow.to.clone()));
                        }
            }
        }

        // Draw primal nodes
        for (i, primal_id) in primal_ids.iter().enumerate() {
            let y = rect.min.y + margin + node_height * (i as f32 + 0.5);

            // Left node (source)
            let left_pos = Pos2::new(rect.min.x + margin + node_width / 2.0, y);
            self.draw_primal_node(&painter, left_pos, primal_id, node_width);

            // Right node (destination)
            let right_pos = Pos2::new(rect.max.x - margin - node_width / 2.0, y);
            self.draw_primal_node(&painter, right_pos, primal_id, node_width);
        }
    }

    /// Draw a Bezier curve for flow visualization
    fn draw_bezier_flow(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        ctrl1: Pos2,
        ctrl2: Pos2,
        end: Pos2,
        width: f32,
        color: Color32,
    ) {
        // Approximate Bezier with line segments
        let segments = 20;
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            let x = mt3 * start.x + 3.0 * mt2 * t * ctrl1.x + 3.0 * mt * t2 * ctrl2.x + t3 * end.x;
            let y = mt3 * start.y + 3.0 * mt2 * t * ctrl1.y + 3.0 * mt * t2 * ctrl2.y + t3 * end.y;

            points.push(Pos2::new(x, y));
        }

        // Draw segments
        for i in 0..segments {
            painter.line_segment([points[i], points[i + 1]], Stroke::new(width, color));
        }
    }

    /// Draw a primal node
    fn draw_primal_node(&self, painter: &egui::Painter, pos: Pos2, primal_id: &str, width: f32) {
        let height = 30.0;
        let rect = Rect::from_center_size(pos, Vec2::new(width, height));

        // Background
        painter.rect_filled(rect, 5.0, Color32::from_rgb(40, 40, 50));
        painter.rect_stroke(
            rect,
            5.0,
            Stroke::new(1.0, Color32::from_rgb(100, 100, 120)),
        );

        // Label
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            primal_id,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    /// Render metrics detail panel
    fn render_metrics_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Flow Metrics");
        ui.separator();

        if let Some((ref from, ref to)) = self.selected_flow {
            if let Some(flow) = self.flows.iter().find(|f| &f.from == from && &f.to == to) {
                egui::Grid::new("traffic_metrics_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("From:");
                        ui.label(from);
                        ui.end_row();

                        ui.label("To:");
                        ui.label(to);
                        ui.end_row();

                        ui.label("Volume:");
                        ui.label(format!("{} B/s", flow.metrics.bytes_per_second));
                        ui.end_row();

                        ui.label("Requests:");
                        ui.label(format!("{:.1} req/s", flow.metrics.requests_per_second));
                        ui.end_row();

                        ui.label("Latency:");
                        ui.label(format!("{:.2} ms", flow.metrics.avg_latency_ms));
                        ui.end_row();

                        ui.label("Error Rate:");
                        ui.label(format!("{:.2}%", flow.metrics.error_rate * 100.0));
                        ui.end_row();
                    });

                ui.add_space(16.0);

                if ui.button("Close").clicked() {
                    self.selected_flow = None;
                }
            }
        } else {
            ui.label("Click on a flow to see metrics");
        }
    }

    /// Update flow colors based on current color scheme
    fn update_flow_colors(&mut self) {
        for flow in &mut self.flows {
            flow.color = Self::calculate_flow_color(&flow.metrics, self.color_scheme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_view_creation() {
        let view = TrafficView::new();
        assert_eq!(view.flows.len(), 0);
        assert!(view.show_metrics);
    }

    #[test]
    fn test_add_flow() {
        let mut view = TrafficView::new();

        let flow = TrafficFlow {
            from: "primal1".to_string(),
            to: "primal2".to_string(),
            metrics: TrafficMetrics::default(),
            color: Color32::GREEN,
        };

        view.add_flow(flow);
        assert_eq!(view.flows.len(), 1);
    }

    #[test]
    fn test_clear_flows() {
        let mut view = TrafficView::new();

        for i in 0..5 {
            view.add_flow(TrafficFlow {
                from: format!("primal{i}"),
                to: format!("primal{}", i + 1),
                metrics: TrafficMetrics::default(),
                color: Color32::GREEN,
            });
        }

        assert_eq!(view.flows.len(), 5);

        view.clear();
        assert_eq!(view.flows.len(), 0);
        assert!(view.selected_flow.is_none());
    }

    #[test]
    fn test_color_schemes() {
        let metrics = TrafficMetrics {
            bytes_per_second: 1000,
            requests_per_second: 10.0,
            avg_latency_ms: 50.0,
            error_rate: 0.05,
        };

        let volume_color = TrafficView::calculate_flow_color(&metrics, ColorScheme::Volume);
        let latency_color = TrafficView::calculate_flow_color(&metrics, ColorScheme::Latency);
        let error_color = TrafficView::calculate_flow_color(&metrics, ColorScheme::ErrorRate);

        // Colors should be different
        assert_ne!(volume_color, latency_color);
        assert_ne!(volume_color, error_color);
        assert_ne!(latency_color, error_color);
    }

    #[test]
    fn test_flow_width_calculation() {
        let mut view = TrafficView::new();

        // Add flows with different volumes
        view.add_flow(TrafficFlow {
            from: "a".to_string(),
            to: "b".to_string(),
            metrics: TrafficMetrics {
                bytes_per_second: 1000,
                ..Default::default()
            },
            color: Color32::GREEN,
        });

        view.add_flow(TrafficFlow {
            from: "b".to_string(),
            to: "c".to_string(),
            metrics: TrafficMetrics {
                bytes_per_second: 10000,
                ..Default::default()
            },
            color: Color32::GREEN,
        });

        let width1 = view.calculate_flow_width(&view.flows[0].metrics);
        let width2 = view.calculate_flow_width(&view.flows[1].metrics);

        // Higher volume should have wider flow
        assert!(width2 > width1);
    }
}
