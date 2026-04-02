// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::tool_integration::{ToolCapability, ToolPanel};
use egui::Color32;
use std::collections::VecDeque;

// === Pure function tests ===

#[test]
fn threshold_color_red_above_high() {
    let c = threshold_color(95.0, 90.0, 70.0, Color32::BLUE);
    assert_eq!(c, Color32::from_rgb(200, 50, 50));
}

#[test]
fn threshold_color_amber_above_mid() {
    let c = threshold_color(75.0, 90.0, 70.0, Color32::BLUE);
    assert_eq!(c, Color32::from_rgb(200, 150, 50));
}

#[test]
fn threshold_color_normal_below_mid() {
    let normal = Color32::from_rgb(50, 150, 200);
    let c = threshold_color(50.0, 90.0, 70.0, normal);
    assert_eq!(c, normal);
}

#[test]
fn threshold_color_boundary_at_high() {
    let c = threshold_color(90.0, 90.0, 70.0, Color32::BLUE);
    assert_eq!(c, Color32::from_rgb(200, 150, 50), "boundary = mid color");
}

#[test]
fn threshold_color_boundary_at_mid() {
    let c = threshold_color(70.0, 90.0, 70.0, Color32::BLUE);
    assert_eq!(c, Color32::BLUE, "boundary = normal color");
}

#[test]
fn sparkline_points_empty_data() {
    let data = VecDeque::new();
    assert!(compute_sparkline_points(&data, 100.0, 50.0, 100.0).is_empty());
}

#[test]
fn sparkline_points_single_datum() {
    let mut data = VecDeque::new();
    data.push_back(50.0);
    assert!(
        compute_sparkline_points(&data, 100.0, 50.0, 100.0).is_empty(),
        "need >= 2 points"
    );
}

#[test]
fn sparkline_points_two_data() {
    let mut data = VecDeque::new();
    data.push_back(0.0);
    data.push_back(100.0);
    let pts = compute_sparkline_points(&data, 200.0, 100.0, 100.0);
    assert_eq!(pts.len(), 2);
    assert!((pts[0][0]).abs() < f32::EPSILON, "first x = 0");
    assert!(
        (pts[0][1] - 100.0).abs() < f32::EPSILON,
        "0% value = bottom"
    );
    assert!((pts[1][0] - 200.0).abs() < f32::EPSILON, "last x = width");
    assert!((pts[1][1]).abs() < f32::EPSILON, "100% value = top");
}

#[test]
fn sparkline_points_mid_value() {
    let mut data = VecDeque::new();
    data.push_back(50.0);
    data.push_back(50.0);
    let pts = compute_sparkline_points(&data, 100.0, 100.0, 100.0);
    assert_eq!(pts.len(), 2);
    assert!((pts[0][1] - 50.0).abs() < f32::EPSILON, "50% = midpoint");
}

#[test]
fn sparkline_clamps_above_max() {
    let mut data = VecDeque::new();
    data.push_back(200.0);
    data.push_back(200.0);
    let pts = compute_sparkline_points(&data, 100.0, 100.0, 100.0);
    assert!((pts[0][1]).abs() < f32::EPSILON, "clamped to top");
}

#[test]
fn format_gb_zero() {
    assert_eq!(format_gb(0), "0.0");
}

#[test]
fn format_gb_one_gb() {
    assert_eq!(format_gb(1_073_741_824), "1.0");
}

#[test]
fn format_gb_sixteen_gb() {
    assert_eq!(format_gb(16 * 1_073_741_824), "16.0");
}

#[test]
fn format_gb_fractional() {
    assert_eq!(format_gb(1_610_612_736), "1.5");
}

#[test]
fn threshold_color_zero() {
    let normal = Color32::from_rgb(100, 100, 100);
    let c = threshold_color(0.0, 90.0, 70.0, normal);
    assert_eq!(c, normal);
}

