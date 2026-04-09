// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_live_badge_creation() {
    let badge = LiveBadge::new("test".to_string(), 1.0);
    assert!(!badge.indicator.is_live);
}

#[test]
fn test_live_metric_update() {
    let mut metric = LiveMetric::new("CPU".to_string(), "proc".to_string(), 1.0);
    metric.update("45.2".to_string(), Some("%".to_string()));
    assert_eq!(metric.value, "45.2");
    assert_eq!(metric.unit, Some("%".to_string()));
    assert!(metric.badge.indicator.is_live);
}

#[test]
fn test_connection_status() {
    let mut status = ConnectionStatus::new(default_connection_target());
    assert!(!status.connected);

    status.mark_connected();
    assert!(status.connected);
    assert!(status.last_connection.is_some());

    status.mark_disconnected();
    assert!(!status.connected);
}

#[test]
fn test_live_badge_mark_updated() {
    let mut badge = LiveBadge::new("test".to_string(), 1.0);
    badge.mark_updated();
    assert!(badge.indicator.is_live);
}

#[test]
fn test_live_graph_header_creation() {
    let mut header = LiveGraphHeader::new("Test".to_string(), "source".to_string(), 1.0);
    header.mark_updated();
    // Just verify no panic
}

#[test]
fn test_live_metric_without_unit() {
    let mut metric = LiveMetric::new("Test".to_string(), "src".to_string(), 1.0);
    metric.update("42".to_string(), None);
    assert_eq!(metric.value, "42");
    assert_eq!(metric.unit, None);
}

#[test]
fn test_connection_status_target_default() {
    let target = default_connection_target();
    let status = ConnectionStatus::new(target.clone());
    assert_eq!(status.target, target);
}

#[test]
fn test_live_badge_new_and_mark_updated() {
    let mut badge = LiveBadge::new(String::new(), 0.5);
    badge.mark_updated();
    assert!(badge.indicator.is_live);
}

#[test]
fn test_live_metric_value_with_unit_formatting() {
    let mut metric = LiveMetric::new("Temp".to_string(), "sensor".to_string(), 1.0);
    metric.update("72.5".to_string(), Some("°F".to_string()));
    assert_eq!(metric.value, "72.5");
    assert_eq!(metric.unit.as_deref(), Some("°F"));
}

#[test]
fn test_connection_status_mark_disconnected_preserves_target() {
    let mut status = ConnectionStatus::new("target:9000".to_string());
    status.mark_connected();
    status.mark_disconnected();
    assert!(!status.connected);
    assert_eq!(status.target, "target:9000");
}

#[test]
fn test_default_connection_target() {
    let target = default_connection_target();
    assert!(!target.is_empty());
    assert!(target.contains(':') || target.contains("localhost"));
}

#[test]
fn test_live_graph_header_creation_and_mark_updated() {
    let mut header = LiveGraphHeader::new("Test".to_string(), "source".to_string(), 1.0);
    assert_eq!(header.title, "Test");
    header.mark_updated();
    assert!(header.badge.indicator.is_live);
}

#[test]
fn test_live_metric_creation_defaults() {
    let metric = LiveMetric::new("Label".to_string(), "src".to_string(), 2.0);
    assert_eq!(metric.label, "Label");
    assert_eq!(metric.value, "0");
    assert_eq!(metric.unit, None);
}

#[test]
fn test_connection_status_last_connection_preserved() {
    let mut status = ConnectionStatus::new("localhost:3000".to_string());
    assert!(status.last_connection.is_none());
    status.mark_connected();
    assert!(status.last_connection.is_some());
}

#[test]
fn test_connection_status_target_custom() {
    let target = "biomeOS at 127.0.0.1:8080".to_string();
    let status = ConnectionStatus::new(target.clone());
    assert_eq!(status.target, target);
}

#[test]
fn test_default_connection_target_non_empty() {
    let target = default_connection_target();
    assert!(!target.is_empty());
}

#[test]
fn test_live_metric_value_with_unit() {
    let mut metric = LiveMetric::new("Test".to_string(), "src".to_string(), 1.0);
    metric.update("99.5".to_string(), Some("ms".to_string()));
    assert_eq!(metric.value, "99.5");
    assert_eq!(metric.unit, Some("ms".to_string()));
}

#[test]
fn test_live_graph_header_title_and_badge() {
    let mut header = LiveGraphHeader::new("Test".to_string(), "src".to_string(), 2.0);
    assert_eq!(header.title, "Test");
    header.mark_updated();
    assert!(header.badge.indicator.is_live);
}

#[test]
fn test_badge_display_state_integration() {
    use crate::live_data_helpers::badge_display_state;
    let s = badge_display_state(0.5, false, true);
    assert_eq!(s.label, "● LIVE");
    let s = badge_display_state(10.0, true, true);
    assert_eq!(s.label, "STALE");
    let s = badge_display_state(0.0, false, false);
    assert_eq!(s.label, "WAITING");
}

