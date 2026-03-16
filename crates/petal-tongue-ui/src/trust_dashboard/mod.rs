// SPDX-License-Identifier: AGPL-3.0-only
//! Trust Status Dashboard
//!
//! Provides rich visualization of trust relationships and status across the primal network.
//! This module leverages the universal adapter system to display trust information
//! in a way that works with ANY trust model (not just ecoPrimals).
//!
//! Architecture: headless-first. All computation and decision logic is in pure
//! functions (`trust_level_to_display_row`, `average_trust_display`,
//! `prepare_trust_display`). Render methods are thin egui widget calls.
//! Audio playback is expressed as `TrustIntent::PlayAudio` rather than
//! calling `audio.play()` directly.

mod compute;
mod types;

use crate::accessibility::ColorPalette;
use egui::{Color32, RichText, Ui};
use petal_tongue_core::{PrimalInfo, PropertyValue};
use std::collections::HashMap;

use compute::trust_level_number_to_label;

#[cfg(test)]
mod tests;

// Re-export public API
pub use compute::{
    average_trust_display, prepare_trust_display, trust_level_style, trust_level_to_display_row,
};
pub use types::{AverageTrustDisplay, TrustDisplayState, TrustIntent, TrustLevelRow, TrustSummary};

fn to_color32(rgba: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

/// Trust Dashboard - visualizes trust relationships across the network
pub struct TrustDashboard {
    pub visible: bool,
    trust_summary: TrustSummary,
    last_update: std::time::Instant,
    display_state: TrustDisplayState,
}

impl Default for TrustDashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustDashboard {
    #[must_use]
    pub fn new() -> Self {
        let summary = TrustSummary::default();
        let display_state = compute::prepare_trust_display(&summary, 0);
        Self {
            visible: false,
            trust_summary: summary,
            last_update: std::time::Instant::now(),
            display_state,
        }
    }

    /// Read-only access to the display state (for headless testing).
    #[must_use]
    pub const fn display_state(&self) -> &TrustDisplayState {
        &self.display_state
    }

    /// Read-only access to the trust summary.
    #[must_use]
    pub const fn trust_summary(&self) -> &TrustSummary {
        &self.trust_summary
    }

    /// Update trust statistics from current primals and rebuild display state.
    pub fn update_from_primals(&mut self, primals: &[PrimalInfo]) {
        let mut summary = TrustSummary {
            trust_distribution: HashMap::new(),
            total_primals: primals.len(),
            family_count: 0,
            unique_families: 0,
            average_trust: None,
        };

        let mut trust_values = Vec::new();
        let mut families = std::collections::HashSet::new();

        for primal in primals {
            if let Some(trust_value) = primal.properties.get("trust_level") {
                let trust_label = match trust_value {
                    PropertyValue::Number(n) => {
                        trust_values.push(*n);
                        trust_level_number_to_label(n.round() as i32)
                    }
                    PropertyValue::String(s) => s.clone(),
                    _ => "Unknown".to_string(),
                };
                *summary.trust_distribution.entry(trust_label).or_insert(0) += 1;
            } else {
                #[expect(deprecated)]
                if let Some(trust_level) = primal.trust_level {
                    trust_values.push(f64::from(trust_level));
                    let trust_label = trust_level_number_to_label(i32::from(trust_level));
                    *summary.trust_distribution.entry(trust_label).or_insert(0) += 1;
                }
            }

            if let Some(family_value) = primal.properties.get("family_id") {
                if let PropertyValue::String(family_id) = family_value {
                    families.insert(family_id.clone());
                    summary.family_count += 1;
                }
            } else {
                #[expect(deprecated)]
                if let Some(family_id) = &primal.family_id {
                    families.insert(family_id.clone());
                    summary.family_count += 1;
                }
            }
        }

        if !trust_values.is_empty() {
            let sum: f64 = trust_values.iter().sum();
            summary.average_trust = Some(sum / trust_values.len() as f64);
        }

        summary.unique_families = families.len();

        self.trust_summary = summary;
        self.last_update = std::time::Instant::now();
        self.display_state = compute::prepare_trust_display(&self.trust_summary, 0);
    }

    /// Render the trust dashboard panel.
    ///
    /// Returns a list of intents produced by user interactions. The caller
    /// decides how to process them (e.g. play audio, update state).
    pub fn render(
        &mut self,
        ui: &mut Ui,
        palette: &ColorPalette,
        font_scale: f32,
        _audio_system: Option<&crate::audio::AudioSystemV2>,
    ) -> Vec<TrustIntent> {
        let elapsed = self.last_update.elapsed().as_secs();
        self.display_state = compute::prepare_trust_display(&self.trust_summary, elapsed);

        let mut intents = Vec::new();
        let ds = &self.display_state;

        ui.heading(
            RichText::new("🔐 Trust Dashboard")
                .size(16.0 * font_scale)
                .strong(),
        );
        ui.add_space(8.0);

        ui.label(
            RichText::new("Network Trust Distribution")
                .size(14.0 * font_scale)
                .strong(),
        );
        ui.add_space(4.0);

        if ds.total_primals == 0 {
            ui.label(
                RichText::new("No primals discovered yet")
                    .size(12.0 * font_scale)
                    .color(palette.text_dim),
            );
            return intents;
        }

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Total Primals:")
                    .size(12.0 * font_scale)
                    .color(palette.text_dim),
            );
            ui.label(
                RichText::new(format!("{}", ds.total_primals))
                    .size(12.0 * font_scale)
                    .strong(),
            );
        });

        ui.add_space(8.0);

        for row in &ds.rows {
            ui.horizontal(|ui| {
                ui.label(RichText::new(row.emoji).size(14.0 * font_scale));
                ui.label(
                    RichText::new(&row.label)
                        .size(12.0 * font_scale)
                        .color(to_color32(row.color)),
                );
                ui.label(
                    RichText::new(format!("{} ({:.0}%)", row.count, row.percentage))
                        .size(12.0 * font_scale)
                        .color(palette.text),
                );
            });
        }

        ui.add_space(12.0);

        if let Some(ref avg) = ds.average {
            ui.label(
                RichText::new("Average Trust Level")
                    .size(14.0 * font_scale)
                    .strong(),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new(avg.emoji).size(18.0 * font_scale));
                ui.label(
                    RichText::new(format!("{:.2}", avg.value))
                        .size(16.0 * font_scale)
                        .color(to_color32(avg.color))
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("({})", avg.label))
                        .size(12.0 * font_scale)
                        .color(palette.text_dim),
                );
            });

            if ui.small_button("🔊 Hear Trust Level").clicked() {
                intents.push(TrustIntent::PlayAudio {
                    sound: avg.sound_name.to_string(),
                });
            }

            ui.add_space(12.0);
        }

        ui.separator();
        ui.add_space(8.0);
        ui.label(
            RichText::new("🌳 Family Relationships")
                .size(14.0 * font_scale)
                .strong(),
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Primals with Family:")
                    .size(12.0 * font_scale)
                    .color(palette.text_dim),
            );
            ui.label(
                RichText::new(format!("{}", ds.family_count))
                    .size(12.0 * font_scale)
                    .strong(),
            );
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Unique Families:")
                    .size(12.0 * font_scale)
                    .color(palette.text_dim),
            );
            ui.label(
                RichText::new(format!("{}", ds.unique_families))
                    .size(12.0 * font_scale)
                    .strong(),
            );
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            RichText::new(&ds.last_update_label)
                .size(10.0 * font_scale)
                .color(palette.text_dim),
        );

        intents
    }

    /// Render a compact version in the top bar
    pub fn render_compact(&self, ui: &mut Ui, palette: &ColorPalette, font_scale: f32) {
        let ds = &self.display_state;

        ui.horizontal(|ui| {
            ui.label(RichText::new("🔐").size(14.0 * font_scale));

            if let Some(ref avg) = ds.average {
                ui.label(RichText::new(avg.emoji).size(12.0 * font_scale));
                ui.label(
                    RichText::new(format!("{:.1}", avg.value))
                        .size(12.0 * font_scale)
                        .color(to_color32(avg.color)),
                );
            } else {
                ui.label(
                    RichText::new("N/A")
                        .size(12.0 * font_scale)
                        .color(palette.text_dim),
                );
            }

            if ds.total_primals > 0 {
                ui.label(
                    RichText::new(format!("({}/{})", ds.family_count, ds.total_primals))
                        .size(10.0 * font_scale)
                        .color(palette.text_dim),
                );
            }
        });
    }
}