#[test]
fn sparkline_points_three_data() {
    let mut data = VecDeque::new();
    data.push_back(0.0);
    data.push_back(50.0);
    data.push_back(100.0);
    let pts = compute_sparkline_points(&data, 100.0, 50.0, 100.0);
    assert_eq!(pts.len(), 3);
    assert!((pts[1][0] - 50.0).abs() < f32::EPSILON);
}

// === Display state tests ===

#[test]
fn cpu_display_state_low_usage() {
    let state = prepare_cpu_display(25.0, 8, 10);
    assert_eq!(state.usage, 25.0);
    assert!((state.bar_fraction - 0.25).abs() < f32::EPSILON);
    assert_eq!(state.bar_color, Color32::from_rgb(50, 150, 200));
    assert_eq!(state.label, "25.0%");
    assert_eq!(state.core_count, 8);
    assert_eq!(
        state.history_label.as_deref(),
        Some("History (10 samples) [LIVE DATA]")
    );
}

#[test]
fn cpu_display_state_high_usage() {
    let state = prepare_cpu_display(95.0, 4, 0);
    assert_eq!(state.bar_color, Color32::from_rgb(200, 50, 50));
    assert!(state.history_label.is_none());
}

#[test]
fn cpu_display_state_mid_usage() {
    let state = prepare_cpu_display(80.0, 4, 5);
    assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
}

#[test]
fn memory_display_state_normal() {
    let total = 16 * 1_073_741_824_u64;
    let used = 4 * 1_073_741_824_u64;
    let state = prepare_memory_display(used, total, 30);
    assert!((state.percent - 25.0).abs() < 0.01);
    assert_eq!(state.bar_color, Color32::from_rgb(50, 200, 150));
    assert!(state.used_gb_label.contains("4.0"));
    assert!(state.used_gb_label.contains("16.0"));
    assert!(state.history_label.is_some());
}

#[test]
fn memory_display_state_critical() {
    let total = 16 * 1_073_741_824_u64;
    let used = 15 * 1_073_741_824_u64;
    let state = prepare_memory_display(used, total, 0);
    assert_eq!(state.bar_color, Color32::from_rgb(200, 50, 50));
    assert!(state.history_label.is_none());
}

#[test]
fn memory_display_state_zero_total() {
    let state = prepare_memory_display(0, 0, 0);
    assert!((state.percent).abs() < f64::EPSILON);
    assert_eq!(state.label, "0.0%");
}

// === Integration tests (tool-level, public API only) ===

#[test]
fn test_system_monitor_default() {
    let tool = SystemMonitorTool::default();
    assert!(!tool.is_visible());
    let meta = tool.metadata();
    assert_eq!(meta.name, "System Monitor");
    assert!(meta.description.contains("Real-time"));
    assert!(meta.capabilities.contains(&ToolCapability::Visual));
    assert!(
        meta.capabilities
            .contains(&ToolCapability::Custom("RealTime".to_string()))
    );
}

#[test]
fn test_system_monitor_toggle_visibility() {
    let mut tool = SystemMonitorTool::default();
    assert!(!tool.is_visible());
    tool.toggle_visibility();
    assert!(tool.is_visible());
    tool.toggle_visibility();
    assert!(!tool.is_visible());
}

#[test]
fn test_system_monitor_status_message() {
    let tool = SystemMonitorTool::default();
    let msg = tool.status_message();
    assert!(msg.is_some());
    let msg = msg.unwrap();
    assert!(msg.contains("CPU:"));
    assert!(msg.contains("MEM:"));
    assert!(msg.contains('%'));
}

#[test]
fn test_system_monitor_metadata_source() {
    let tool = SystemMonitorTool::default();
    let meta = tool.metadata();
    assert_eq!(meta.icon, "📡");
    assert!(meta.source.is_some());
    assert!(meta.source.as_ref().unwrap().contains("kernel.org"));
}

