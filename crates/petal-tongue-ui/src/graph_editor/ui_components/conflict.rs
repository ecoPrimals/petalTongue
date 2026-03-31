// SPDX-License-Identifier: AGPL-3.0-or-later
//! Conflict resolution widget and related types.

use egui::{Color32, RichText, Ui};

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
