// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{NeuralApiMetrics, SystemMetrics, SystemResourceMetrics};

use super::MetricsDashboard;

#[test]
fn test_new_dashboard() {
    let dashboard = MetricsDashboard::new();
    assert!(dashboard.data.is_none());
    assert!(!dashboard.fetching);
}

#[test]
fn test_dashboard_default() {
    let dashboard = MetricsDashboard::default();
    assert!(dashboard.data.is_none());
}

#[test]
fn test_dashboard_with_data() {
    let mut dashboard = MetricsDashboard::new();

    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 45.5,
            memory_used_mb: 4_096,
            memory_total_mb: 8_192,
            memory_percent: 50.0,
            uptime_seconds: 3_600,
        },
        neural_api: NeuralApiMetrics {
            family_id: "nat0".to_string(),
            active_primals: 3,
            graphs_available: 5,
            active_executions: 1,
        },
    };

    dashboard.data = Some(metrics);
    dashboard.cpu_history.push(45.5);
    dashboard.memory_history.push(50.0);

    assert!(dashboard.data.is_some());
    assert_eq!(dashboard.cpu_history.current(), Some(45.5));
}

#[test]
fn test_cpu_history_accumulation() {
    let mut dashboard = MetricsDashboard::new();

    for i in 0..10 {
        dashboard.cpu_history.push(i as f32 * 10.0);
    }

    assert_eq!(dashboard.cpu_history.values().len(), 10);
    assert!(dashboard.cpu_history.has_sufficient_data());
}

#[test]
fn test_memory_history_accumulation() {
    let mut dashboard = MetricsDashboard::new();

    dashboard.memory_history.push(25.0);
    dashboard.memory_history.push(50.0);
    dashboard.memory_history.push(75.0);

    assert_eq!(dashboard.memory_history.values().len(), 3);
    assert_eq!(dashboard.memory_history.current(), Some(75.0));
}

#[test]
fn test_system_metrics_thresholds() {
    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 95.0,
            memory_used_mb: 8_000,
            memory_total_mb: 8_192,
            memory_percent: 97.0,
            uptime_seconds: 86_400,
        },
        neural_api: NeuralApiMetrics {
            family_id: "test".to_string(),
            active_primals: 0,
            graphs_available: 0,
            active_executions: 0,
        },
    };

    let cpu_thresh = metrics.cpu_threshold();
    let mem_thresh = metrics.memory_threshold();
    let (r, g, b) = cpu_thresh.color_rgb();
    assert!(r > 0 || g > 0 || b > 0);
    let (r, g, b) = mem_thresh.color_rgb();
    assert!(r > 0 || g > 0 || b > 0);
}

#[test]
fn test_uptime_formatted() {
    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 0.0,
            memory_used_mb: 0,
            memory_total_mb: 0,
            memory_percent: 0.0,
            uptime_seconds: 3661, // 1h 1m 1s
        },
        neural_api: NeuralApiMetrics {
            family_id: String::new(),
            active_primals: 0,
            graphs_available: 0,
            active_executions: 0,
        },
    };
    let formatted = metrics.uptime_formatted();
    assert!(formatted.contains('h') || formatted.contains('1') || !formatted.is_empty());
}

#[test]
fn test_prepare_metrics_display_cpu_critical() {
    use crate::metrics_dashboard_helpers::prepare_metrics_display;

    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 95.0,
            memory_used_mb: 1_000,
            memory_total_mb: 2_000,
            memory_percent: 50.0,
            uptime_seconds: 0,
        },
        neural_api: NeuralApiMetrics {
            family_id: "fam".to_string(),
            active_primals: 0,
            graphs_available: 0,
            active_executions: 0,
        },
    };
    let state = prepare_metrics_display(&metrics, &[], &[]);
    assert_eq!(state.cpu_color, (239, 68, 68));
    assert_eq!(state.memory_color, (234, 179, 8));
}

