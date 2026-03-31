// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::graph_editor::streaming::{Alternative, ErrorInfo, NodeStatus, Pattern, ResourceUsage};

#[test]
fn test_progress_percent_text() {
    assert_eq!(progress_percent_text(0.0), "0%");
    assert_eq!(progress_percent_text(0.5), "50%");
    assert_eq!(progress_percent_text(1.0), "100%");
    assert_eq!(progress_percent_text(0.333), "33%");
}

#[test]
fn test_resource_usage_display() {
    let r = ResourceUsage {
        cpu_percent: 45.5,
        memory_mb: 1024,
        disk_io_mbps: 12.3,
        network_mbps: 5.6,
    };
    let (cpu, mem, disk, net) = resource_usage_display(&r);
    assert!(cpu.contains("45.5"));
    assert!(mem.contains("1024"));
    assert!(disk.contains("12.3"));
    assert!(net.contains("5.6"));
}

#[test]
fn test_alternative_display() {
    let alt = Alternative {
        description: "Option A".to_string(),
        confidence: 0.75,
        reason_not_chosen: "Less reliable".to_string(),
    };
    let (desc, conf, reason) = alternative_display(&alt);
    assert_eq!(desc, "Option A");
    assert_eq!(conf, "(75%)");
    assert!(reason.contains("Less reliable"));
}

#[test]
fn test_pattern_display() {
    let p = Pattern {
        description: "Similar case".to_string(),
        source: "history".to_string(),
        relevance: 0.9,
    };
    let (desc, rel) = pattern_display(&p);
    assert_eq!(desc, "Similar case");
    assert!(rel.contains("history"));
    assert!(rel.contains("90"));
}

#[test]
fn test_node_status_display_pending() {
    let (icon, rgb, text) = node_status_display(&NodeStatus::Pending);
    assert_eq!(icon, "⚪");
    assert_eq!(rgb, [128, 128, 128]);
    assert_eq!(text, "Pending");
}

#[test]
fn test_node_status_display_running() {
    let (icon, rgb, text) = node_status_display(&NodeStatus::Running { progress: 50 });
    assert_eq!(icon, "🔵");
    assert_eq!(rgb, [0, 128, 255]);
    assert_eq!(text, "Running (50%)");
}

#[test]
fn test_node_status_display_completed() {
    let (icon, rgb, text) = node_status_display(&NodeStatus::Completed);
    assert_eq!(icon, "✅");
    assert_eq!(rgb, [0, 255, 0]);
    assert_eq!(text, "Completed");
}

#[test]
fn test_node_status_display_failed() {
    let (icon, rgb, text) = node_status_display(&NodeStatus::Failed {
        error: "err".to_string(),
    });
    assert_eq!(icon, "❌");
    assert_eq!(rgb, [255, 0, 0]);
    assert_eq!(text, "Failed");
}

#[test]
fn test_node_status_display_paused() {
    let (icon, rgb, text) = node_status_display(&NodeStatus::Paused);
    assert_eq!(icon, "⏸️");
    assert_eq!(rgb, [255, 255, 0]);
    assert_eq!(text, "Paused");
}

#[test]
fn test_confidence_color_rgb_high() {
    let rgb = confidence_color_rgb(0.9);
    assert_eq!(rgb, [0, 255, 0]);
}

#[test]
fn test_confidence_color_rgb_mid() {
    let rgb = confidence_color_rgb(0.6);
    assert_eq!(rgb, [255, 255, 0]);
}

#[test]
fn test_confidence_color_rgb_low() {
    let rgb = confidence_color_rgb(0.3);
    assert_eq!(rgb, [255, 165, 0]);
}

#[test]
fn test_confidence_color_rgb_boundary() {
    let rgb_08 = confidence_color_rgb(0.8);
    assert_eq!(rgb_08, [255, 255, 0]); // 0.8 is not > 0.8
    let rgb_081 = confidence_color_rgb(0.81);
    assert_eq!(rgb_081, [0, 255, 0]);
}

#[test]
fn test_conflict_types() {
    let conflict = Conflict {
        conflict_type: ConflictType::UserVsAI,
        user_change: "User change".to_string(),
        ai_change: "AI change".to_string(),
        description: "Test conflict".to_string(),
    };

    assert!(matches!(conflict.conflict_type, ConflictType::UserVsAI));
}

#[test]
fn test_conflict_resolution_variants() {
    let variants = [
        ConflictResolutionChoice::KeepUser,
        ConflictResolutionChoice::KeepAI,
        ConflictResolutionChoice::MergeBoth,
        ConflictResolutionChoice::Cancel,
    ];

    assert_eq!(variants.len(), 4);
}

#[test]
fn test_error_header_text() {
    let err = ErrorInfo {
        error_type: "ConnectionError".to_string(),
        message: "Failed".to_string(),
        details: None,
        recoverable: false,
        suggested_action: None,
    };
    assert_eq!(error_header_text(&err), "❌ ConnectionError");
}

#[test]
fn test_error_recoverable_display_recoverable() {
    let err = ErrorInfo {
        error_type: "X".to_string(),
        message: "Y".to_string(),
        details: None,
        recoverable: true,
        suggested_action: Some("Retry".to_string()),
    };
    let (msg, sugg) = error_recoverable_display(&err);
    assert_eq!(msg, "⚠️ Recoverable error");
    assert_eq!(sugg, Some("💡 Suggestion: Retry".to_string()));
}

#[test]
fn test_error_recoverable_display_non_recoverable() {
    let err = ErrorInfo {
        error_type: "X".to_string(),
        message: "Y".to_string(),
        details: None,
        recoverable: false,
        suggested_action: None,
    };
    let (msg, sugg) = error_recoverable_display(&err);
    assert_eq!(msg, "❌ Non-recoverable error");
    assert_eq!(sugg, None);
}

#[test]
fn test_error_recoverable_color_rgb() {
    assert_eq!(error_recoverable_color_rgb(true), [255, 165, 0]);
    assert_eq!(error_recoverable_color_rgb(false), [255, 0, 0]);
}

#[test]
fn test_confidence_percent_text() {
    assert_eq!(confidence_percent_text(0.0), "0%");
    assert_eq!(confidence_percent_text(0.5), "50%");
    assert_eq!(confidence_percent_text(1.0), "100%");
    assert_eq!(confidence_percent_text(0.75), "75%");
}

#[test]
fn test_conflict_label_colors() {
    assert_eq!(conflict_user_label_color_rgb(), [100, 200, 255]);
    assert_eq!(conflict_ai_label_color_rgb(), [255, 200, 100]);
}

#[test]
fn test_conflict_label_texts() {
    assert_eq!(conflict_user_label_text(), "Your Change:");
    assert_eq!(conflict_ai_label_text(), "AI Suggestion:");
    assert_eq!(conflict_header_text(), "⚠️ Conflict Detected");
}

#[test]
fn test_format_confidence_display() {
    assert_eq!(format_confidence_display(0.5), "Confidence: 50%");
    assert_eq!(format_confidence_display(1.0), "Confidence: 100%");
}

#[test]
fn test_format_data_source_item() {
    assert_eq!(format_data_source_item("api"), "  • api");
}

#[test]
fn test_format_rationale_item() {
    assert_eq!(format_rationale_item(0), "  1.");
    assert_eq!(format_rationale_item(2), "  3.");
}