#[test]
fn test_connection_status_display_integration() {
    use crate::live_data_helpers::connection_status_display;
    let s = connection_status_display(true, None);
    assert_eq!(s.symbol, "●");
    assert_eq!(s.status_text, "Connected");
    let s = connection_status_display(false, Some(30.0));
    assert_eq!(s.symbol, "○");
    assert_eq!(s.status_text, "Disconnected");
}

#[test]
fn test_format_age_for_display_integration() {
    use crate::live_data_helpers::format_age_for_display;
    assert_eq!(format_age_for_display(0.5), "Just now");
    assert_eq!(format_age_for_display(45.0), "45.0s ago");
    assert_eq!(format_age_for_display(3660.0), "1.0h ago");
}

#[test]
fn test_format_metric_value_integration() {
    use crate::live_data_helpers::format_metric_value;
    assert_eq!(format_metric_value(72.5, "°F"), "72.5°F");
    assert_eq!(format_metric_value(42.0, ""), "42");
}

#[test]
fn test_live_metric_render_large_invalid_parse() {
    let mut metric = LiveMetric::new("Temp".to_string(), "sensor".to_string(), 1.0);
    metric.update("not_a_number".to_string(), Some("°C".to_string()));
    assert_eq!(metric.value, "not_a_number");
}

#[test]
fn test_connection_status_empty_target() {
    let status = ConnectionStatus::new(String::new());
    assert_eq!(status.target, "");
    assert!(!status.connected);
}

#[test]
fn test_live_badge_source_preserved() {
    let badge = LiveBadge::new("test_source".to_string(), 2.0);
    assert_eq!(badge.indicator.source, "test_source");
    assert!((badge.indicator.update_interval - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_live_badge_render() {
    let mut badge = LiveBadge::new("test".to_string(), 1.0);
    badge.mark_updated();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            badge.render(ui);
        });
    });
}

#[test]
fn test_live_badge_render_with_timestamp() {
    let mut badge = LiveBadge::new("test".to_string(), 1.0);
    badge.mark_updated();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            badge.render_with_timestamp(ui);
        });
    });
}

#[test]
fn test_live_badge_render_full() {
    let mut badge = LiveBadge::new("test".to_string(), 1.0);
    badge.mark_updated();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            badge.render_full(ui);
        });
    });
}

#[test]
fn test_live_graph_header_render() {
    let mut header = LiveGraphHeader::new("Test Graph".to_string(), "source".to_string(), 1.0);
    header.mark_updated();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            header.render(ui);
        });
    });
}

#[test]
fn test_live_graph_header_render_compact() {
    let mut header = LiveGraphHeader::new("Test".to_string(), "src".to_string(), 2.0);
    header.mark_updated();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            header.render_compact(ui);
        });
    });
}

#[test]
fn test_live_metric_render() {
    let mut metric = LiveMetric::new("CPU".to_string(), "proc".to_string(), 1.0);
    metric.update("45.2".to_string(), Some("%".to_string()));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            metric.render(ui);
        });
    });
}

#[test]
fn test_live_metric_render_large() {
    let mut metric = LiveMetric::new("Memory".to_string(), "proc".to_string(), 1.0);
    metric.update("2.5".to_string(), Some("GB".to_string()));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            metric.render_large(ui);
        });
    });
}

#[test]
fn test_connection_status_render() {
    let mut status = ConnectionStatus::new("localhost:3000".to_string());
    status.mark_connected();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            status.render(ui);
        });
    });
}

#[test]
fn test_connection_status_render_compact() {
    let status = ConnectionStatus::new("target:9000".to_string());

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            status.render_compact(ui);
        });
    });
}

#[test]
fn test_render_timestamp() {
    let instant = std::time::Instant::now();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_timestamp(ui, instant);
        });
    });
}

#[test]
fn test_request_live_updates() {
    let ctx = egui::Context::default();
    request_live_updates(&ctx);
}

#[test]
fn test_live_metric_render_without_unit() {
    let mut metric = LiveMetric::new("Count".to_string(), "src".to_string(), 1.0);
    metric.update("42".to_string(), None);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            metric.render(ui);
        });
    });
}

#[test]
fn test_connection_status_render_recently_connected() {
    let mut status = ConnectionStatus::new("localhost:3000".to_string());
    status.mark_connected();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            status.render(ui);
        });
    });
}

#[test]
fn test_live_graph_header_render_with_frequency() {
    let mut header = LiveGraphHeader::new("Test".to_string(), "src".to_string(), 2.5);
    header.mark_updated();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            header.render(ui);
        });
    });
}

#[test]
fn test_live_metric_render_large_with_unit() {
    let mut metric = LiveMetric::new("Temp".to_string(), "sensor".to_string(), 1.0);
    metric.update("98.6".to_string(), Some("°F".to_string()));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            metric.render_large(ui);
        });
    });
}

#[test]
fn test_badge_display_state_stale() {
    use crate::live_data_helpers::badge_display_state;
    let s = badge_display_state(100.0, true, true);
    assert_eq!(s.label, "STALE");
}

#[test]
fn test_badge_display_state_live_just_now() {
    use crate::live_data_helpers::badge_display_state;
    let s = badge_display_state(0.5, false, true);
    assert_eq!(s.label, "● LIVE");
}
