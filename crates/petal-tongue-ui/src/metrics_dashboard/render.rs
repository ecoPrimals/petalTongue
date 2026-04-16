// SPDX-License-Identifier: AGPL-3.0-or-later
//! egui rendering for [`super::state::MetricsDashboard`].

use crate::metrics_dashboard_helpers::{
    active_executions_color_rgb, cpu_history_avg_max, format_cpu_avg_display,
    format_cpu_max_display, format_cpu_percent, format_memory_used_total, prepare_metrics_display,
    sparkline_points_in_rect,
};
use egui::{Color32, ProgressBar, RichText, Stroke, Ui, Vec2};

use super::state::MetricsDashboard;

impl MetricsDashboard {
    /// Render the metrics dashboard
    pub fn render(&self, ui: &mut Ui) {
        ui.heading("📊 System Metrics");

        ui.separator();

        if let Some(data) = &self.data {
            let cpu: Vec<f64> = self
                .cpu_history
                .values()
                .into_iter()
                .map(f64::from)
                .collect();
            let mem: Vec<f64> = self
                .memory_history
                .values()
                .into_iter()
                .map(f64::from)
                .collect();
            let state = prepare_metrics_display(data, &cpu, &mem);

            Self::render_cpu_section(ui, &state);
            ui.add_space(12.0);
            Self::render_memory_section(ui, &state);
            ui.add_space(12.0);
            Self::render_system_info(ui, &state);
            ui.add_space(12.0);
            Self::render_neural_api_info(ui, &state);
        } else {
            ui.label(
                RichText::new("No metrics data available").color(Color32::from_rgb(156, 163, 175)),
            );
            ui.label("Waiting for Neural API...");
        }
    }

    /// Render CPU usage section with sparkline (thin egui wrapper)
    fn render_cpu_section(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("CPU Usage").strong().size(14.0));

            let (r, g, b) = state.cpu_color;
            let color = Color32::from_rgb(r, g, b);

            let progress = (state.cpu_percent / 100.0) as f32;
            ui.add(
                ProgressBar::new(progress)
                    .fill(color)
                    .text(format_cpu_percent(state.cpu_percent)),
            );

            let cpu_values: Vec<f32> = state.cpu_history.iter().map(|&v| v as f32).collect();
            if cpu_values.len() >= 3 {
                ui.add_space(4.0);
                Self::render_sparkline(ui, &cpu_values, "CPU History", color);
            }

            if !state.cpu_history.is_empty() {
                let (avg, max) = cpu_history_avg_max(&state.cpu_history);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format_cpu_avg_display(avg))
                            .color(Color32::from_rgb(156, 163, 175)),
                    );
                    ui.label(
                        RichText::new(format_cpu_max_display(max))
                            .color(Color32::from_rgb(156, 163, 175)),
                    );
                });
            }
        });
    }

    /// Render memory usage section with sparkline (thin egui wrapper)
    fn render_memory_section(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("Memory Usage").strong().size(14.0));

            let (r, g, b) = state.memory_color;
            let color = Color32::from_rgb(r, g, b);

            let progress = (state.memory_percent / 100.0) as f32;
            ui.add(
                ProgressBar::new(progress)
                    .fill(color)
                    .text(format_cpu_percent(state.memory_percent)),
            );

            ui.label(format_memory_used_total(
                state.memory_used_mb,
                state.memory_total_mb,
            ));

            if !state.memory_history.is_empty() {
                ui.add_space(4.0);
                let mem_values: Vec<f32> = state.memory_history.iter().map(|&v| v as f32).collect();
                Self::render_sparkline(ui, &mem_values, "Memory History", color);
            }
        });
    }

    /// Render system information (thin egui wrapper)
    fn render_system_info(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("System Information").strong().size(14.0));

            ui.horizontal(|ui| {
                ui.label("⏱️ Uptime:");
                ui.label(RichText::new(&state.uptime_text).color(Color32::from_rgb(59, 130, 246)));
            });
        });
    }

    /// Render Neural API information (thin egui wrapper)
    fn render_neural_api_info(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("Neural API Status").strong().size(14.0));

            ui.horizontal(|ui| {
                ui.label("🧬 Family:");
                ui.label(RichText::new(&state.family_id).color(Color32::from_rgb(168, 85, 247)));
            });

            ui.horizontal(|ui| {
                ui.label("🌸 Active Primals:");
                ui.label(
                    RichText::new(state.active_primals.to_string())
                        .strong()
                        .color(Color32::from_rgb(34, 197, 94)),
                );
            });

            ui.horizontal(|ui| {
                ui.label("📊 Available Graphs:");
                ui.label(state.graphs_available.to_string());
            });

            ui.horizontal(|ui| {
                ui.label("⚡ Active Executions:");
                let rgb = active_executions_color_rgb(state.active_executions);
                let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                ui.label(RichText::new(state.active_executions.to_string()).color(color));
            });
        });
    }

    /// Render a sparkline chart (thin egui wrapper)
    fn render_sparkline(ui: &mut Ui, values: &[f32], label: &str, color: Color32) {
        if values.len() < 2 {
            return;
        }

        let desired_size = Vec2::new(ui.available_width(), 40.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let points: Vec<egui::Pos2> = sparkline_points_in_rect(
                values,
                rect.left(),
                rect.top(),
                rect.width(),
                rect.height(),
            )
            .into_iter()
            .map(|(x, y)| egui::pos2(x, y))
            .collect();

            if points.len() >= 2 {
                let stroke = Stroke::new(2.0, color);
                ui.painter().add(egui::Shape::line(points.clone(), stroke));

                if points.len() >= 2 {
                    let mut area_points = points;
                    area_points.push(egui::pos2(rect.right(), rect.bottom()));
                    area_points.push(egui::pos2(rect.left(), rect.bottom()));

                    let fill_color =
                        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30);
                    ui.painter().add(egui::Shape::convex_polygon(
                        area_points,
                        fill_color,
                        Stroke::NONE,
                    ));
                }
            }

            ui.painter().text(
                egui::pos2(rect.left() + 4.0, rect.top() + 4.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(156, 163, 175),
            );
        }
    }
}
