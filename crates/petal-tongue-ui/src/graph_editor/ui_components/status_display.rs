// SPDX-License-Identifier: AGPL-3.0-or-later
//! Status display widget — node execution status, progress, resources, errors.

use egui::{Color32, RichText, Ui};

use super::display::{
    error_header_text, error_recoverable_color_rgb, error_recoverable_display, node_status_display,
    progress_percent_text, resource_usage_display,
};
use crate::graph_editor::streaming::{ErrorInfo, NodeStatus, ResourceUsage};

/// Status display widget - Shows node execution status
///
/// Displays real-time status updates for graph nodes.
pub struct StatusDisplay;

impl StatusDisplay {
    /// Render node status badge
    pub fn show_node_status(ui: &mut Ui, node_id: &str, status: &NodeStatus) {
        let (icon, color_rgb, text) = node_status_display(status);
        let color = Color32::from_rgb(color_rgb[0], color_rgb[1], color_rgb[2]);

        ui.horizontal(|ui| {
            // Icon
            ui.label(RichText::new(icon).size(16.0));

            // Node ID
            ui.label(RichText::new(node_id).strong().color(color));

            // Status text
            ui.label(RichText::new(text).color(color));
        });

        // Show error details if failed
        if let NodeStatus::Failed { error } = status {
            ui.indent("error_details", |ui| {
                ui.label(RichText::new(error).color(Color32::RED).italics());
            });
        }
    }

    /// Render progress bar
    pub fn show_progress(ui: &mut Ui, progress: f32, message: &str) {
        ui.vertical(|ui| {
            let progress_bar = egui::ProgressBar::new(progress)
                .text(progress_percent_text(progress))
                .animate(true);

            ui.add(progress_bar);

            // Progress message
            if !message.is_empty() {
                ui.label(RichText::new(message).italics().size(12.0));
            }
        });
    }

    /// Render resource usage
    pub fn show_resources(ui: &mut Ui, resources: &ResourceUsage) {
        let (cpu_str, mem_str, disk_str, net_str) = resource_usage_display(resources);
        ui.vertical(|ui| {
            ui.heading(RichText::new("Resources").size(14.0));

            ui.horizontal(|ui| {
                ui.label("CPU:");
                ui.label(RichText::new(cpu_str).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Memory:");
                ui.label(RichText::new(mem_str).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Disk I/O:");
                ui.label(RichText::new(disk_str).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Network:");
                ui.label(RichText::new(net_str).strong());
            });
        });
    }

    /// Render error information
    pub fn show_error(ui: &mut Ui, error: &ErrorInfo) {
        let header = error_header_text(error);
        let (recoverable_msg, suggestion_msg) = error_recoverable_display(error);
        let rgb = error_recoverable_color_rgb(error.recoverable);
        let recoverable_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
        ui.vertical(|ui| {
            ui.colored_label(Color32::RED, RichText::new(header).strong());

            ui.label(&error.message);

            if let Some(details) = &error.details {
                ui.indent("error_details", |ui| {
                    ui.label(RichText::new(details).italics().size(12.0));
                });
            }

            ui.colored_label(recoverable_color, recoverable_msg);
            if let Some(suggestion) = suggestion_msg {
                ui.colored_label(Color32::YELLOW, suggestion);
            }
        });
    }
}
