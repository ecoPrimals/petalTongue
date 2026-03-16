// SPDX-License-Identifier: AGPL-3.0-only
//! Metrics Dashboard - Pure helper functions (headless-first, fully testable)
//!
//! Extracted logic for sparklines, threshold colors, and display state preparation.

use petal_tongue_core::SystemMetrics;

/// Map value to RGB color based on warning/critical thresholds.
/// Green below warning, yellow between warning and critical, red at/above critical.
#[must_use]
pub fn threshold_color_rgb(value: f64, warning: f64, critical: f64) -> (u8, u8, u8) {
    if value < warning {
        (34, 197, 94) // green-500
    } else if value < critical {
        (234, 179, 8) // yellow-500
    } else {
        (239, 68, 68) // red-500
    }
}

/// Generate sparkline point coordinates from history data.
/// Returns points (x, y) in rect [0, width] x [0, height]; y=0 at top (min value).
#[must_use]
#[cfg_attr(not(test), expect(dead_code, reason = "public API for headless testing"))]
pub fn sparkline_points(history: &[f64], width: f32, height: f32) -> Vec<(f32, f32)> {
    if history.len() < 2 {
        return Vec::new();
    }
    let min_val = history.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = history.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max_val - min_val).max(0.1);

    history
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = (i as f32 / (history.len() - 1) as f32) * width;
            let normalized = (value - min_val) / range;
            let y = (normalized as f32).mul_add(-height, height);
            (x, y)
        })
        .collect()
}

/// Sparkline points with explicit rect origin (for egui rendering).
#[must_use]
pub fn sparkline_points_in_rect(
    history: &[f32],
    x_start: f32,
    y_start: f32,
    width: f32,
    height: f32,
) -> Vec<(f32, f32)> {
    if history.len() < 2 {
        return Vec::new();
    }
    let min_val = history.iter().copied().fold(f32::INFINITY, f32::min);
    let max_val = history.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_val - min_val).max(0.1);

    history
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = (i as f32 / (history.len() - 1) as f32).mul_add(width, x_start);
            let normalized = (value - min_val) / range;
            let y = y_start + height - normalized * height;
            (x, y)
        })
        .collect()
}

/// Precomputed display state for metrics dashboard rendering.
#[derive(Debug, Clone)]
pub struct MetricDisplayState {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage percentage
    pub memory_percent: f64,
    /// CPU color (r, g, b) based on thresholds
    pub cpu_color: (u8, u8, u8),
    /// Memory color (r, g, b) based on thresholds
    pub memory_color: (u8, u8, u8),
    /// CPU history for sparkline
    pub cpu_history: Vec<f64>,
    /// Memory history for sparkline
    pub memory_history: Vec<f64>,
    /// Formatted uptime string
    pub uptime_text: String,
    /// Whether Neural API is connected (data available)
    #[cfg_attr(not(test), expect(dead_code, reason = "display state API for headless testing"))]
    pub neural_api_connected: bool,
    /// Memory used in MB (for display)
    pub memory_used_mb: u64,
    /// Memory total in MB (for display)
    pub memory_total_mb: u64,
    /// Neural API family ID
    pub family_id: String,
    /// Active primals count
    pub active_primals: u32,
    /// Available graphs count
    pub graphs_available: u32,
    /// Active executions count
    pub active_executions: u32,
}

/// Default CPU/memory warning and critical thresholds (50%, 80%).
const WARNING_THRESHOLD: f64 = 50.0;
const CRITICAL_THRESHOLD: f64 = 80.0;

/// Prepare display state from metrics and history data.
#[must_use]
pub fn prepare_metrics_display(
    data: &SystemMetrics,
    cpu_history: &[f64],
    memory_history: &[f64],
) -> MetricDisplayState {
    let cpu_percent = f64::from(data.system.cpu_percent);
    let memory_percent = f64::from(data.system.memory_percent);

    MetricDisplayState {
        cpu_percent,
        memory_percent,
        cpu_color: threshold_color_rgb(cpu_percent, WARNING_THRESHOLD, CRITICAL_THRESHOLD),
        memory_color: threshold_color_rgb(memory_percent, WARNING_THRESHOLD, CRITICAL_THRESHOLD),
        cpu_history: cpu_history.to_vec(),
        memory_history: memory_history.to_vec(),
        uptime_text: format_uptime_display(data.system.uptime_seconds),
        neural_api_connected: true,
        memory_used_mb: data.system.memory_used_mb,
        memory_total_mb: data.system.memory_total_mb,
        family_id: data.neural_api.family_id.clone(),
        active_primals: data.neural_api.active_primals,
        graphs_available: data.neural_api.graphs_available,
        active_executions: data.neural_api.active_executions,
    }
}

