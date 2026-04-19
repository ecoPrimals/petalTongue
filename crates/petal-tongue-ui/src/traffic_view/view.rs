// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traffic View - main view logic and rendering
//!
//! Architecture: headless-first. Pure geometry/color functions live in `helpers`.
//! The render method returns `Vec<TrafficIntent>` instead of mutating `self` directly.

use egui::Vec2;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use std::collections::HashMap;

use super::helpers::calculate_flow_color;
use super::render::{render_metrics_panel, render_toolbar, render_traffic_diagram};
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

    #[must_use]
    pub(crate) fn flows(&self) -> &[TrafficFlow] {
        &self.flows
    }

    #[must_use]
    pub(crate) const fn primals_map(&self) -> &HashMap<String, PrimalInfo> {
        &self.primals
    }

    #[must_use]
    pub(crate) const fn min_flow_width(&self) -> f32 {
        self.min_flow_width
    }

    #[must_use]
    pub(crate) const fn max_flow_width(&self) -> f32 {
        self.max_flow_width
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

    pub(crate) fn max_volume_impl(&self) -> u64 {
        self.flows
            .iter()
            .map(|f| f.metrics.bytes_per_second)
            .max()
            .unwrap_or(1)
    }

    /// Render the traffic view. Returns intents for the caller to apply.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<TrafficIntent> {
        let mut intents = Vec::new();

        intents.extend(render_toolbar(
            ui,
            self.color_scheme,
            self.show_metrics,
            self.flows.len(),
        ));

        ui.separator();

        if self.show_metrics && self.selected_flow.is_some() {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width() * 0.7, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        intents.extend(render_traffic_diagram(self, ui));
                    },
                );

                ui.separator();

                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        intents.extend(render_metrics_panel(self, ui));
                    },
                );
            });
        } else {
            intents.extend(render_traffic_diagram(self, ui));
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
