// SPDX-License-Identifier: AGPL-3.0-or-later
//! UI Components for Collaborative Intelligence
//!
//! Provides egui widgets for displaying:
//! - Node status and progress
//! - AI reasoning (transparent decision-making)
//! - Conflict resolution (human vs AI modifications)

use egui::{Color32, RichText, Ui};

use super::streaming::{AIReasoning, Alternative, ErrorInfo, NodeStatus, Pattern, ResourceUsage};

// --- Pure logic (testable, no egui) ---

/// Node status display data: (icon, `color_rgb`, text).
#[must_use]
pub fn node_status_display(status: &NodeStatus) -> (&'static str, [u8; 3], String) {
    match status {
        NodeStatus::Pending => ("⚪", [128, 128, 128], "Pending".to_string()),
        NodeStatus::Running { progress } => ("🔵", [0, 128, 255], format!("Running ({progress}%)")),
        NodeStatus::Completed => ("✅", [0, 255, 0], "Completed".to_string()),
        NodeStatus::Failed { .. } => ("❌", [255, 0, 0], "Failed".to_string()),
        NodeStatus::Paused => ("⏸️", [255, 255, 0], "Paused".to_string()),
    }
}

/// Progress value (0.0–1.0) to percent display string.
#[must_use]
pub fn progress_percent_text(progress: f32) -> String {
    format!("{:.0}%", progress * 100.0)
}

/// Resource usage display strings: (cpu, memory, `disk_io`, network).
#[must_use]
pub fn resource_usage_display(resources: &ResourceUsage) -> (String, String, String, String) {
    (
        format!("{:.1}%", resources.cpu_percent),
        format!("{} MB", resources.memory_mb),
        format!("{:.1} MB/s", resources.disk_io_mbps),
        format!("{:.1} MB/s", resources.network_mbps),
    )
}

/// Alternative display data: (description, `confidence_str`, `reason_str`).
#[must_use]
pub fn alternative_display(alt: &Alternative) -> (&str, String, String) {
    (
        &alt.description,
        format!("({:.0}%)", alt.confidence * 100.0),
        format!("→ {}", alt.reason_not_chosen),
    )
}

/// Pattern display data: (description, `relevance_str`).
#[must_use]
pub fn pattern_display(pattern: &Pattern) -> (&str, String) {
    (
        &pattern.description,
        format!(
            "({}, {:.0}% relevant)",
            pattern.source,
            pattern.relevance * 100.0
        ),
    )
}

/// Confidence value (0.0–1.0) to RGB color.
#[must_use]
pub fn confidence_color_rgb(confidence: f32) -> [u8; 3] {
    if confidence > 0.8 {
        [0, 255, 0] // Green
    } else if confidence > 0.5 {
        [255, 255, 0] // Yellow
    } else {
        [255, 165, 0] // Orange
    }
}

#[must_use]
pub fn error_header_text(error: &ErrorInfo) -> String {
    format!("❌ {}", error.error_type)
}

#[must_use]
pub fn error_recoverable_display(error: &ErrorInfo) -> (String, Option<String>) {
    (
        if error.recoverable {
            "⚠️ Recoverable error".to_string()
        } else {
            "❌ Non-recoverable error".to_string()
        },
        error
            .suggested_action
            .as_ref()
            .map(|a| format!("💡 Suggestion: {a}")),
    )
}

#[must_use]
pub const fn error_recoverable_color_rgb(recoverable: bool) -> [u8; 3] {
    if recoverable {
        [255, 165, 0]
    } else {
        [255, 0, 0]
    }
}

#[must_use]
pub fn confidence_percent_text(confidence: f32) -> String {
    format!("{:.0}%", confidence * 100.0)
}

// --- UI widgets (use egui) ---

/// Status display widget - Shows node execution status
///
/// Displays real-time status updates for graph nodes.
pub struct StatusDisplay;

impl StatusDisplay {
    /// Render node status badge
    pub fn show_node_status(ui: &mut Ui, node_id: &str, status: &NodeStatus) {
        let (icon, color_rgb, text) = node_status_display(status);
        let color = Color32::from_rgb(color_rgb[0], color_rgb[1], color_rgb[2]);

        ui.horizontal(|ui| {
            // Icon
            ui.label(RichText::new(icon).size(16.0));

            // Node ID
            ui.label(RichText::new(node_id).strong().color(color));

            // Status text
            ui.label(RichText::new(text).color(color));
        });

        // Show error details if failed
        if let NodeStatus::Failed { error } = status {
            ui.indent("error_details", |ui| {
                ui.label(RichText::new(error).color(Color32::RED).italics());
            });
        }
    }

