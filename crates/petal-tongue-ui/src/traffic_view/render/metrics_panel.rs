// SPDX-License-Identifier: AGPL-3.0-or-later

//! Detail panel for the selected flow’s metrics.

use egui::Ui;

use crate::traffic_view::helpers::prepare_flow_detail;
use crate::traffic_view::types::TrafficIntent;
use crate::traffic_view::view::TrafficView;

pub fn render_metrics_panel(view: &TrafficView, ui: &mut Ui) -> Vec<TrafficIntent> {
    let mut intents = Vec::new();

    ui.heading("Flow Metrics");
    ui.separator();

    if let Some((from, to)) = view.selected_flow() {
        if let Some(flow) = view.flows().iter().find(|f| f.from == *from && f.to == *to) {
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