/// Format byte count as human-readable string (e.g., "1.5 MB").
#[must_use]
#[cfg_attr(not(test), expect(dead_code, reason = "public API for headless testing"))]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Compute (avg, max) from a history slice. Returns (0.0, 0.0) for empty.
#[must_use]
pub fn cpu_history_avg_max(history: &[f64]) -> (f64, f64) {
    if history.is_empty() {
        return (0.0, 0.0);
    }
    let sum: f64 = history.iter().sum();
    let avg = sum / history.len() as f64;
    let max = history.iter().copied().fold(0.0_f64, f64::max);
    (avg, max)
}

/// RGB color for active executions: green when > 0, gray when 0.
#[must_use]
pub const fn active_executions_color_rgb(active: u32) -> [u8; 3] {
    if active > 0 {
        [34, 197, 94]
    } else {
        [156, 163, 175]
    }
}

#[must_use]
pub fn format_cpu_percent(percent: f64) -> String {
    format!("{percent:.1}%")
}

#[must_use]
pub fn format_cpu_avg_display(avg: f64) -> String {
    format!("Avg: {avg:.1}%")
}

#[must_use]
pub fn format_cpu_max_display(max: f64) -> String {
    format!("Max: {max:.1}%")
}

#[must_use]
pub fn format_memory_used_total(used_mb: u64, total_mb: u64) -> String {
    format!("{used_mb} MB / {total_mb} MB")
}

