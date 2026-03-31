// SPDX-License-Identifier: AGPL-3.0-or-later
//! AI reasoning panel — transparent decision-making display.

use egui::{Color32, RichText, Ui};

use super::display::{
    alternative_display, confidence_color_rgb, format_confidence_display, format_data_source_item,
    format_rationale_item, pattern_display,
};
use crate::graph_editor::streaming::{AIReasoning, Alternative, Pattern};

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
