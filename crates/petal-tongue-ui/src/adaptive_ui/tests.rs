// SPDX-License-Identifier: AGPL-3.0-or-later
//! Adaptive UI tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::formatting::{
    count_healthy_primals, effective_device_for_rendering, format_cli_primal_line,
    format_cli_primal_status, format_desktop_primal_indicator, format_metrics_line,
    format_phone_primal_color_rgb, format_phone_primal_icon, format_topology_node_count,
    format_watch_health_summary, format_watch_topology_count, watch_health_all_ok,
};
use super::*;
use petal_tongue_core::{PrimalHealthStatus, PrimalId};

#[test]
fn test_adaptive_ui_manager_creation() {
    let caps = RenderingCapabilities::detect();
    let manager = AdaptiveUIManager::new(caps.clone());
    assert_eq!(manager.device_type(), caps.device_type);
    assert_eq!(manager.ui_complexity(), caps.ui_complexity);
}

#[test]
fn test_desktop_renderer_selection() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Desktop;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::Desktop);
}

#[test]
fn test_phone_renderer_selection() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Phone;
    caps.ui_complexity = UIComplexity::Minimal;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::Phone);
    assert_eq!(manager.ui_complexity(), UIComplexity::Minimal);
}

#[test]
fn test_watch_renderer_selection() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Watch;
    caps.ui_complexity = UIComplexity::Essential;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::Watch);
    assert_eq!(manager.ui_complexity(), UIComplexity::Essential);
}

#[test]
fn test_unknown_device_defaults_to_desktop() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Unknown;
    let _manager = AdaptiveUIManager::new(caps);
}

#[test]
fn test_tablet_renderer_selection() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Tablet;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::Tablet);
}

#[test]
fn test_cli_renderer_selection() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::CLI;
    caps.ui_complexity = UIComplexity::Essential;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::CLI);
    assert_eq!(manager.ui_complexity(), UIComplexity::Essential);
}

#[test]
fn test_tv_renderer_selection() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::TV;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::TV);
}

#[test]
fn test_ui_complexity_levels() {
    let mut caps = RenderingCapabilities::detect();
    caps.ui_complexity = UIComplexity::Full;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.ui_complexity(), UIComplexity::Full);

    let mut caps = RenderingCapabilities::detect();
    caps.ui_complexity = UIComplexity::Simplified;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.ui_complexity(), UIComplexity::Simplified);
}

#[test]
fn test_all_device_types_create_manager() {
    for device_type in [
        DeviceType::Desktop,
        DeviceType::Phone,
        DeviceType::Watch,
        DeviceType::CLI,
        DeviceType::Tablet,
        DeviceType::TV,
        DeviceType::Unknown,
    ] {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = device_type;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), device_type);
    }
}

#[test]
fn test_unknown_device_uses_desktop_renderer() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Unknown;
    let manager = AdaptiveUIManager::new(caps);
    assert_eq!(manager.device_type(), DeviceType::Unknown);
}

#[test]
fn test_effective_device_for_rendering() {
    assert_eq!(
        effective_device_for_rendering(DeviceType::Unknown),
        DeviceType::Desktop
    );
    assert_eq!(
        effective_device_for_rendering(DeviceType::Phone),
        DeviceType::Phone
    );
    assert_eq!(
        effective_device_for_rendering(DeviceType::Desktop),
        DeviceType::Desktop
    );
}

#[test]
fn test_format_cli_primal_status() {
    assert_eq!(format_cli_primal_status(PrimalHealthStatus::Healthy), "OK");
    assert_eq!(
        format_cli_primal_status(PrimalHealthStatus::Warning),
        "WARN"
    );
    assert_eq!(
        format_cli_primal_status(PrimalHealthStatus::Critical),
        "CRIT"
    );
    assert_eq!(
        format_cli_primal_status(PrimalHealthStatus::Unknown),
        "UNKN"
    );
}

#[test]
fn test_format_watch_health_summary() {
    assert_eq!(format_watch_health_summary(5, 5), "✅ 5/5 OK");
    assert_eq!(format_watch_health_summary(3, 5), "⚠️ 3/5");
}

#[test]
fn test_format_phone_primal_icon() {
    assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Healthy), "✅");
    assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Warning), "⚠️");
    assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Critical), "❌");
    assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Unknown), "❓");
}

#[test]
fn test_format_desktop_primal_indicator() {
    let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Healthy);
    assert_eq!(text, "●");
    assert_eq!(rgb, [0, 255, 0]);

    let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Warning);
    assert_eq!(text, "●");
    assert_eq!(rgb, [255, 255, 0]);

    let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Critical);
    assert_eq!(text, "●");
    assert_eq!(rgb, [255, 0, 0]);

    let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Unknown);
    assert_eq!(text, "○");
    assert_eq!(rgb, [128, 128, 128]);
}

