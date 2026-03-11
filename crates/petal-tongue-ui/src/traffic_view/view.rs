// SPDX-License-Identifier: AGPL-3.0-only
//! Traffic View - main view logic and rendering

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::collections::HashMap;

use super::types::{ColorScheme, TrafficFlow, TrafficMetrics};

/// Bezier control points for flow curve from (from_x, from_y) to (to_x, to_y).
/// Returns (ctrl1, ctrl2) as [x, y] arrays.
#[must_use]
pub(crate) fn bezier_control_points(
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
) -> ([f32; 2], [f32; 2]) {
    let dx = to_x - from_x;
    let control_offset = dx.abs() * 0.3;
    let ctrl1 = [from_x + control_offset * dx.signum(), from_y];
    let ctrl2 = [to_x - control_offset * dx.signum(), to_y];
    (ctrl1, ctrl2)
}

/// Compute lane positions for primals in the traffic view.
/// Returns for each index: (y, left_center_x, right_center_x).
#[must_use]
pub(crate) fn primal_lane_layout(
    primal_count: usize,
    rect_min_x: f32,
    rect_min_y: f32,
    rect_max_x: f32,
    rect_max_y: f32,
    margin: f32,
    node_width: f32,
) -> Vec<(f32, f32, f32)> {
    if primal_count == 0 {
        return Vec::new();
    }
    let node_height = (rect_max_y - rect_min_y - 2.0 * margin) / primal_count as f32;
    let left_center_x = rect_min_x + margin + node_width / 2.0;
    let right_center_x = rect_max_x - margin - node_width / 2.0;
    (0..primal_count)
        .map(|i| {
            let y = rect_min_y + margin + node_height * (i as f32 + 0.5);
            (y, left_center_x, right_center_x)
        })
        .collect()
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
            self.primals.insert(primal.id.as_str().to_string(), primal);
        }
    }

    /// Clear all flows
    pub fn clear(&mut self) {
        self.flows.clear();
        self.selected_flow = None;
    }

    /// Get number of flows (for testing)
    #[must_use]
    pub fn flow_count(&self) -> usize {
        self.flows.len()
    }

    /// Get number of primals (for testing)
    #[must_use]
    pub fn primal_count(&self) -> usize {
        self.primals.len()
    }

    /// Get show_metrics (for testing)
    #[cfg(test)]
    #[must_use]
    pub fn show_metrics(&self) -> bool {
        self.show_metrics
    }

    /// Get flows (for testing)
    #[cfg(test)]
    #[must_use]
    pub fn flows(&self) -> &[TrafficFlow] {
        &self.flows
    }

    /// Update traffic metrics from topology edges
    pub fn update_from_topology(&mut self, edges: &[TopologyEdge]) {
        self.flows.clear();

        for edge in edges {
            // Create flow with default metrics
            // In production, these would come from telemetry
            let flow = TrafficFlow {
                from: edge.from.as_str().to_string(),
                to: edge.to.as_str().to_string(),
                metrics: TrafficMetrics {
                    bytes_per_second: 1000 + (edge.from.as_str().len() * 100) as u64,
                    requests_per_second: 10.0 + (edge.to.as_str().len() as f64 * 0.5),
                    avg_latency_ms: 5.0 + (edge.from.as_str().len() as f64 * 0.2),
                    error_rate: 0.01, // 1% default
                },
                color: Self::calculate_flow_color(&TrafficMetrics::default(), ColorScheme::Volume),
            };

            self.add_flow(flow);
        }
    }

    /// Calculate flow color based on metrics and scheme
    pub(crate) fn calculate_flow_color(metrics: &TrafficMetrics, scheme: ColorScheme) -> Color32 {
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
    pub(crate) fn calculate_flow_width(&self, metrics: &TrafficMetrics) -> f32 {
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

        let lane_layout = primal_lane_layout(
            primal_ids.len(),
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.max.y,
            margin,
            node_width,
        );

        let primal_positions: HashMap<_, _> = primal_ids
            .iter()
            .enumerate()
            .filter_map(|(i, id)| {
                lane_layout.get(i).map(|&(y, left_x, right_x)| {
                    (id.clone(), (Pos2::new(left_x, y), Pos2::new(right_x, y)))
                })
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

                let (ctrl1_arr, ctrl2_arr) = bezier_control_points(start.x, start.y, end.x, end.y);
                let ctrl1 = Pos2::new(ctrl1_arr[0], ctrl1_arr[1]);
                let ctrl2 = Pos2::new(ctrl2_arr[0], ctrl2_arr[1]);

                // Draw flow path
                self.draw_bezier_flow(&painter, start, ctrl1, ctrl2, end, width, flow.color);

                // Check for click
                let click_rect = Rect::from_center_size(
                    Pos2::new(f32::midpoint(start.x, end.x), f32::midpoint(start.y, end.y)),
                    Vec2::splat(30.0),
                );
                if response.clicked()
                    && let Some(pointer_pos) = response.interact_pointer_pos()
                    && click_rect.contains(pointer_pos)
                {
                    self.selected_flow = Some((flow.from.clone(), flow.to.clone()));
                }
            }
        }

        // Draw primal nodes
        for (i, primal_id) in primal_ids.iter().enumerate() {
            if let Some(&(y, left_x, right_x)) = lane_layout.get(i) {
                let left_pos = Pos2::new(left_x, y);
                let right_pos = Pos2::new(right_x, y);
                self.draw_primal_node(&painter, left_pos, primal_id, node_width);
                self.draw_primal_node(&painter, right_pos, primal_id, node_width);
            }
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
    fn bezier_control_points_left_to_right() {
        let (ctrl1, ctrl2) = bezier_control_points(100.0, 50.0, 400.0, 100.0);
        let dx = 400.0_f32 - 100.0;
        let offset = dx.abs() * 0.3;
        assert!((ctrl1[0] - (100.0 + offset)).abs() < f32::EPSILON);
        assert!((ctrl1[1] - 50.0).abs() < f32::EPSILON);
        assert!((ctrl2[0] - (400.0 - offset)).abs() < f32::EPSILON);
        assert!((ctrl2[1] - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bezier_control_points_right_to_left() {
        let (ctrl1, ctrl2) = bezier_control_points(400.0, 50.0, 100.0, 100.0);
        let dx: f32 = 100.0 - 400.0;
        let offset = dx.abs() * 0.3;
        assert!((ctrl1[0] - (400.0 + dx.signum() * offset)).abs() < f32::EPSILON);
        assert!((ctrl2[0] - (100.0 - dx.signum() * offset)).abs() < f32::EPSILON);
    }

    #[test]
    fn primal_lane_layout_empty() {
        let layout = primal_lane_layout(0, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
        assert!(layout.is_empty());
    }

    #[test]
    fn primal_lane_layout_single() {
        let layout = primal_lane_layout(1, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
        assert_eq!(layout.len(), 1);
        let (y, left_x, right_x) = layout[0];
        assert!((y - 300.0).abs() < f32::EPSILON);
        assert!((left_x - 80.0).abs() < f32::EPSILON);
        assert!((right_x - 720.0).abs() < f32::EPSILON);
    }

    #[test]
    fn primal_lane_layout_three() {
        let layout = primal_lane_layout(3, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
        assert_eq!(layout.len(), 3);
        let node_height = (600.0 - 40.0) / 3.0;
        for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
            let expected_y = 20.0 + node_height * (i as f32 + 0.5);
            assert!((y - expected_y).abs() < f32::EPSILON);
            assert!((left_x - 80.0).abs() < f32::EPSILON);
            assert!((right_x - 720.0).abs() < f32::EPSILON);
        }
    }
}
