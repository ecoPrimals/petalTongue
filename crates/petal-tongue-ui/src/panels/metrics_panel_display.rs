// SPDX-License-Identifier: AGPL-3.0-only
//! Pure display logic for Metrics Panel
//!
//! Extracted formatting and display state preparation for testability.

use crate::panels::metrics_panel::SystemMetrics;
use std::time::{Duration, Instant};

/// Formats uptime duration as human-readable string (e.g., "1h 5m", "2d 3h 0m").
#[must_use]
pub fn format_uptime(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

/// Formats update age for display (e.g., "Updated 5s ago", "Stale (>30s)").
#[must_use]
pub fn format_update_age(age_secs: u64) -> String {
    if age_secs >= 30 {
        "Stale (>30s)".to_string()
    } else {
        format!("Updated {age_secs}s ago")
    }
}

/// Pre-computed display state for the Metrics panel.
#[derive(Debug, Clone)]
pub struct MetricsPanelDisplayState {
    pub metrics_summary: Option<MetricsSummary>,
    pub error_message: Option<String>,
    pub update_age_text: Option<String>,
    pub is_stale: bool,
}

/// Summary of system metrics for display.
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub uptime_str: String,
    pub family_id: String,
    pub active_primals: u64,
    pub graphs_available: u64,
    pub active_executions: u64,
}

/// Prepares display state from metrics, last update time, and optional error.
#[must_use]
pub fn prepare_metrics_panel_display(
    metrics: &Option<SystemMetrics>,
    last_update: Instant,
    error: &Option<String>,
) -> MetricsPanelDisplayState {
    let age_secs = last_update.elapsed().as_secs();
    let is_stale = age_secs >= 30;

    let update_age_text = metrics.as_ref().map(|_| format_update_age(age_secs));

    let metrics_summary = metrics.as_ref().map(|m| MetricsSummary {
        cpu_percent: m.system.cpu_percent,
        memory_percent: m.system.memory_percent,
        memory_used_mb: m.system.memory_used_mb,
        memory_total_mb: m.system.memory_total_mb,
        uptime_str: format_uptime(Duration::from_secs(m.system.uptime_seconds)),
        family_id: m.neural_api.family_id.clone(),
        active_primals: m.neural_api.active_primals,
        graphs_available: m.neural_api.graphs_available,
        active_executions: m.neural_api.active_executions,
    });

    MetricsPanelDisplayState {
        metrics_summary,
        error_message: error.clone(),
        update_age_text,
        is_stale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::metrics_panel::{NeuralApiStats, SystemStats};

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(Duration::from_secs(59)), "0m");
        assert_eq!(format_uptime(Duration::from_secs(60)), "1m");
        assert_eq!(format_uptime(Duration::from_secs(3600)), "1h 0m");
        assert_eq!(format_uptime(Duration::from_secs(3661)), "1h 1m");
        assert_eq!(format_uptime(Duration::from_secs(86400)), "1d 0h 0m");
        assert_eq!(format_uptime(Duration::from_secs(90061)), "1d 1h 1m");
    }

    #[test]
    fn test_format_update_age() {
        assert_eq!(format_update_age(0), "Updated 0s ago");
        assert_eq!(format_update_age(5), "Updated 5s ago");
        assert_eq!(format_update_age(29), "Updated 29s ago");
        assert_eq!(format_update_age(30), "Stale (>30s)");
        assert_eq!(format_update_age(60), "Stale (>30s)");
    }

    #[test]
    fn test_prepare_metrics_panel_display_with_metrics() {
        let metrics = SystemMetrics {
            timestamp: "2026-01-15T22:00:00Z".to_string(),
            system: SystemStats {
                cpu_percent: 16.5,
                memory_used_mb: 32768,
                memory_total_mb: 49152,
                memory_percent: 66.7,
                uptime_seconds: 86400,
            },
            neural_api: NeuralApiStats {
                family_id: "nat0".to_string(),
                active_primals: 3,
                graphs_available: 5,
                active_executions: 0,
            },
        };

        let last_update = Instant::now() - Duration::from_secs(2);
        let display = prepare_metrics_panel_display(&Some(metrics), last_update, &None);

        assert!(display.metrics_summary.is_some());
        let summary = display.metrics_summary.unwrap();
        assert_eq!(summary.cpu_percent, 16.5);
        assert_eq!(summary.memory_percent, 66.7);
        assert_eq!(summary.uptime_str, "1d 0h 0m");
        assert_eq!(summary.family_id, "nat0");
        assert_eq!(summary.active_primals, 3);
        assert!(display.error_message.is_none());
        let age_text = display.update_age_text.as_ref().unwrap();
        assert!(age_text.starts_with("Updated ") && age_text.ends_with("s ago"));
        assert!(!display.is_stale);
    }

    #[test]
    fn test_prepare_metrics_panel_display_stale() {
        let last_update = Instant::now() - Duration::from_secs(45);
        let display = prepare_metrics_panel_display(
            &None,
            last_update,
            &Some("Neural API not available".to_string()),
        );

        assert!(display.metrics_summary.is_none());
        assert_eq!(
            display.error_message.as_deref(),
            Some("Neural API not available")
        );
        assert!(display.update_age_text.is_none());
        assert!(display.is_stale);
    }

    #[test]
    fn test_prepare_metrics_panel_display_with_metrics_stale() {
        let metrics = SystemMetrics {
            timestamp: "2026-01-15T22:00:00Z".to_string(),
            system: SystemStats {
                cpu_percent: 10.0,
                memory_used_mb: 1000,
                memory_total_mb: 2000,
                memory_percent: 50.0,
                uptime_seconds: 3600,
            },
            neural_api: NeuralApiStats {
                family_id: "test".to_string(),
                active_primals: 1,
                graphs_available: 2,
                active_executions: 0,
            },
        };

        let last_update = Instant::now() - Duration::from_secs(35);
        let display = prepare_metrics_panel_display(&Some(metrics), last_update, &None);

        assert!(display.metrics_summary.is_some());
        assert_eq!(display.update_age_text.as_deref(), Some("Stale (>30s)"));
        assert!(display.is_stale);
    }
}