    /// Render progress bar
    pub fn show_progress(ui: &mut Ui, progress: f32, message: &str) {
        ui.vertical(|ui| {
            let progress_bar = egui::ProgressBar::new(progress)
                .text(progress_percent_text(progress))
                .animate(true);

            ui.add(progress_bar);

            // Progress message
            if !message.is_empty() {
                ui.label(RichText::new(message).italics().size(12.0));
            }
        });
    }

    /// Render resource usage
    pub fn show_resources(ui: &mut Ui, resources: &ResourceUsage) {
        let (cpu_str, mem_str, disk_str, net_str) = resource_usage_display(resources);
        ui.vertical(|ui| {
            ui.heading(RichText::new("Resources").size(14.0));

            ui.horizontal(|ui| {
                ui.label("CPU:");
                ui.label(RichText::new(cpu_str).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Memory:");
                ui.label(RichText::new(mem_str).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Disk I/O:");
                ui.label(RichText::new(disk_str).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Network:");
                ui.label(RichText::new(net_str).strong());
            });
        });
    }

    /// Render error information
    pub fn show_error(ui: &mut Ui, error: &ErrorInfo) {
        let header = error_header_text(error);
        let (recoverable_msg, suggestion_msg) = error_recoverable_display(error);
        let rgb = error_recoverable_color_rgb(error.recoverable);
        let recoverable_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
        ui.vertical(|ui| {
            ui.colored_label(Color32::RED, RichText::new(header).strong());

            ui.label(&error.message);

            if let Some(details) = &error.details {
                ui.indent("error_details", |ui| {
                    ui.label(RichText::new(details).italics().size(12.0));
                });
            }

            ui.colored_label(recoverable_color, recoverable_msg);
            if let Some(suggestion) = suggestion_msg {
                ui.colored_label(Color32::YELLOW, suggestion);
            }
        });
    }
}

/// AI Reasoning display widget - Transparent AI decision-making
///
/// Shows "why" the AI made a decision, alternatives considered, and data sources.
pub struct ReasoningDisplay;

impl ReasoningDisplay {
    /// Render AI reasoning panel
    pub fn show(ui: &mut Ui, reasoning: &AIReasoning) {
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(30, 30, 40))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🤖 AI Reasoning").size(16.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let rgb = confidence_color_rgb(reasoning.confidence);
                            let confidence_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);

                            ui.colored_label(
                                confidence_color,
                                RichText::new(format_confidence_display(reasoning.confidence))
                                    .strong(),
                            );
                        });
                    });

                    ui.separator();

                    // Decision
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Decision:").strong());
                        ui.label(&reasoning.decision);
                    });

                    ui.add_space(8.0);

                    // Rationale
                    if !reasoning.rationale.is_empty() {
                        ui.label(RichText::new("Why:").strong());
                        for (i, reason) in reasoning.rationale.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format_rationale_item(i));
                                ui.label(reason);
                            });
                        }
                        ui.add_space(8.0);
                    }

                    // Alternatives
                    if !reasoning.alternatives.is_empty() {
                        ui.label(RichText::new("Alternatives Considered:").strong());
                        for alt in &reasoning.alternatives {
                            Self::show_alternative(ui, alt);
                        }
                        ui.add_space(8.0);
                    }

                    // Data sources
                    if !reasoning.data_sources.is_empty() {
                        ui.label(RichText::new("Data Sources:").strong());
                        ui.horizontal_wrapped(|ui| {
                            for source in &reasoning.data_sources {
                                ui.label(format_data_source_item(source));
                            }
                        });
                        ui.add_space(8.0);
                    }

                    // Patterns
                    if !reasoning.patterns.is_empty() {
                        ui.label(RichText::new("Historical Patterns:").strong());
                        for pattern in &reasoning.patterns {
                            Self::show_pattern(ui, pattern);
                        }
                    }
                });
            });
    }

    /// Show alternative option
    fn show_alternative(ui: &mut Ui, alt: &Alternative) {
        let (desc, conf_str, reason_str) = alternative_display(alt);
        ui.horizontal(|ui| {
            ui.label("  •");
            ui.label(desc);
            ui.label(RichText::new(conf_str).size(12.0).color(Color32::GRAY));
        });

        ui.indent("alt_reason", |ui| {
            ui.label(
                RichText::new(reason_str)
                    .size(12.0)
                    .italics()
                    .color(Color32::GRAY),
            );
        });
    }

    /// Show historical pattern
    fn show_pattern(ui: &mut Ui, pattern: &Pattern) {
        let (desc, rel_str) = pattern_display(pattern);
        ui.horizontal(|ui| {
            ui.label("  •");
            ui.label(desc);
            ui.label(RichText::new(rel_str).size(12.0).color(Color32::GRAY));
        });
    }
}

