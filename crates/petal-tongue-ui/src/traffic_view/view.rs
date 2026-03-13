// SPDX-License-Identifier: AGPL-3.0-only
//! Traffic View - main view logic and rendering
//!
//! Architecture: headless-first. Pure geometry functions (`bezier_control_points`,
//! `primal_lane_layout`, `calculate_flow_color`, `calculate_flow_width`) are
//! fully testable. The render method returns `Vec<TrafficIntent>` instead of
//! mutating `self` directly for user interactions.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::collections::HashMap;

use super::types::{ColorScheme, TrafficFlow, TrafficMetrics};

// ============================================================================
// Intent type (headless-testable, no egui context needed)
// ============================================================================

/// User interaction intent produced by the traffic view render method.
/// The caller applies these after render returns, keeping the render
/// method free of side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrafficIntent {
    SetColorScheme(ColorScheme),
    SelectFlow { from: String, to: String },
    CloseDetails,
    ToggleMetrics,
    Clear,
}

// ============================================================================
// Display state for metrics panel (headless-testable)
// ============================================================================

/// Pre-computed detail display for a selected flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowDetailDisplay {
    pub from: String,
    pub to: String,
    pub volume_label: String,
    pub requests_label: String,
    pub latency_label: String,
    pub error_rate_label: String,
}

/// Build a `FlowDetailDisplay` from a flow (pure, no egui).
#[must_use]
pub fn prepare_flow_detail(flow: &TrafficFlow) -> FlowDetailDisplay {
    FlowDetailDisplay {
        from: flow.from.clone(),
        to: flow.to.clone(),
        volume_label: format!("{} B/s", flow.metrics.bytes_per_second),
        requests_label: format!("{:.1} req/s", flow.metrics.requests_per_second),
        latency_label: format!("{:.2} ms", flow.metrics.avg_latency_ms),
        error_rate_label: format!("{:.2}%", flow.metrics.error_rate * 100.0),
    }
}

// ============================================================================
// Pure geometry / color functions
// ============================================================================

