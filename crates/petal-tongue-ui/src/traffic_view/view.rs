// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traffic View - main view logic and rendering
//!
//! Architecture: headless-first. Pure geometry/color functions live in `helpers`.
//! The render method returns `Vec<TrafficIntent>` instead of mutating `self` directly.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::collections::HashMap;

use super::helpers::{
    bezier_control_points, calculate_flow_color, calculate_flow_width, prepare_flow_detail,
    primal_lane_layout,
};
use super::types::{ColorScheme, TrafficFlow, TrafficIntent, TrafficMetrics};

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

    #[cfg(test)]
    #[must_use]
    pub fn max_volume(&self) -> u64 {
        self.max_volume_impl()
    }

    fn max_volume_impl(&self) -> u64 {
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

        let max_vol = self.max_volume_impl();

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

                Self::draw_bezier_flow(
                    &painter,
                    start,
                    ctrl1,
                    ctrl2,
                    end,
                    width,
                    Self::to_color32(flow.color),
                );

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

    fn to_color32(rgba: [u8; 4]) -> Color32 {
        Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
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
            ui.label("Select a flow to perceive metrics");
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod view_tests {
    use super::*;
    use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo};

    #[test]
    fn test_render_with_metrics_and_selected_flow() {
        let mut view = TrafficView::new();
        view.add_flow(TrafficFlow {
            from: "a".to_string(),
            to: "b".to_string(),
            metrics: TrafficMetrics {
                bytes_per_second: 5000,
                requests_per_second: 10.0,
                avg_latency_ms: 5.0,
                error_rate: 0.01,
            },
            color: [0, 255, 0, 255],
        });
        view.apply_intents(&[TrafficIntent::SelectFlow {
            from: "a".to_string(),
            to: "b".to_string(),
        }]);
        assert!(view.show_metrics());
        assert!(view.selected_flow().is_some());

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let intents = view.render(ui);
                view.apply_intents(&intents);
            });
        });
    }

    #[test]
    fn test_render_with_flows_no_primals_derives_ids() {
        let mut view = TrafficView::new();
        view.add_flow(TrafficFlow {
            from: "primal_x".to_string(),
            to: "primal_y".to_string(),
            metrics: TrafficMetrics::default(),
            color: [0, 255, 0, 255],
        });
        assert_eq!(view.primal_count(), 0);
        assert_eq!(view.flow_count(), 1);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let intents = view.render(ui);
                view.apply_intents(&intents);
            });
        });
    }

    #[test]
    fn test_render_empty_flows_shows_message() {
        let mut view = TrafficView::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let intents = view.render(ui);
                assert!(intents.is_empty());
            });
        });
    }

    #[test]
    fn test_apply_intent_close_details() {
        let mut view = TrafficView::new();
        view.add_flow(TrafficFlow {
            from: "a".to_string(),
            to: "b".to_string(),
            metrics: TrafficMetrics::default(),
            color: [0, 255, 0, 255],
        });
        view.apply_intents(&[TrafficIntent::SelectFlow {
            from: "a".to_string(),
            to: "b".to_string(),
        }]);
        view.apply_intents(&[TrafficIntent::CloseDetails]);
        assert!(view.selected_flow().is_none());
    }

    #[test]
    fn test_render_toggle_metrics_intent() {
        let mut view = TrafficView::new();
        view.add_flow(TrafficFlow {
            from: "a".to_string(),
            to: "b".to_string(),
            metrics: TrafficMetrics::default(),
            color: [0, 255, 0, 255],
        });
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut intents = view.render(ui);
                intents.push(TrafficIntent::ToggleMetrics);
                view.apply_intents(&intents);
            });
        });
        assert!(!view.show_metrics());
    }

    #[test]
    fn test_render_with_primals_and_flows() {
        let mut view = TrafficView::new();
        view.set_primals(vec![
            PrimalInfo::new(
                PrimalId::from("alpha"),
                "Alpha",
                "Compute",
                "http://localhost",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
            PrimalInfo::new(
                PrimalId::from("beta"),
                "Beta",
                "Storage",
                "http://localhost",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
        ]);
        view.add_flow(TrafficFlow {
            from: "alpha".to_string(),
            to: "beta".to_string(),
            metrics: TrafficMetrics {
                bytes_per_second: 10000,
                ..Default::default()
            },
            color: [0, 255, 0, 255],
        });

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let intents = view.render(ui);
                view.apply_intents(&intents);
            });
        });
    }

    #[test]
    fn test_max_volume_single_flow() {
        let mut view = TrafficView::new();
        view.add_flow(TrafficFlow {
            from: "a".to_string(),
            to: "b".to_string(),
            metrics: TrafficMetrics {
                bytes_per_second: 42,
                ..Default::default()
            },
            color: [0, 255, 0, 255],
        });
        assert_eq!(view.max_volume(), 42);
    }
}
