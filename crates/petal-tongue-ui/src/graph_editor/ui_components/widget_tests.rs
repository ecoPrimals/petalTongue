// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::graph_editor::streaming::{
    AIReasoning, Alternative, ErrorInfo, NodeStatus, Pattern, ResourceUsage,
};

#[test]
fn test_status_display_show_node_status_pending() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_node_status(ui, "node-1", &NodeStatus::Pending);
        });
    });
}

#[test]
fn test_status_display_show_node_status_failed() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_node_status(
                ui,
                "node-1",
                &NodeStatus::Failed {
                    error: "Connection failed".to_string(),
                },
            );
        });
    });
}

#[test]
fn test_status_display_show_progress() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_progress(ui, 0.5, "Processing...");
        });
    });
}

#[test]
fn test_status_display_show_progress_empty_message() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_progress(ui, 0.0, "");
        });
    });
}

#[test]
fn test_status_display_show_resources() {
    let resources = ResourceUsage {
        cpu_percent: 45.5,
        memory_mb: 1024,
        disk_io_mbps: 12.3,
        network_mbps: 5.6,
    };
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_resources(ui, &resources);
        });
    });
}

#[test]
fn test_status_display_show_error() {
    let error = ErrorInfo {
        error_type: "ConnectionError".to_string(),
        message: "Failed to connect".to_string(),
        details: Some("Connection refused".to_string()),
        recoverable: true,
        suggested_action: Some("Retry".to_string()),
    };
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_error(ui, &error);
        });
    });
}

#[test]
fn test_status_display_show_error_minimal() {
    let error = ErrorInfo {
        error_type: "X".to_string(),
        message: "Y".to_string(),
        details: None,
        recoverable: false,
        suggested_action: None,
    };
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            StatusDisplay::show_error(ui, &error);
        });
    });
}

#[test]
fn test_reasoning_display_show_minimal() {
    let reasoning = AIReasoning {
        decision: "Use option A".to_string(),
        confidence: 0.9,
        rationale: vec![],
        alternatives: vec![],
        data_sources: vec![],
        patterns: vec![],
    };
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ReasoningDisplay::show(ui, &reasoning);
        });
    });
}

#[test]
fn test_reasoning_display_show_full() {
    let reasoning = AIReasoning {
        decision: "Use option A".to_string(),
        confidence: 0.9,
        rationale: vec!["Reason 1".to_string(), "Reason 2".to_string()],
        alternatives: vec![Alternative {
            description: "Option B".to_string(),
            confidence: 0.7,
            reason_not_chosen: "Less reliable".to_string(),
        }],
        data_sources: vec!["api".to_string(), "cache".to_string()],
        patterns: vec![Pattern {
            description: "Similar case".to_string(),
            source: "history".to_string(),
            relevance: 0.9,
        }],
    };
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ReasoningDisplay::show(ui, &reasoning);
        });
    });
}

#[test]
fn test_conflict_resolution_show() {
    let conflict = Conflict {
        conflict_type: ConflictType::UserVsAI,
        user_change: "User's change".to_string(),
        ai_change: "AI's suggestion".to_string(),
        description: "Concurrent modification".to_string(),
    };
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _choice = ConflictResolution::show(ui, &conflict);
        });
    });
}

#[test]
fn test_conflict_type_variants() {
    match ConflictType::UserVsAI {
        ConflictType::UserVsAI => {}
        _ => panic!("expected UserVsAI"),
    }
    match ConflictType::UserVsUser {
        ConflictType::UserVsUser => {}
        _ => panic!("expected UserVsUser"),
    }
    match ConflictType::ExecutionVsModification {
        ConflictType::ExecutionVsModification => {}
        _ => panic!("expected ExecutionVsModification"),
    }
}
