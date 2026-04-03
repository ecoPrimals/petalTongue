// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filter bar, search-driven list, and aggregate stats.

use super::detail_view;
use super::{DeviceFilter, DevicePanel};
use crate::biomeos_integration::Device;
use egui::{Color32, Ui};

pub(super) fn render_filter_bar(panel: &mut DevicePanel, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Filter:");

        if ui
            .selectable_label(panel.filter == DeviceFilter::All, "All")
            .clicked()
        {
            panel.filter = DeviceFilter::All;
        }

        if ui
            .selectable_label(panel.filter == DeviceFilter::Available, "Available")
            .clicked()
        {
            panel.filter = DeviceFilter::Available;
        }

        if ui
            .selectable_label(panel.filter == DeviceFilter::Assigned, "Assigned")
            .clicked()
        {
            panel.filter = DeviceFilter::Assigned;
        }
    });
}

pub(super) fn render_stats(panel: &DevicePanel, ui: &mut Ui) {
    let (total, online, assigned) = DevicePanel::compute_device_stats(&panel.devices);

    ui.horizontal(|ui| {
        ui.label(format!("Total: {total}"));
        ui.separator();
        ui.colored_label(Color32::GREEN, format!("Online: {online}"));
        ui.separator();
        ui.label(format!("Assigned: {assigned}"));
    });
}

pub(super) fn filtered_devices(panel: &DevicePanel) -> Vec<&Device> {
    panel
        .devices
        .iter()
        .filter(|device| {
            let filter_match = match panel.filter {
                DeviceFilter::All => true,
                DeviceFilter::Available => device.assigned_to.is_none(),
                DeviceFilter::Assigned => device.assigned_to.is_some(),
            };

            let search_match = if panel.search_query.is_empty() {
                true
            } else {
                let query = panel.search_query.to_lowercase();
                device.name.to_lowercase().contains(&query)
                    || device.id.to_lowercase().contains(&query)
            };

            filter_match && search_match
        })
        .collect()
}

pub(super) fn render_scrollable_list(panel: &mut DevicePanel, ui: &mut Ui) {
    egui::ScrollArea::vertical()
        .id_salt("device_list")
        .show(ui, |ui| {
            let filtered_devices: Vec<Device> =
                filtered_devices(panel).into_iter().cloned().collect();

            if filtered_devices.is_empty() {
                ui.colored_label(Color32::GRAY, "No devices found");
            } else {
                for device in &filtered_devices {
                    detail_view::render_device_card(panel, ui, device);
                }
            }
        });
}
