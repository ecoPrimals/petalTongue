//! UI Components for Collaborative Intelligence
//!
//! Provides egui widgets for displaying:
//! - Node status and progress
//! - AI reasoning (transparent decision-making)
//! - Conflict resolution (human vs AI modifications)

use egui::{Color32, RichText, Ui};

use super::streaming::{AIReasoning, Alternative, ErrorInfo, NodeStatus, Pattern, ResourceUsage};

/// Status display widget - Shows node execution status
///
/// Displays real-time status updates for graph nodes.
pub struct StatusDisplay;

impl StatusDisplay {
    /// Render node status badge
    pub fn show_node_status(ui: &mut Ui, node_id: &str, status: &NodeStatus) {
        ui.horizontal(|ui| {
            // Status icon and color
            let (icon, color, text) = match status {
                NodeStatus::Pending => ("⚪", Color32::GRAY, "Pending"),
                NodeStatus::Running { progress } => {
                    let color = Color32::from_rgb(0, 128, 255);
                    let text = format!("Running ({}%)", progress);
                    ("🔵", color, text.leak() as &str)
                }
                NodeStatus::Completed => ("✅", Color32::GREEN, "Completed"),
                NodeStatus::Failed { .. } => ("❌", Color32::RED, "Failed"),
                NodeStatus::Paused => ("⏸️", Color32::YELLOW, "Paused"),
            };

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
            // Progress bar
            let progress_bar = egui::ProgressBar::new(progress)
                .text(format!("{:.0}%", progress * 100.0))
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
        ui.vertical(|ui| {
            ui.heading(RichText::new("Resources").size(14.0));

            ui.horizontal(|ui| {
                ui.label("CPU:");
                ui.label(RichText::new(format!("{:.1}%", resources.cpu_percent)).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Memory:");
                ui.label(RichText::new(format!("{} MB", resources.memory_mb)).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Disk I/O:");
                ui.label(RichText::new(format!("{:.1} MB/s", resources.disk_io_mbps)).strong());
            });

            ui.horizontal(|ui| {
                ui.label("Network:");
                ui.label(RichText::new(format!("{:.1} MB/s", resources.network_mbps)).strong());
            });
        });
    }

    /// Render error information
    pub fn show_error(ui: &mut Ui, error: &ErrorInfo) {
        ui.vertical(|ui| {
            ui.colored_label(
                Color32::RED,
                RichText::new(format!("❌ {}", error.error_type)).strong(),
            );

            ui.label(&error.message);

            if let Some(details) = &error.details {
                ui.indent("error_details", |ui| {
                    ui.label(RichText::new(details).italics().size(12.0));
                });
            }

            if error.recoverable {
                ui.colored_label(Color32::from_rgb(255, 165, 0), "⚠️ Recoverable error");

                if let Some(action) = &error.suggested_action {
                    ui.colored_label(Color32::YELLOW, format!("💡 Suggestion: {}", action));
                }
            } else {
                ui.colored_label(Color32::RED, "❌ Non-recoverable error");
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
                            let confidence_color = if reasoning.confidence > 0.8 {
                                Color32::GREEN
                            } else if reasoning.confidence > 0.5 {
                                Color32::YELLOW
                            } else {
                                Color32::from_rgb(255, 165, 0)
                            };

                            ui.colored_label(
                                confidence_color,
                                RichText::new(format!(
                                    "Confidence: {:.0}%",
                                    reasoning.confidence * 100.0
                                ))
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
                                ui.label(format!("  {}.", i + 1));
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
                                ui.label(format!("  • {}", source));
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
        ui.horizontal(|ui| {
            ui.label("  •");
            ui.label(&alt.description);
            ui.label(
                RichText::new(format!("({:.0}%)", alt.confidence * 100.0))
                    .size(12.0)
                    .color(Color32::GRAY),
            );
        });

        ui.indent("alt_reason", |ui| {
            ui.label(
                RichText::new(format!("→ {}", alt.reason_not_chosen))
                    .size(12.0)
                    .italics()
                    .color(Color32::GRAY),
            );
        });
    }

    /// Show historical pattern
    fn show_pattern(ui: &mut Ui, pattern: &Pattern) {
        ui.horizontal(|ui| {
            ui.label("  •");
            ui.label(&pattern.description);
            ui.label(
                RichText::new(format!(
                    "({}, {:.0}% relevant)",
                    pattern.source,
                    pattern.relevance * 100.0
                ))
                .size(12.0)
                .color(Color32::GRAY),
            );
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
                            RichText::new("⚠️ Conflict Detected")
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
                            ui.label(
                                RichText::new("Your Change:")
                                    .strong()
                                    .color(Color32::from_rgb(100, 200, 255)),
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
                            ui.label(
                                RichText::new("AI Suggestion:")
                                    .strong()
                                    .color(Color32::from_rgb(255, 200, 100)),
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
