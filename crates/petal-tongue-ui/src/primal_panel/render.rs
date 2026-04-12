// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rendering logic for primal panel UI components.

use super::PrimalPanel;
use super::display;
use super::filter::PrimalFilter;
use super::stats;
use crate::biomeos_integration::Primal;
use crate::ui_events::{UIEvent, UIEventHandler};
use egui::{Color32, RichText, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Renders the filter bar (All/Healthy/Degraded/Error).
pub fn render_filter_bar(panel: &mut PrimalPanel, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Filter:");

        if ui
            .selectable_label(panel.filter == PrimalFilter::All, "All")
            .clicked()
        {
            panel.filter = PrimalFilter::All;
        }

        if ui
            .selectable_label(panel.filter == PrimalFilter::Healthy, "Healthy")
            .clicked()
        {
            panel.filter = PrimalFilter::Healthy;
        }

        if ui
            .selectable_label(panel.filter == PrimalFilter::Degraded, "Degraded")
            .clicked()
        {
            panel.filter = PrimalFilter::Degraded;
        }

        if ui
            .selectable_label(panel.filter == PrimalFilter::Degraded, "Error")
            .clicked()
        {
            panel.filter = PrimalFilter::Degraded;
        }
    });
}

/// Renders the stats bar (total, healthy, degraded, error counts).
pub fn render_stats(panel: &PrimalPanel, ui: &mut Ui) {
    let (total, healthy, degraded, error) = stats::compute_primal_stats(&panel.primals);

    ui.horizontal(|ui| {
        ui.label(format!("Total: {total}"));
        ui.separator();
        ui.colored_label(Color32::GREEN, format!("Healthy: {healthy}"));
        ui.separator();
        ui.colored_label(Color32::YELLOW, format!("Degraded: {degraded}"));
        ui.separator();
        ui.colored_label(Color32::RED, format!("Error: {error}"));
    });
}

/// Renders an individual primal card with selection and drop zone.
pub fn render_primal_card(
    panel: &mut PrimalPanel,
    ui: &mut Ui,
    primal: &Primal,
    event_handler: Arc<RwLock<UIEventHandler>>,
) {
    let is_selected = panel.selected.as_ref() == Some(&primal.id);

    let response = egui::Frame::none()
        .fill(if is_selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().faint_bg_color
        })
        .inner_margin(egui::Margin::same(8.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Primal header
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&primal.name).strong().size(16.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (text, rgb) = display::health_display_data(&primal.health);
                        ui.colored_label(Color32::from_rgb(rgb[0], rgb[1], rgb[2]), text);
                    });
                });

                ui.add_space(4.0);

                // Primal stats
                ui.horizontal(|ui| {
                    ui.label("Load:");
                    let load = primal.load;
                    let bar_color = display::load_bar_color(load);

                    ui.add(
                        egui::ProgressBar::new(load as f32)
                            .fill(bar_color)
                            .show_percentage(),
                    );
                });

                ui.add_space(4.0);

                // Capabilities
                ui.horizontal(|ui| {
                    ui.label("Capabilities:");
                    ui.label(primal.capabilities.len().to_string());
                });

                ui.add_space(4.0);

                // Assigned devices
                ui.horizontal(|ui| {
                    ui.label("Devices:");
                    if primal.assigned_devices.is_empty() {
                        ui.colored_label(Color32::GRAY, "None");
                    } else {
                        ui.label(primal.assigned_devices.len().to_string());
                    }
                });
            });
        })
        .response;

    // Selection
    if response.clicked() {
        panel.selected = if is_selected {
            None
        } else {
            Some(primal.id.clone())
        };
    }

    // Drop zone for device assignment
    let is_dragging_device = ui.memory(|mem| {
        mem.data
            .get_temp::<String>(egui::Id::new("dragged_device"))
            .is_some()
    });

    if is_dragging_device {
        // Check if hovering over this primal
        if response.hovered() {
            // Highlight as drop zone
            let highlight_rect = response.rect.expand(2.0);
            ui.painter()
                .rect_stroke(highlight_rect, 4.0, (2.0, Color32::LIGHT_BLUE));

            // Show drop hint
            response.on_hover_text(format!("Drop device here to assign to {}", primal.name));

            // Handle drop
            if !ui.input(|i| i.pointer.is_decidedly_dragging()) {
                // Drag ended, check if we can get the device ID
                if let Some(device_id) = ui.memory_mut(|mem| {
                    mem.data
                        .remove_temp::<String>(egui::Id::new("dragged_device"))
                }) {
                    info!("🎯 Device {} dropped on primal {}", device_id, primal.id);

                    // Send assignment event
                    let primal_id = primal.id.clone();
                    tokio::spawn(async move {
                        event_handler
                            .write()
                            .await
                            .handle_event(UIEvent::DeviceAssigned(device_id, primal_id))
                            .await;
                    });
                }
            }
        }
    }

    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::{Health, Primal};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn make_panel() -> PrimalPanel {
        PrimalPanel::new(Arc::new(RwLock::new(UIEventHandler::new())))
    }

    fn make_primal(id: &str, health: Health, load: f64, devices: Vec<String>) -> Primal {
        Primal {
            id: id.to_string(),
            name: id.to_string(),
            health,
            capabilities: vec!["compute".to_string()],
            load,
            assigned_devices: devices,
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn render_filter_bar_headless() {
        let mut panel = make_panel();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_filter_bar(&mut panel, ui);
            });
        });
    }

    #[test]
    fn render_filter_bar_with_different_filters() {
        let mut panel = make_panel();
        panel.filter = PrimalFilter::Healthy;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_filter_bar(&mut panel, ui);
            });
        });
    }

    #[test]
    fn render_stats_headless_empty() {
        let panel = make_panel();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_stats(&panel, ui);
            });
        });
    }

    #[test]
    fn render_stats_headless_with_primals() {
        let mut panel = make_panel();
        panel.primals = vec![
            make_primal("p1", Health::Healthy, 0.5, vec![]),
            make_primal("p2", Health::Degraded, 0.8, vec![]),
            make_primal("p3", Health::Offline, 0.0, vec![]),
        ];
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_stats(&panel, ui);
            });
        });
    }

    #[test]
    fn render_primal_card_headless() {
        let mut panel = make_panel();
        let primal = make_primal("p1", Health::Healthy, 0.5, vec![]);
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_primal_card(&mut panel, ui, &primal, event_handler.clone());
            });
        });
    }

    #[test]
    fn render_primal_card_with_assigned_devices() {
        let mut panel = make_panel();
        let primal = make_primal("p1", Health::Healthy, 0.5, vec!["dev-1".to_string()]);
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_primal_card(&mut panel, ui, &primal, event_handler.clone());
            });
        });
    }

    #[test]
    fn render_primal_card_selected() {
        let mut panel = make_panel();
        panel.selected = Some("p1".to_string());
        let primal = make_primal("p1", Health::Healthy, 0.5, vec![]);
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_primal_card(&mut panel, ui, &primal, event_handler.clone());
            });
        });
    }
}