#[test]
fn test_format_watch_health_summary_edge_cases() {
    assert_eq!(format_watch_health_summary(0, 5), "⚠️ 0/5");
    assert_eq!(format_watch_health_summary(1, 1), "✅ 1/1 OK");
    assert_eq!(format_watch_health_summary(0, 0), "✅ 0/0 OK");
}

#[test]
fn test_format_phone_primal_color_rgb() {
    let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Healthy);
    assert_eq!(rgb, [0, 255, 0]);
    let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Warning);
    assert_eq!(rgb, [255, 255, 0]);
    let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Critical);
    assert_eq!(rgb, [255, 0, 0]);
    let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Unknown);
    assert_eq!(rgb, [128, 128, 128]);
}

#[test]
fn test_watch_health_all_ok() {
    assert!(watch_health_all_ok(5, 5));
    assert!(!watch_health_all_ok(3, 5));
}

#[test]
fn test_format_cli_primal_line() {
    assert_eq!(format_cli_primal_line("OK", "primal1"), "[OK] primal1");
}

#[test]
fn test_format_topology_node_count() {
    assert_eq!(format_topology_node_count(10), "Topology: 10 nodes");
}

#[test]
fn test_format_metrics_line() {
    assert_eq!(format_metrics_line("cpu: 50%"), "Metrics: cpu: 50%");
}

#[test]
fn test_format_watch_topology_count() {
    assert_eq!(format_watch_topology_count(3), "🕸️ 3");
}

#[test]
fn test_count_healthy_primals() {
    let primals = vec![
        PrimalInfo::new(
            PrimalId::from("a"),
            "a",
            "",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            PrimalId::from("b"),
            "b",
            "",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Warning,
            0,
        ),
    ];
    assert_eq!(count_healthy_primals(&primals), 1);
    assert_eq!(count_healthy_primals(&[]), 0);
}

#[test]
fn test_count_healthy_primals_all_healthy() {
    let primals = vec![
        PrimalInfo::new(
            "a",
            "a",
            "t",
            "http://x",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            "b",
            "b",
            "t",
            "http://x",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
    ];
    assert_eq!(count_healthy_primals(&primals), 2);
}

#[test]
fn test_count_healthy_primals_none_healthy() {
    let primals = vec![
        PrimalInfo::new(
            "a",
            "a",
            "t",
            "http://x",
            vec![],
            PrimalHealthStatus::Critical,
            0,
        ),
        PrimalInfo::new(
            "b",
            "b",
            "t",
            "http://x",
            vec![],
            PrimalHealthStatus::Unknown,
            0,
        ),
    ];
    assert_eq!(count_healthy_primals(&primals), 0);
}

fn sample_primals() -> Vec<PrimalInfo> {
    vec![
        PrimalInfo::new(
            "p1",
            "Primal One",
            "compute",
            "http://localhost:8080",
            vec!["cap1".to_string(), "cap2".to_string()],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            "p2",
            "Primal Two",
            "storage",
            "http://localhost:8081",
            vec![],
            PrimalHealthStatus::Warning,
            0,
        ),
    ]
}

#[test]
fn test_render_primal_list_desktop() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Desktop;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

#[test]
fn test_render_primal_list_empty() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Desktop;
    let manager = AdaptiveUIManager::new(caps);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &[]);
        });
    });
}

#[test]
fn test_render_topology_all_devices() {
    let primals = sample_primals();
    for device in [
        DeviceType::Desktop,
        DeviceType::Phone,
        DeviceType::Watch,
        DeviceType::CLI,
        DeviceType::Tablet,
        DeviceType::TV,
    ] {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = device;
        let manager = AdaptiveUIManager::new(caps);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_topology(ui, &primals);
            });
        });
    }
}

#[test]
fn test_render_metrics_all_devices() {
    let metrics = "cpu: 45% | mem: 2.1GB";
    for device in [
        DeviceType::Desktop,
        DeviceType::Phone,
        DeviceType::Watch,
        DeviceType::CLI,
        DeviceType::Tablet,
        DeviceType::TV,
    ] {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = device;
        let manager = AdaptiveUIManager::new(caps);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_metrics(ui, metrics);
            });
        });
    }
}

#[test]
fn test_render_primal_list_phone() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Phone;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

#[test]
fn test_render_primal_list_watch() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Watch;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

#[test]
fn test_render_primal_list_cli() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::CLI;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

#[test]
fn test_render_primal_list_tablet() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Tablet;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

#[test]
fn test_render_primal_list_tv() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::TV;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

#[test]
fn test_render_unknown_device_uses_desktop_renderer() {
    let mut caps = RenderingCapabilities::detect();
    caps.device_type = DeviceType::Unknown;
    let manager = AdaptiveUIManager::new(caps);
    let primals = sample_primals();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
            manager.render_topology(ui, &primals);
            manager.render_metrics(ui, "test");
        });
    });
}