/// Format uptime seconds into human-readable string (e.g., "1d 2h 34m").
#[must_use]
pub fn format_uptime_display(uptime_secs: u64) -> String {
    let days = uptime_secs / 86_400;
    let hours = (uptime_secs % 86_400) / 3_600;
    let minutes = (uptime_secs % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{NeuralApiMetrics, SystemResourceMetrics};

    #[test]
    fn test_threshold_color_below_warning() {
        let (r, g, b) = threshold_color_rgb(30.0, 50.0, 80.0);
        assert_eq!((r, g, b), (34, 197, 94)); // green
    }

    #[test]
    fn test_threshold_color_at_warning() {
        let (r, g, b) = threshold_color_rgb(50.0, 50.0, 80.0);
        assert_eq!((r, g, b), (234, 179, 8)); // yellow (at warning = not below)
    }

    #[test]
    fn test_threshold_color_between_warning_and_critical() {
        let (r, g, b) = threshold_color_rgb(65.0, 50.0, 80.0);
        assert_eq!((r, g, b), (234, 179, 8)); // yellow
    }

    #[test]
    fn test_threshold_color_at_critical() {
        let (r, g, b) = threshold_color_rgb(80.0, 50.0, 80.0);
        assert_eq!((r, g, b), (239, 68, 68)); // red
    }

    #[test]
    fn test_threshold_color_above_critical() {
        let (r, g, b) = threshold_color_rgb(95.0, 50.0, 80.0);
        assert_eq!((r, g, b), (239, 68, 68)); // red
    }

    #[test]
    fn test_sparkline_points_empty() {
        let pts = sparkline_points(&[], 100.0, 40.0);
        assert!(pts.is_empty());
    }

    #[test]
    fn test_sparkline_points_single() {
        let pts = sparkline_points(&[42.0], 100.0, 40.0);
        assert!(pts.is_empty());
    }

    #[test]
    fn test_sparkline_points_two_points() {
        let pts = sparkline_points(&[0.0, 100.0], 100.0, 40.0);
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], (0.0, 40.0)); // min at bottom
        assert_eq!(pts[1], (100.0, 0.0)); // max at top
    }

    #[test]
    fn test_sparkline_points_multiple() {
        let pts = sparkline_points(&[10.0, 50.0, 90.0], 200.0, 50.0);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].0 - 0.0).abs() < 0.01);
        assert!((pts[1].0 - 100.0).abs() < 0.01);
        assert!((pts[2].0 - 200.0).abs() < 0.01);
        assert!((pts[0].1 - 50.0).abs() < 0.01); // min at bottom
        assert!((pts[2].1 - 0.0).abs() < 0.01); // max at top
    }

    #[test]
    fn test_sparkline_points_constant_values() {
        let pts = sparkline_points(&[50.0, 50.0, 50.0], 100.0, 40.0);
        assert_eq!(pts.len(), 3);
        for (_, y) in &pts {
            assert!((*y - 40.0).abs() < 0.01); // at bottom when min==max (normalized=0)
        }
    }

    #[test]
    fn test_sparkline_points_in_rect() {
        let pts = sparkline_points_in_rect(&[0.0f32, 100.0f32], 10.0, 20.0, 80.0, 30.0);
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], (10.0, 50.0)); // x_start, y_start+height
        assert_eq!(pts[1], (90.0, 20.0)); // x_start+width, y_start
    }

    #[test]
    fn test_prepare_metrics_display() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 45.0,
                memory_used_mb: 4_096,
                memory_total_mb: 8_192,
                memory_percent: 70.0,
                uptime_seconds: 3_661,
            },
            neural_api: NeuralApiMetrics {
                family_id: "nat0".to_string(),
                active_primals: 3,
                graphs_available: 5,
                active_executions: 1,
            },
        };
        let cpu = vec![30.0, 45.0, 60.0];
        let mem = vec![50.0, 60.0, 70.0];

        let state = prepare_metrics_display(&metrics, &cpu, &mem);

        assert!((state.cpu_percent - 45.0).abs() < 0.01);
        assert!((state.memory_percent - 70.0).abs() < 0.01);
        assert_eq!(state.cpu_color, (34, 197, 94)); // green (< 50)
        assert_eq!(state.memory_color, (234, 179, 8)); // yellow (50-80)
        assert_eq!(state.cpu_history, cpu);
        assert_eq!(state.memory_history, mem);
        assert!(state.uptime_text.contains('h'));
        assert!(state.neural_api_connected);
        assert_eq!(state.memory_used_mb, 4_096);
        assert_eq!(state.memory_total_mb, 8_192);
        assert_eq!(state.family_id, "nat0");
        assert_eq!(state.active_primals, 3);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert!(format_bytes(1024).starts_with("1.0 KB"));
        assert!(format_bytes(1024 * 1024).starts_with("1.0 MB"));
        assert!(format_bytes(1536 * 1024).starts_with("1.5 MB"));
        assert!(format_bytes(1024 * 1024 * 1024).starts_with("1.0 GB"));
    }

    #[test]
    fn test_format_uptime_display() {
        assert_eq!(format_uptime_display(0), "0m");
        assert_eq!(format_uptime_display(60), "1m");
        assert_eq!(format_uptime_display(3_600), "1h 0m");
        assert_eq!(format_uptime_display(86_400), "1d 0h 0m");
        assert_eq!(format_uptime_display(90_061), "1d 1h 1m");
    }

    #[test]
    fn test_cpu_history_avg_max() {
        let (avg, max) = cpu_history_avg_max(&[]);
        assert_eq!(avg, 0.0);
        assert_eq!(max, 0.0);

        let (avg, max) = cpu_history_avg_max(&[30.0, 50.0, 70.0]);
        assert!((avg - 50.0).abs() < 0.01);
        assert_eq!(max, 70.0);

        let (avg, max) = cpu_history_avg_max(&[100.0]);
        assert_eq!(avg, 100.0);
        assert_eq!(max, 100.0);
    }

    #[test]
    fn test_active_executions_color_rgb() {
        let rgb = active_executions_color_rgb(0);
        assert_eq!(rgb, [156, 163, 175]);

        let rgb = active_executions_color_rgb(1);
        assert_eq!(rgb, [34, 197, 94]);

        let rgb = active_executions_color_rgb(5);
        assert_eq!(rgb, [34, 197, 94]);
    }

    #[test]
    fn test_format_cpu_percent() {
        assert_eq!(format_cpu_percent(45.5), "45.5%");
        assert_eq!(format_cpu_percent(100.0), "100.0%");
    }

    #[test]
    fn test_format_cpu_avg_display() {
        assert_eq!(format_cpu_avg_display(50.0), "Avg: 50.0%");
    }

    #[test]
    fn test_format_cpu_max_display() {
        assert_eq!(format_cpu_max_display(75.0), "Max: 75.0%");
    }

    #[test]
    fn test_format_memory_used_total() {
        assert_eq!(format_memory_used_total(1024, 2048), "1024 MB / 2048 MB");
    }
}