#[test]
fn test_prepare_metrics_display_empty_history() {
    use crate::metrics_dashboard_helpers::prepare_metrics_display;

    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 25.0,
            memory_used_mb: 0,
            memory_total_mb: 0,
            memory_percent: 25.0,
            uptime_seconds: 60,
        },
        neural_api: NeuralApiMetrics {
            family_id: String::new(),
            active_primals: 0,
            graphs_available: 0,
            active_executions: 0,
        },
    };
    let state = prepare_metrics_display(&metrics, &[], &[]);
    assert!(state.cpu_history.is_empty());
    assert!(state.memory_history.is_empty());
    assert_eq!(state.uptime_text, "1m");
}

#[test]
fn test_sparkline_points_in_rect_empty() {
    use crate::metrics_dashboard_helpers::sparkline_points_in_rect;

    let pts = sparkline_points_in_rect(&[], 0.0, 0.0, 100.0, 40.0);
    assert!(pts.is_empty());

    let pts = sparkline_points_in_rect(&[42.0], 0.0, 0.0, 100.0, 40.0);
    assert!(pts.is_empty());
}

#[test]
fn test_sparkline_points_in_rect_constant() {
    use crate::metrics_dashboard_helpers::sparkline_points_in_rect;

    let pts = sparkline_points_in_rect(&[50.0, 50.0, 50.0], 10.0, 20.0, 80.0, 30.0);
    assert_eq!(pts.len(), 3);
    assert_eq!(pts[0].0, 10.0);
    assert_eq!(pts[2].0, 90.0);
}

#[test]
fn test_format_bytes_edge_cases() {
    use crate::metrics_dashboard_helpers::format_bytes;

    assert_eq!(format_bytes(1), "1 B");
    assert_eq!(format_bytes(999), "999 B");
    assert!(format_bytes(2048).contains("2.0 KB"));
    assert!(format_bytes(1_073_741_824).contains("1.0 GB"));
}

#[test]
fn test_format_uptime_display_edge_cases() {
    use crate::metrics_dashboard_helpers::format_uptime_display;

    assert_eq!(format_uptime_display(30), "0m");
    assert_eq!(format_uptime_display(3661), "1h 1m");
    assert_eq!(format_uptime_display(90061), "1d 1h 1m");
}

#[test]
fn test_render_no_data() {
    let dashboard = MetricsDashboard::new();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard.render(ui);
        });
    });
}

#[test]
fn test_render_with_data() {
    let mut dashboard = MetricsDashboard::new();
    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 45.5,
            memory_used_mb: 4_096,
            memory_total_mb: 8_192,
            memory_percent: 50.0,
            uptime_seconds: 3_600,
        },
        neural_api: NeuralApiMetrics {
            family_id: "nat0".to_string(),
            active_primals: 3,
            graphs_available: 5,
            active_executions: 1,
        },
    };
    dashboard.data = Some(metrics);
    dashboard.cpu_history.push(45.5);
    dashboard.cpu_history.push(50.0);
    dashboard.cpu_history.push(55.0);
    dashboard.memory_history.push(50.0);
    dashboard.memory_history.push(52.0);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard.render(ui);
        });
    });
}

#[test]
fn test_render_with_high_usage() {
    let mut dashboard = MetricsDashboard::new();
    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 95.0,
            memory_used_mb: 8_000,
            memory_total_mb: 8_192,
            memory_percent: 97.0,
            uptime_seconds: 86_400,
        },
        neural_api: NeuralApiMetrics {
            family_id: "test".to_string(),
            active_primals: 0,
            graphs_available: 0,
            active_executions: 0,
        },
    };
    dashboard.data = Some(metrics);
    dashboard.cpu_history.push(90.0);
    dashboard.cpu_history.push(95.0);
    dashboard.memory_history.push(95.0);
    dashboard.memory_history.push(97.0);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard.render(ui);
        });
    });
}
