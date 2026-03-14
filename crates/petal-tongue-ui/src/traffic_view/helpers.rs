// SPDX-License-Identifier: AGPL-3.0-only
//! Traffic View - Pure geometry and color functions (fully testable, no egui context)

use egui::Color32;

use super::types::{ColorScheme, TrafficFlow, TrafficMetrics};

/// Bezier control points for flow curve from (from_x, from_y) to (to_x, to_y).
/// Returns (ctrl1, ctrl2) as [x, y] arrays.
#[must_use]
pub fn bezier_control_points(
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
) -> ([f32; 2], [f32; 2]) {
    let dx = to_x - from_x;
    let control_offset = dx.abs() * 0.3;
    let ctrl1 = [from_x + control_offset * dx.signum(), from_y];
    let ctrl2 = [to_x - control_offset * dx.signum(), to_y];
    (ctrl1, ctrl2)
}

/// Compute lane positions for primals in the traffic view.
/// Returns for each index: (y, left_center_x, right_center_x).
#[must_use]
pub fn primal_lane_layout(
    primal_count: usize,
    rect_min_x: f32,
    rect_min_y: f32,
    rect_max_x: f32,
    rect_max_y: f32,
    margin: f32,
    node_width: f32,
) -> Vec<(f32, f32, f32)> {
    if primal_count == 0 {
        return Vec::new();
    }
    let node_height = 2.0f32.mul_add(-margin, rect_max_y - rect_min_y) / primal_count as f32;
    let left_center_x = rect_min_x + margin + node_width / 2.0;
    let right_center_x = rect_max_x - margin - node_width / 2.0;
    (0..primal_count)
        .map(|i| {
            let y = node_height.mul_add(i as f32 + 0.5, rect_min_y + margin);
            (y, left_center_x, right_center_x)
        })
        .collect()
}

/// Calculate flow color based on metrics and scheme (pure function).
#[must_use]
pub fn calculate_flow_color(metrics: &TrafficMetrics, scheme: ColorScheme) -> Color32 {
    match scheme {
        ColorScheme::Volume => {
            let normalized = (metrics.bytes_per_second as f32 / 100_000.0).min(1.0);
            Color32::from_rgb(
                (255.0 * normalized) as u8,
                (255.0 * (1.0 - normalized)) as u8,
                32,
            )
        }
        ColorScheme::Latency => {
            let normalized = (metrics.avg_latency_ms as f32 / 100.0).min(1.0);
            Color32::from_rgb(
                (255.0 * normalized) as u8,
                (255.0 * (1.0 - normalized)) as u8,
                64,
            )
        }
        ColorScheme::ErrorRate => {
            let normalized = (metrics.error_rate as f32 * 10.0).min(1.0);
            Color32::from_rgb(
                (255.0 * normalized) as u8,
                (255.0 * (1.0 - normalized) * 0.8) as u8,
                96,
            )
        }
    }
}

/// Calculate flow width based on volume relative to max (pure function).
#[must_use]
pub fn calculate_flow_width(
    metrics: &TrafficMetrics,
    max_volume: u64,
    min_width: f32,
    max_width: f32,
) -> f32 {
    let max_vol = max_volume.max(1);
    let normalized = metrics.bytes_per_second as f32 / max_vol as f32;
    (max_width - min_width).mul_add(normalized, min_width)
}

/// Build a `FlowDetailDisplay` from a flow (pure, no egui).
#[must_use]
pub fn prepare_flow_detail(flow: &TrafficFlow) -> super::types::FlowDetailDisplay {
    super::types::FlowDetailDisplay {
        from: flow.from.clone(),
        to: flow.to.clone(),
        volume_label: format!("{} B/s", flow.metrics.bytes_per_second),
        requests_label: format!("{:.1} req/s", flow.metrics.requests_per_second),
        latency_label: format!("{:.2} ms", flow.metrics.avg_latency_ms),
        error_rate_label: format!("{:.2}%", flow.metrics.error_rate * 100.0),
    }
}