#[test]
fn test_system_monitor_metadata_version() {
    let tool = SystemMonitorTool::default();
    let meta = tool.metadata();
    assert_eq!(meta.version, "0.1.0");
}

#[test]
fn test_system_monitor_toggle_multiple() {
    let mut tool = SystemMonitorTool::default();
    for _ in 0..4 {
        tool.toggle_visibility();
    }
    assert!(!tool.is_visible());
}

#[test]
fn test_system_monitor_status_message_format() {
    let tool = SystemMonitorTool::default();
    let msg = tool.status_message().expect("status message");
    assert!(msg.starts_with("CPU:"));
    assert!(msg.contains("MEM:"));
    assert!(msg.contains('%'));
    let parts: Vec<&str> = msg.split('|').collect();
    assert_eq!(parts.len(), 2);
}

#[test]
fn test_system_monitor_metadata_capabilities_count() {
    let tool = SystemMonitorTool::default();
    let meta = tool.metadata();
    assert!(meta.capabilities.len() >= 2);
}

#[test]
fn test_cpu_display_accessor() {
    let tool = SystemMonitorTool::default();
    let display = tool.cpu_display();
    assert_eq!(display.label, "0.0%");
}

#[test]
fn test_mem_display_accessor() {
    let tool = SystemMonitorTool::default();
    let display = tool.mem_display();
    assert_eq!(display.label, "0.0%");
}

#[test]
fn test_status_message_zero_memory() {
    let tool = SystemMonitorTool::default();
    let msg = tool.status_message();
    assert!(msg.is_some());
    let msg = msg.unwrap();
    assert!(msg.contains("CPU:"));
    assert!(msg.contains("MEM:"));
}

#[test]
fn test_prepare_memory_display_amber_threshold() {
    let total = 16 * 1_073_741_824_u64;
    let used = 12 * 1_073_741_824_u64;
    let state = prepare_memory_display(used, total, 5);
    assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
}

#[test]
fn test_prepare_cpu_display_amber_threshold() {
    let state = prepare_cpu_display(75.0, 8, 20);
    assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
}

#[test]
fn test_threshold_color_exactly_mid() {
    let c = threshold_color(70.0, 90.0, 70.0, Color32::GREEN);
    assert_eq!(c, Color32::GREEN);
}

#[test]
fn test_threshold_color_exactly_high() {
    let c = threshold_color(91.0, 90.0, 70.0, Color32::BLUE);
    assert_eq!(c, Color32::from_rgb(200, 50, 50));
}

#[test]
fn test_sparkline_points_four_data() {
    let mut data = VecDeque::new();
    data.push_back(0.0);
    data.push_back(25.0);
    data.push_back(50.0);
    data.push_back(100.0);
    let pts = compute_sparkline_points(&data, 300.0, 100.0, 100.0);
    assert_eq!(pts.len(), 4);
    assert!((pts[0][0]).abs() < f32::EPSILON);
    assert!((pts[3][0] - 300.0).abs() < f32::EPSILON);
}

#[test]
fn test_format_gb_small() {
    assert_eq!(format_gb(107_374_182), "0.1");
}

#[test]
fn test_memory_display_state_amber() {
    let total = 16 * 1_073_741_824_u64;
    let used = 12 * 1_073_741_824_u64;
    let state = prepare_memory_display(used, total, 10);
    assert!((state.percent - 75.0).abs() < 0.01);
    assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
}

#[test]
fn test_cpu_display_state_no_history() {
    let state = prepare_cpu_display(50.0, 4, 0);
    assert!(state.history_label.is_none());
}

#[test]
fn test_metadata_icon() {
    let tool = SystemMonitorTool::default();
    let meta = tool.metadata();
    assert!(!meta.icon.is_empty());
}

#[test]
fn test_metadata_name() {
    let tool = SystemMonitorTool::default();
    let meta = tool.metadata();
    assert_eq!(meta.name, "System Monitor");
}
