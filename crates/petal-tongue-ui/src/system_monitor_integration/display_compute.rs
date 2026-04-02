// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure metric → display transformations (no `&self`, no egui widgets).

#![expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "system monitor metrics use numeric casts for display; precision loss acceptable"
)]

use super::display_state::{CpuDisplayState, MemoryDisplayState};
use egui::Color32;
use std::collections::VecDeque;

/// Map a metric value to a traffic-light color.
///
/// Returns red above `high`, amber above `mid`, and the provided `normal` color otherwise.
#[must_use]
pub fn threshold_color(value: f64, high: f64, mid: f64, normal: Color32) -> Color32 {
    if value > high {
        Color32::from_rgb(200, 50, 50)
    } else if value > mid {
        Color32::from_rgb(200, 150, 50)
    } else {
        normal
    }
}

/// Compute sparkline points in normalised coordinates.
///
/// Returns `Vec<[f32; 2]>` where x is in `[0, width]` and y is in `[0, height]`
/// (0 = top, height = bottom). The caller maps these into screen-space.
#[must_use]
pub fn compute_sparkline_points(
    data: &VecDeque<f32>,
    width: f32,
    height: f32,
    max_value: f32,
) -> Vec<[f32; 2]> {
    if data.len() < 2 {
        return Vec::new();
    }
    let len_minus_one = (data.len() - 1).max(1) as f32;
    data.iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = (i as f32 / len_minus_one) * width;
            let y = (1.0 - (value / max_value).min(1.0)) * height;
            [x, y]
        })
        .collect()
}

/// Format bytes as GB with one decimal place.
#[must_use]
pub fn format_gb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1_073_741_824.0)
}

/// Prepare CPU display state from current stats (pure computation).
#[must_use]
pub fn prepare_cpu_display(
    cpu_usage: f32,
    core_count: usize,
    history_len: usize,
) -> CpuDisplayState {
    CpuDisplayState {
        usage: cpu_usage,
        bar_fraction: cpu_usage / 100.0,
        bar_color: threshold_color(
            f64::from(cpu_usage),
            90.0,
            70.0,
            Color32::from_rgb(50, 150, 200),
        ),
        label: format!("{cpu_usage:.1}%"),
        core_count,
        history_label: if history_len > 0 {
            Some(format!("History ({history_len} samples) [LIVE DATA]"))
        } else {
            None
        },
    }
}

/// Prepare memory display state from current stats (pure computation).
#[must_use]
pub fn prepare_memory_display(
    used_bytes: u64,
    total_bytes: u64,
    history_len: usize,
) -> MemoryDisplayState {
    let percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };
    MemoryDisplayState {
        percent,
        bar_fraction: percent as f32 / 100.0,
        bar_color: threshold_color(percent, 90.0, 70.0, Color32::from_rgb(50, 200, 150)),
        label: format!("{percent:.1}%"),
        used_gb_label: format!(
            "Used: {} / {} GB",
            format_gb(used_bytes),
            format_gb(total_bytes)
        ),
        history_label: if history_len > 0 {
            Some(format!("History ({history_len} samples) [LIVE DATA]"))
        } else {
            None
        },
    }
}