/// Conflict resolution widget - Handle concurrent modifications
///
/// Shows conflicts between user and AI modifications, allowing user to choose.
pub struct ConflictResolution;

/// Conflict between concurrent modifications
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// User's proposed change
    pub user_change: String,
    /// AI's proposed change
    pub ai_change: String,
    /// Human-readable conflict description
    pub description: String,
}

/// Type of conflict
#[derive(Debug, Clone)]
pub enum ConflictType {
    /// User change vs AI suggestion
    UserVsAI,
    /// User change vs another user's change
    UserVsUser,
    /// Modification during execution
    ExecutionVsModification,
}

/// User's choice for resolving a conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolutionChoice {
    /// Keep user's change, discard AI's
    KeepUser,
    /// Keep AI's change, discard user's
    KeepAI,
    /// Merge both changes
    MergeBoth,
    /// Cancel and revert both
    Cancel,
}

impl ConflictResolution {
    /// Show conflict resolution dialog
    pub fn show(ui: &mut Ui, conflict: &Conflict) -> Option<ConflictResolutionChoice> {
        let mut result = None;

        egui::Frame::popup(ui.style())
            .fill(Color32::from_rgb(40, 40, 50))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(conflict_header_text())
                                .size(16.0)
                                .strong()
                                .color(Color32::YELLOW),
                        );
                    });

                    ui.separator();

                    // Description
                    ui.label(&conflict.description);
                    ui.add_space(8.0);

                    // Show changes
                    ui.horizontal(|ui| {
                        // User change
                        ui.vertical(|ui| {
                            let rgb = conflict_user_label_color_rgb();
                            ui.label(
                                RichText::new(conflict_user_label_text())
                                    .strong()
                                    .color(Color32::from_rgb(rgb[0], rgb[1], rgb[2])),
                            );
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(30, 30, 40))
                                .show(ui, |ui| {
                                    ui.label(&conflict.user_change);
                                });
                        });

                        ui.add_space(16.0);

                        // AI change
                        ui.vertical(|ui| {
                            let rgb = conflict_ai_label_color_rgb();
                            ui.label(
                                RichText::new(conflict_ai_label_text())
                                    .strong()
                                    .color(Color32::from_rgb(rgb[0], rgb[1], rgb[2])),
                            );
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(30, 30, 40))
                                .show(ui, |ui| {
                                    ui.label(&conflict.ai_change);
                                });
                        });
                    });

                    ui.add_space(16.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("✓ Keep My Change").clicked() {
                            result = Some(ConflictResolutionChoice::KeepUser);
                        }

                        if ui.button("🤖 Use AI Suggestion").clicked() {
                            result = Some(ConflictResolutionChoice::KeepAI);
                        }

                        if ui.button("🔀 Merge Both").clicked() {
                            result = Some(ConflictResolutionChoice::MergeBoth);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✗ Cancel").clicked() {
                                result = Some(ConflictResolutionChoice::Cancel);
                            }
                        });
                    });
                });
            });

        result
    }
}

#[must_use]
pub const fn conflict_user_label_color_rgb() -> [u8; 3] {
    [100, 200, 255]
}

#[must_use]
pub const fn conflict_ai_label_color_rgb() -> [u8; 3] {
    [255, 200, 100]
}

#[must_use]
pub const fn conflict_user_label_text() -> &'static str {
    "Your Change:"
}

#[must_use]
pub const fn conflict_ai_label_text() -> &'static str {
    "AI Suggestion:"
}

#[must_use]
pub const fn conflict_header_text() -> &'static str {
    "⚠️ Conflict Detected"
}

#[must_use]
pub fn format_confidence_display(confidence: f32) -> String {
    format!("Confidence: {}", confidence_percent_text(confidence))
}

#[must_use]
pub fn format_data_source_item(source: &str) -> String {
    format!("  • {source}")
}

#[must_use]
pub fn format_rationale_item(index: usize) -> String {
    format!("  {}.", index + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_editor::streaming::{
        Alternative, ErrorInfo, NodeStatus, Pattern, ResourceUsage,
    };

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
        let variants = vec![
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
}
