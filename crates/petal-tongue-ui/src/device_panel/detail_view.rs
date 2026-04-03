// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-device card rendering (row content, drag affordances).

use super::DevicePanel;
use crate::biomeos_integration::{Device, DeviceStatus, DeviceType};
use egui::{Color32, RichText, Ui};

/// Get icon for device type
pub const fn device_icon(device_type: DeviceType) -> &'static str {
    match device_type {
        DeviceType::GPU => "🎮",
        DeviceType::CPU => "🧠",
        DeviceType::Storage => "💾",
        DeviceType::Network => "🌐",
        DeviceType::Memory => "🔲",
        DeviceType::Other => "❓",
    }
}

pub(super) fn render_device_card(panel: &mut DevicePanel, ui: &mut Ui, device: &Device) {
    let is_selected = panel.selected.as_ref() == Some(&device.id);

    let response = egui::Frame::none()
        .fill(if is_selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().faint_bg_color
        })
        .inner_margin(egui::Margin::same(8.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(device_icon(device.device_type)).size(20.0));

                ui.vertical(|ui| {
                    ui.label(RichText::new(&device.name).strong());

                    ui.horizontal(|ui| {
                        let (color, text) = match device.status {
                            DeviceStatus::Online => (Color32::GREEN, "● Online"),
                            DeviceStatus::Offline => (Color32::GRAY, "● Offline"),
                            DeviceStatus::Busy => (Color32::YELLOW, "● Busy"),
                            DeviceStatus::Error => (Color32::RED, "● Error"),
                        };
                        ui.colored_label(color, text);

                        if let Some(primal_id) = &device.assigned_to {
                            ui.separator();
                            ui.label(format!("→ {primal_id}"));
                        }
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let usage = device.resource_usage;
                    let bar_color = DevicePanel::usage_bar_color(usage);

                    ui.add(
                        egui::ProgressBar::new(usage as f32)
                            .fill(bar_color)
                            .show_percentage(),
                    );
                });
            });
        })
        .response;

    if response.clicked() {
        panel.selected = if is_selected {
            None
        } else {
            Some(device.id.clone())
        };
    }

    let is_dragging = response.dragged();
    if is_dragging {
        ui.memory_mut(|mem| {
            mem.data
                .insert_temp(egui::Id::new("dragged_device"), device.id.clone());
        });
    }

    if response.hovered() {
        response.on_hover_text("Drag to assign to a primal");
    }

    ui.add_space(4.0);
}
