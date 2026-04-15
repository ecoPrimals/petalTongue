// SPDX-License-Identifier: AGPL-3.0-or-later

//! Top toolbar: color scheme, metrics toggle, clear.

use egui::Ui;

use crate::traffic_view::types::{ColorScheme, TrafficIntent};

pub fn render(
    ui: &mut Ui,
    color_scheme: ColorScheme,
    show_metrics: bool,
    flow_count: usize,
) -> Vec<TrafficIntent> {
    let mut intents = Vec::new();

    ui.horizontal(|ui| {
        ui.heading("🌊 Traffic View");
        ui.separator();

        ui.label("Color by:");
        if ui
            .selectable_label(color_scheme == ColorScheme::Volume, "Volume")
            .clicked()
        {
            intents.push(TrafficIntent::SetColorScheme(ColorScheme::Volume));
        }
        if ui
            .selectable_label(color_scheme == ColorScheme::Latency, "Latency")
            .clicked()
        {
            intents.push(TrafficIntent::SetColorScheme(ColorScheme::Latency));
        }
        if ui
            .selectable_label(color_scheme == ColorScheme::ErrorRate, "Errors")
            .clicked()
        {
            intents.push(TrafficIntent::SetColorScheme(ColorScheme::ErrorRate));
        }

        ui.separator();

        let mut metrics_val = show_metrics;
        if ui.checkbox(&mut metrics_val, "Show Metrics").clicked() {
            intents.push(TrafficIntent::ToggleMetrics);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Clear").clicked() {
                intents.push(TrafficIntent::Clear);
            }
            ui.label(format!("Flows: {flow_count}"));
        });
    });

    intents
}