/// Bezier control points for flow curve from (from_x, from_y) to (to_x, to_y).
/// Returns (ctrl1, ctrl2) as [x, y] arrays.
#[must_use]
pub fn bezier_control_points(
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
pub fn primal_lane_layout(
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
    let node_height = 2.0f32.mul_add(-margin, rect_max_y - rect_min_y) / primal_count as f32;
    let left_center_x = rect_min_x + margin + node_width / 2.0;
    let right_center_x = rect_max_x - margin - node_width / 2.0;
    (0..primal_count)
        .map(|i| {
            let y = node_height.mul_add(i as f32 + 0.5, rect_min_y + margin);
            (y, left_center_x, right_center_x)
        })
        .collect()
}

/// Calculate flow color based on metrics and scheme (pure function).
#[must_use]
pub fn calculate_flow_color(metrics: &TrafficMetrics, scheme: ColorScheme) -> Color32 {
    match scheme {
        ColorScheme::Volume => {
            let normalized = (metrics.bytes_per_second as f32 / 100_000.0).min(1.0);
            Color32::from_rgb(
                (255.0 * normalized) as u8,
                (255.0 * (1.0 - normalized)) as u8,
                32,
            )
        }
        ColorScheme::Latency => {
            let normalized = (metrics.avg_latency_ms as f32 / 100.0).min(1.0);
            Color32::from_rgb(
                (255.0 * normalized) as u8,
                (255.0 * (1.0 - normalized)) as u8,
                64,
            )
        }
        ColorScheme::ErrorRate => {
            let normalized = (metrics.error_rate as f32 * 10.0).min(1.0);
            Color32::from_rgb(
                (255.0 * normalized) as u8,
                (255.0 * (1.0 - normalized) * 0.8) as u8,
                96,
            )
        }
    }
}

/// Calculate flow width based on volume relative to max (pure function).
#[must_use]
pub fn calculate_flow_width(
    metrics: &TrafficMetrics,
    max_volume: u64,
    min_width: f32,
    max_width: f32,
) -> f32 {
    let max_vol = max_volume.max(1);
    let normalized = metrics.bytes_per_second as f32 / max_vol as f32;
    (max_width - min_width).mul_add(normalized, min_width)
}

// ============================================================================
// TrafficView
// ============================================================================

/// Traffic View - Sankey-style flow visualization
pub struct TrafficView {
    flows: Vec<TrafficFlow>,
    primals: HashMap<String, PrimalInfo>,
    selected_flow: Option<(String, String)>,
    show_metrics: bool,
    color_scheme: ColorScheme,
    min_flow_width: f32,
    max_flow_width: f32,
}

impl Default for TrafficView {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficView {
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

    pub fn add_flow(&mut self, flow: TrafficFlow) {
        self.flows.push(flow);
    }

    pub fn set_primals(&mut self, primals: Vec<PrimalInfo>) {
        self.primals.clear();
        for primal in primals {
            self.primals.insert(primal.id.as_str().to_string(), primal);
        }
    }

    pub fn clear(&mut self) {
        self.flows.clear();
        self.selected_flow = None;
    }

    #[must_use]
    pub const fn flow_count(&self) -> usize {
        self.flows.len()
    }

    #[must_use]
    pub fn primal_count(&self) -> usize {
        self.primals.len()
    }

    #[cfg(test)]
    #[must_use]
    pub const fn show_metrics(&self) -> bool {
        self.show_metrics
    }

    #[cfg(test)]
    #[must_use]
    pub fn flows(&self) -> &[TrafficFlow] {
        &self.flows
    }

    /// Read-only access to the currently selected flow (for headless testing).
    #[must_use]
    pub const fn selected_flow(&self) -> Option<&(String, String)> {
        self.selected_flow.as_ref()
    }

    /// Read-only access to the current color scheme (for headless testing).
    #[must_use]
    pub const fn color_scheme(&self) -> ColorScheme {
        self.color_scheme
    }

    /// Update traffic metrics from topology edges
    pub fn update_from_topology(&mut self, edges: &[TopologyEdge]) {
        self.flows.clear();

        for edge in edges {
            let flow = TrafficFlow {
                from: edge.from.as_str().to_string(),
                to: edge.to.as_str().to_string(),
                metrics: TrafficMetrics {
                    bytes_per_second: 1000 + (edge.from.as_str().len() * 100) as u64,
                    requests_per_second: (edge.to.as_str().len() as f64).mul_add(0.5, 10.0),
                    avg_latency_ms: (edge.from.as_str().len() as f64).mul_add(0.2, 5.0),
                    error_rate: 0.01,
                },
                color: calculate_flow_color(&TrafficMetrics::default(), ColorScheme::Volume),
            };

            self.add_flow(flow);
        }
    }

    /// Apply a list of intents to this view. Call after `render()`.
    pub fn apply_intents(&mut self, intents: &[TrafficIntent]) {
        for intent in intents {
            match intent {
                TrafficIntent::SetColorScheme(scheme) => {
                    self.color_scheme = *scheme;
                    self.update_flow_colors();
                }
                TrafficIntent::SelectFlow { from, to } => {
                    self.selected_flow = Some((from.clone(), to.clone()));
                }
                TrafficIntent::CloseDetails => {
                    self.selected_flow = None;
                }
                TrafficIntent::ToggleMetrics => {
                    self.show_metrics = !self.show_metrics;
                }
                TrafficIntent::Clear => {
                    self.clear();
                }
            }
        }
    }

    /// Max volume across all current flows.
    fn max_volume(&self) -> u64 {
        self.flows
            .iter()
            .map(|f| f.metrics.bytes_per_second)
            .max()
            .unwrap_or(1)
    }

    /// Render the traffic view. Returns intents for the caller to apply.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<TrafficIntent> {
        let mut intents = Vec::new();

        ui.horizontal(|ui| {
            ui.heading("🌊 Traffic View");
            ui.separator();

            ui.label("Color by:");
            if ui
                .selectable_label(self.color_scheme == ColorScheme::Volume, "Volume")
                .clicked()
            {
                intents.push(TrafficIntent::SetColorScheme(ColorScheme::Volume));
            }
            if ui
                .selectable_label(self.color_scheme == ColorScheme::Latency, "Latency")
                .clicked()
            {
                intents.push(TrafficIntent::SetColorScheme(ColorScheme::Latency));
            }
            if ui
                .selectable_label(self.color_scheme == ColorScheme::ErrorRate, "Errors")
                .clicked()
            {
                intents.push(TrafficIntent::SetColorScheme(ColorScheme::ErrorRate));
            }

            ui.separator();

            let mut metrics_val = self.show_metrics;
            if ui.checkbox(&mut metrics_val, "Show Metrics").clicked() {
                intents.push(TrafficIntent::ToggleMetrics);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    intents.push(TrafficIntent::Clear);
                }
                ui.label(format!("Flows: {}", self.flows.len()));
            });
        });

        ui.separator();

        if self.show_metrics && self.selected_flow.is_some() {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width() * 0.7, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let diagram_intents = self.render_traffic_diagram(ui);
                        intents.extend(diagram_intents);
                    },
                );

                ui.separator();

                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let panel_intents = self.render_metrics_panel(ui);
                        intents.extend(panel_intents);
                    },
                );
            });
        } else {
            let diagram_intents = self.render_traffic_diagram(ui);
            intents.extend(diagram_intents);
        }

        intents
    }

    /// Render the main traffic diagram (Sankey-style). Returns click intents.
    fn render_traffic_diagram(&self, ui: &mut egui::Ui) -> Vec<TrafficIntent> {
        let mut intents = Vec::new();
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
            return intents;
        }

        let mut primal_ids: Vec<_> = self.primals.keys().cloned().collect();
        if primal_ids.is_empty() {
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
            return intents;
        }

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

        let max_vol = self.max_volume();

        for flow in &self.flows {
            if let (Some((_from_left, from_right)), Some((to_left, _to_right))) = (
                primal_positions.get(&flow.from),
                primal_positions.get(&flow.to),
            ) {
                let width = calculate_flow_width(
                    &flow.metrics,
                    max_vol,
                    self.min_flow_width,
                    self.max_flow_width,
                );

                let start = *from_right;
                let end = *to_left;

                let (ctrl1_arr, ctrl2_arr) = bezier_control_points(start.x, start.y, end.x, end.y);
                let ctrl1 = Pos2::new(ctrl1_arr[0], ctrl1_arr[1]);
                let ctrl2 = Pos2::new(ctrl2_arr[0], ctrl2_arr[1]);

                Self::draw_bezier_flow(&painter, start, ctrl1, ctrl2, end, width, flow.color);

                let click_rect = Rect::from_center_size(
                    Pos2::new(f32::midpoint(start.x, end.x), f32::midpoint(start.y, end.y)),
                    Vec2::splat(30.0),
                );
                if response.clicked()
                    && let Some(pointer_pos) = response.interact_pointer_pos()
                    && click_rect.contains(pointer_pos)
                {
                    intents.push(TrafficIntent::SelectFlow {
                        from: flow.from.clone(),
                        to: flow.to.clone(),
                    });
                }
            }
        }

        for (i, primal_id) in primal_ids.iter().enumerate() {
            if let Some(&(y, left_x, right_x)) = lane_layout.get(i) {
                let left_pos = Pos2::new(left_x, y);
                let right_pos = Pos2::new(right_x, y);
                Self::draw_primal_node(&painter, left_pos, primal_id, node_width);
                Self::draw_primal_node(&painter, right_pos, primal_id, node_width);
            }
        }

        intents
    }

    fn draw_bezier_flow(
        painter: &egui::Painter,
        start: Pos2,
        ctrl1: Pos2,
        ctrl2: Pos2,
        end: Pos2,
        width: f32,
        color: Color32,
    ) {
        let segments = 20;
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            let x = (3.0 * mt * t2).mul_add(ctrl2.x, mt3 * start.x + 3.0 * mt2 * t * ctrl1.x)
                + t3 * end.x;
            let y = (3.0 * mt * t2).mul_add(ctrl2.y, mt3 * start.y + 3.0 * mt2 * t * ctrl1.y)
                + t3 * end.y;

            points.push(Pos2::new(x, y));
        }

        for i in 0..segments {
            painter.line_segment([points[i], points[i + 1]], Stroke::new(width, color));
        }
    }

    fn draw_primal_node(painter: &egui::Painter, pos: Pos2, primal_id: &str, width: f32) {
        let height = 30.0;
        let rect = Rect::from_center_size(pos, Vec2::new(width, height));

        painter.rect_filled(rect, 5.0, Color32::from_rgb(40, 40, 50));
        painter.rect_stroke(
            rect,
            5.0,
            Stroke::new(1.0, Color32::from_rgb(100, 100, 120)),
        );

        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            primal_id,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    /// Render metrics detail panel. Returns intents.
    fn render_metrics_panel(&self, ui: &mut egui::Ui) -> Vec<TrafficIntent> {
        let mut intents = Vec::new();

        ui.heading("Flow Metrics");
        ui.separator();

        if let Some((ref from, ref to)) = self.selected_flow {
            if let Some(flow) = self.flows.iter().find(|f| &f.from == from && &f.to == to) {
                let detail = prepare_flow_detail(flow);

                egui::Grid::new("traffic_metrics_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("From:");
                        ui.label(&detail.from);
                        ui.end_row();

                        ui.label("To:");
                        ui.label(&detail.to);
                        ui.end_row();

                        ui.label("Volume:");
                        ui.label(&detail.volume_label);
                        ui.end_row();

                        ui.label("Requests:");
                        ui.label(&detail.requests_label);
                        ui.end_row();

                        ui.label("Latency:");
                        ui.label(&detail.latency_label);
                        ui.end_row();

                        ui.label("Error Rate:");
                        ui.label(&detail.error_rate_label);
                        ui.end_row();
                    });

                ui.add_space(16.0);

                if ui.button("Close").clicked() {
                    intents.push(TrafficIntent::CloseDetails);
                }
            }
        } else {
            ui.label("Click on a flow to see metrics");
        }

        intents
    }

    fn update_flow_colors(&mut self) {
        for flow in &mut self.flows {
            flow.color = calculate_flow_color(&flow.metrics, self.color_scheme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Pure function tests ===

    #[test]
    fn calculate_flow_color_volume_zero() {
        let m = TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        };
        let c = calculate_flow_color(&m, ColorScheme::Volume);
        assert_eq!(c.r(), 0);
        assert_eq!(c.g(), 255);
    }

    #[test]
    fn calculate_flow_color_volume_max() {
        let m = TrafficMetrics {
            bytes_per_second: 200_000,
            ..Default::default()
        };
        let c = calculate_flow_color(&m, ColorScheme::Volume);
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 0);
    }

    #[test]
    fn calculate_flow_color_latency_fast() {
        let m = TrafficMetrics {
            avg_latency_ms: 0.0,
            ..Default::default()
        };
        let c = calculate_flow_color(&m, ColorScheme::Latency);
        assert_eq!(c.r(), 0);
    }

    #[test]
    fn calculate_flow_color_error_high() {
        let m = TrafficMetrics {
            error_rate: 0.5,
            ..Default::default()
        };
        let c = calculate_flow_color(&m, ColorScheme::ErrorRate);
        assert!(c.r() > 200);
    }

    #[test]
    fn calculate_flow_width_min() {
        let m = TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        };
        let w = calculate_flow_width(&m, 10000, 2.0, 40.0);
        assert!((w - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn calculate_flow_width_max() {
        let m = TrafficMetrics {
            bytes_per_second: 10000,
            ..Default::default()
        };
        let w = calculate_flow_width(&m, 10000, 2.0, 40.0);
        assert!((w - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn prepare_flow_detail_formats() {
        let flow = TrafficFlow {
            from: "alpha".to_string(),
            to: "beta".to_string(),
            metrics: TrafficMetrics {
                bytes_per_second: 5000,
                requests_per_second: 42.5,
                avg_latency_ms: 12.34,
                error_rate: 0.05,
            },
            color: Color32::RED,
        };
        let d = prepare_flow_detail(&flow);
        assert_eq!(d.from, "alpha");
        assert_eq!(d.to, "beta");
        assert_eq!(d.volume_label, "5000 B/s");
        assert_eq!(d.requests_label, "42.5 req/s");
        assert_eq!(d.latency_label, "12.34 ms");
        assert_eq!(d.error_rate_label, "5.00%");
    }

    #[test]
    fn apply_intents_set_color_scheme() {
        let mut view = TrafficView::new();
        view.apply_intents(&[TrafficIntent::SetColorScheme(ColorScheme::Latency)]);
        assert_eq!(view.color_scheme(), ColorScheme::Latency);
    }

    #[test]
    fn apply_intents_select_and_close() {
        let mut view = TrafficView::new();
        view.apply_intents(&[TrafficIntent::SelectFlow {
            from: "a".to_string(),
            to: "b".to_string(),
        }]);
        assert!(view.selected_flow().is_some());
        view.apply_intents(&[TrafficIntent::CloseDetails]);
        assert!(view.selected_flow().is_none());
    }

    #[test]
    fn apply_intents_toggle_metrics() {
        let mut view = TrafficView::new();
        assert!(view.show_metrics());
        view.apply_intents(&[TrafficIntent::ToggleMetrics]);
        assert!(!view.show_metrics());
    }

    #[test]
    fn apply_intents_clear() {
        let mut view = TrafficView::new();
        view.add_flow(TrafficFlow {
            from: "a".to_string(),
            to: "b".to_string(),
            metrics: TrafficMetrics::default(),
            color: Color32::GREEN,
        });
        view.apply_intents(&[TrafficIntent::Clear]);
        assert_eq!(view.flow_count(), 0);
    }

    // === Existing geometry tests ===

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
        assert!((ctrl1[0] - dx.signum().mul_add(offset, 400.0)).abs() < f32::EPSILON);
        assert!((ctrl2[0] - dx.signum().mul_add(-offset, 100.0)).abs() < f32::EPSILON);
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
