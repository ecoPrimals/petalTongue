// SPDX-License-Identifier: AGPL-3.0-or-later
//! Headless-testable display structs for CPU and memory metrics (values pre-computed for egui).

use egui::Color32;

/// Pre-computed CPU display data. Produced by `prepare_cpu_display()`,
/// consumed by `render_cpu()`. All formatting and threshold logic lives
/// in the producer so tests can validate without a UI context.
#[derive(Debug, Clone)]
pub struct CpuDisplayState {
    pub usage: f32,
    pub bar_fraction: f32,
    pub bar_color: Color32,
    pub label: String,
    pub core_count: usize,
    pub history_label: Option<String>,
}

/// Pre-computed memory display data.
#[derive(Debug, Clone)]
pub struct MemoryDisplayState {
    pub percent: f64,
    pub bar_fraction: f32,
    pub bar_color: Color32,
    pub label: String,
    pub used_gb_label: String,
    pub history_label: Option<String>,
}
