// SPDX-License-Identifier: AGPL-3.0-or-later
//! Display formatting utilities for primal panel (load bars, health indicators).

use crate::biomeos_integration::Health;
use egui::Color32;

/// Returns RGB color for load bar based on load value (green/yellow/red).
#[must_use]
pub fn load_bar_color_rgb(load: f64) -> [u8; 3] {
    if load > 0.9 {
        [255, 0, 0]
    } else if load > 0.7 {
        [255, 255, 0]
    } else {
        [0, 255, 0]
    }
}

/// Returns (display text, RGB) for health status.
#[must_use]
pub const fn health_display_data(health: &Health) -> (&'static str, [u8; 3]) {
    match health {
        Health::Healthy => ("● Healthy", [0, 255, 0]),
        Health::Degraded => ("● Degraded", [255, 255, 0]),
        Health::Offline => ("● Offline", [255, 0, 0]),
    }
}

/// Returns egui Color32 for load bar based on load value.
#[must_use]
pub fn load_bar_color(load: f64) -> Color32 {
    let rgb = load_bar_color_rgb(load);
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}
