// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure layout/adaptation logic (testable, no egui).

use petal_tongue_core::{DeviceType, PrimalHealthStatus, PrimalInfo};

/// Device type to use for rendering (Unknown defaults to Desktop)
#[must_use]
pub const fn effective_device_for_rendering(device_type: DeviceType) -> DeviceType {
    match device_type {
        DeviceType::Unknown => DeviceType::Desktop,
        other => other,
    }
}

/// CLI-style status text for primal health
#[must_use]
pub const fn format_cli_primal_status(health: PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "OK",
        PrimalHealthStatus::Warning => "WARN",
        PrimalHealthStatus::Critical => "CRIT",
        PrimalHealthStatus::Unknown => "UNKN",
    }
}

/// Count primals with healthy status
#[must_use]
pub fn count_healthy_primals(primals: &[PrimalInfo]) -> usize {
    primals
        .iter()
        .filter(|p| matches!(p.health, PrimalHealthStatus::Healthy))
        .count()
}

/// Watch-style health summary (healthy/total)
#[must_use]
pub fn format_watch_health_summary(healthy: usize, total: usize) -> String {
    if healthy == total {
        format!("✅ {healthy}/{total} OK")
    } else {
        format!("⚠️ {healthy}/{total}")
    }
}

#[must_use]
pub const fn watch_health_all_ok(healthy: usize, total: usize) -> bool {
    healthy == total
}

#[must_use]
pub fn format_cli_primal_line(status: &str, name: &str) -> String {
    format!("[{status}] {name}")
}

#[must_use]
pub fn format_topology_node_count(count: usize) -> String {
    format!("Topology: {count} nodes")
}

#[must_use]
pub fn format_metrics_line(metrics_data: &str) -> String {
    format!("Metrics: {metrics_data}")
}

#[must_use]
pub fn format_watch_topology_count(count: usize) -> String {
    format!("🕸️ {count}")
}

/// Phone-style status color RGB for primal health
#[must_use]
pub const fn format_phone_primal_color_rgb(health: PrimalHealthStatus) -> [u8; 3] {
    match health {
        PrimalHealthStatus::Healthy => [0, 255, 0],
        PrimalHealthStatus::Warning => [255, 255, 0],
        PrimalHealthStatus::Critical => [255, 0, 0],
        PrimalHealthStatus::Unknown => [128, 128, 128],
    }
}

/// Phone-style status icon for primal health
#[must_use]
pub const fn format_phone_primal_icon(health: PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "✅",
        PrimalHealthStatus::Warning => "⚠️",
        PrimalHealthStatus::Critical => "❌",
        PrimalHealthStatus::Unknown => "❓",
    }
}

/// Desktop/tablet status indicator (text, rgb color)
#[must_use]
pub const fn format_desktop_primal_indicator(
    health: PrimalHealthStatus,
) -> (&'static str, [u8; 3]) {
    match health {
        PrimalHealthStatus::Healthy => ("●", [0, 255, 0]),
        PrimalHealthStatus::Warning => ("●", [255, 255, 0]),
        PrimalHealthStatus::Critical => ("●", [255, 0, 0]),
        PrimalHealthStatus::Unknown => ("○", [128, 128, 128]),
    }
}
