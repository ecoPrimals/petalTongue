//! Trust Status Dashboard
//!
//! Provides rich visualization of trust relationships and status across the primal network.
//! This module leverages the universal adapter system to display trust information
//! in a way that works with ANY trust model (not just ecoPrimals).

use crate::accessibility::ColorPalette;
use crate::audio::AudioSystemV2;
use egui::{Color32, RichText, Ui};
use petal_tongue_core::{PrimalInfo, PropertyValue};
use std::collections::HashMap;

/// Trust Dashboard - visualizes trust relationships across the network
pub struct TrustDashboard {
    /// Show the trust panel
    pub visible: bool,
    /// Last trust summary data
    trust_summary: TrustSummary,
    /// Last update timestamp
    last_update: std::time::Instant,
}

/// Summary of trust information across the network
#[derive(Default, Clone)]
struct TrustSummary {
    /// Count of primals by trust level
    trust_distribution: HashMap<String, usize>,
    /// Total primals
    total_primals: usize,
    /// Count of primals with family relationships
    family_count: usize,
    /// Unique families
    unique_families: usize,
    /// Average trust level (if numeric)
    average_trust: Option<f64>,
}

impl Default for TrustDashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustDashboard {
    /// Create a new trust dashboard
    #[must_use]
    pub fn new() -> Self {
        Self {
            visible: false,
            trust_summary: TrustSummary::default(),
            last_update: std::time::Instant::now(),
        }
    }

    /// Update trust statistics from current primals
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
            // Count trust levels
            if let Some(trust_value) = primal.properties.get("trust_level") {
                let trust_label = match trust_value {
                    PropertyValue::Number(n) => {
                        trust_values.push(*n);
                        match n.round() as i32 {
                            0 => "None (0)".to_string(),
                            1 => "Limited (1)".to_string(),
                            2 => "Elevated (2)".to_string(),
                            3 => "Full (3)".to_string(),
                            _ => format!("Unknown ({n})"),
                        }
                    }
                    PropertyValue::String(s) => s.clone(),
                    _ => "Unknown".to_string(),
                };
                *summary.trust_distribution.entry(trust_label).or_insert(0) += 1;
            } else {
                // Check deprecated field for backward compatibility
                #[allow(deprecated)]
                if let Some(trust_level) = primal.trust_level {
                    trust_values.push(f64::from(trust_level));
                    let trust_label = match trust_level {
                        0 => "None (0)".to_string(),
                        1 => "Limited (1)".to_string(),
                        2 => "Elevated (2)".to_string(),
                        3 => "Full (3)".to_string(),
                        _ => format!("Unknown ({trust_level})"),
                    };
                    *summary.trust_distribution.entry(trust_label).or_insert(0) += 1;
                }
            }

            // Count families
            if let Some(family_value) = primal.properties.get("family_id") {
                if let PropertyValue::String(family_id) = family_value {
                    families.insert(family_id.clone());
                    summary.family_count += 1;
                }
            } else {
                // Check deprecated field for backward compatibility
                #[allow(deprecated)]
                if let Some(family_id) = &primal.family_id {
                    families.insert(family_id.clone());
                    summary.family_count += 1;
                }
            }
        }

        // Calculate average trust
        if !trust_values.is_empty() {
            let sum: f64 = trust_values.iter().sum();
            summary.average_trust = Some(sum / trust_values.len() as f64);
        }

        summary.unique_families = families.len();

        self.trust_summary = summary;
        self.last_update = std::time::Instant::now();
    }

    /// Render the trust dashboard panel
    pub fn render(
        &mut self,
        ui: &mut Ui,
        palette: &ColorPalette,
        font_scale: f32,
        audio_system: Option<&AudioSystemV2>,
    ) {
        ui.heading(
            RichText::new("🔐 Trust Dashboard")
                .size(16.0 * font_scale)
                .strong(),
        );
        ui.add_space(8.0);

        // Trust distribution section
        ui.label(
            RichText::new("Network Trust Distribution")
                .size(14.0 * font_scale)
                .strong(),
        );
        ui.add_space(4.0);

        if self.trust_summary.total_primals == 0 {
            ui.label(
                RichText::new("No primals discovered yet")
                    .size(12.0 * font_scale)
                    .color(palette.text_dim),
            );
            return;
        }

        // Total primals
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Total Primals:")
                    .size(12.0 * font_scale)
                    .color(palette.text_dim),
            );
            ui.label(
                RichText::new(format!("{}", self.trust_summary.total_primals))
                    .size(12.0 * font_scale)
                    .strong(),
            );
        });

        ui.add_space(8.0);

        // Trust level breakdown
        let mut trust_levels: Vec<_> = self.trust_summary.trust_distribution.iter().collect();
        trust_levels.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count (descending)

        for (level, count) in trust_levels {
            let percentage = (*count as f32 / self.trust_summary.total_primals as f32) * 100.0;

            // Determine color based on trust level
            let (emoji, color) = if level.contains("Full") || level.contains("(3)") {
                ("🟢", Color32::from_rgb(76, 175, 80))
            } else if level.contains("Elevated") || level.contains("(2)") {
                ("🟠", Color32::from_rgb(255, 152, 0))
            } else if level.contains("Limited") || level.contains("(1)") {
                ("🟡", Color32::from_rgb(255, 235, 59))
            } else {
                ("⚫", Color32::from_rgb(158, 158, 158))
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(emoji).size(14.0 * font_scale));
                ui.label(RichText::new(level).size(12.0 * font_scale).color(color));
                ui.label(
                    RichText::new(format!("{count} ({percentage:.0}%)"))
                        .size(12.0 * font_scale)
                        .color(palette.text),
                );
            });
        }

        ui.add_space(12.0);

        // Average trust (if available)
        if let Some(avg) = self.trust_summary.average_trust {
            ui.label(
                RichText::new("Average Trust Level")
                    .size(14.0 * font_scale)
                    .strong(),
            );
            ui.add_space(4.0);

            let (emoji, color, label) = match avg.round() as i32 {
                0 => ("⚫", Color32::from_rgb(158, 158, 158), "None"),
                1 => ("🟡", Color32::from_rgb(255, 235, 59), "Limited"),
                2 => ("🟠", Color32::from_rgb(255, 152, 0), "Elevated"),
                3 => ("🟢", Color32::from_rgb(76, 175, 80), "Full"),
                _ => ("❓", palette.text_dim, "Unknown"),
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(emoji).size(18.0 * font_scale));
                ui.label(
                    RichText::new(format!("{avg:.2}"))
                        .size(16.0 * font_scale)
                        .color(color)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("({label})"))
                        .size(12.0 * font_scale)
                        .color(palette.text_dim),
                );
            });

            // Play audio cue if audio is enabled
            if let Some(audio) = audio_system
                && ui.small_button("🔊 Hear Trust Level").clicked()
            {
                // Play notification based on trust level
                let sound = match avg.round() as i32 {
                    3 => "success",
                    2 => "notification",
                    1 => "warning",
                    0 => "error",
                    _ => "notification",
                };
                let _ = audio.play(sound);
            }

            ui.add_space(12.0);
        }

        // Family relationships section
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
                RichText::new(format!("{}", self.trust_summary.family_count))
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
                RichText::new(format!("{}", self.trust_summary.unique_families))
                    .size(12.0 * font_scale)
                    .strong(),
            );
        });

        ui.add_space(12.0);

        // Last update timestamp
        ui.separator();
        ui.add_space(4.0);
        let elapsed = self.last_update.elapsed().as_secs();
        ui.label(
            RichText::new(format!("Updated {elapsed} seconds ago"))
                .size(10.0 * font_scale)
                .color(palette.text_dim),
        );
    }

    /// Render a compact version in the top bar
    pub fn render_compact(&self, ui: &mut Ui, palette: &ColorPalette, font_scale: f32) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔐").size(14.0 * font_scale));

            if let Some(avg) = self.trust_summary.average_trust {
                let (emoji, color) = match avg.round() as i32 {
                    0 => ("⚫", Color32::from_rgb(158, 158, 158)),
                    1 => ("🟡", Color32::from_rgb(255, 235, 59)),
                    2 => ("🟠", Color32::from_rgb(255, 152, 0)),
                    3 => ("🟢", Color32::from_rgb(76, 175, 80)),
                    _ => ("❓", palette.text_dim),
                };

                ui.label(RichText::new(emoji).size(12.0 * font_scale));
                ui.label(
                    RichText::new(format!("{avg:.1}"))
                        .size(12.0 * font_scale)
                        .color(color),
                );
            } else {
                ui.label(
                    RichText::new("N/A")
                        .size(12.0 * font_scale)
                        .color(palette.text_dim),
                );
            }

            if self.trust_summary.total_primals > 0 {
                ui.label(
                    RichText::new(format!(
                        "({}/{})",
                        self.trust_summary.family_count, self.trust_summary.total_primals
                    ))
                    .size(10.0 * font_scale)
                    .color(palette.text_dim),
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{PrimalHealthStatus, Properties};

    fn create_test_primal(id: &str, trust: Option<u8>, family: Option<&str>) -> PrimalInfo {
        let mut props = Properties::new();
        if let Some(t) = trust {
            props.insert("trust_level".to_string(), PropertyValue::Number(t as f64));
        }
        if let Some(f) = family {
            props.insert(
                "family_id".to_string(),
                PropertyValue::String(f.to_string()),
            );
        }

        PrimalInfo {
            id: id.to_string(),
            name: format!("Test Primal {}", id),
            primal_type: "Test".to_string(),
            endpoint: "http://test".to_string(),
            capabilities: vec![],
            health: PrimalHealthStatus::Healthy,
            last_seen: 0,
            endpoints: None,
            metadata: None,
            properties: props,
            #[allow(deprecated)]
            trust_level: trust,
            #[allow(deprecated)]
            family_id: family.map(String::from),
        }
    }

    #[test]
    fn test_trust_dashboard_creation() {
        let dashboard = TrustDashboard::new();
        assert!(!dashboard.visible);
        assert_eq!(dashboard.trust_summary.total_primals, 0);
    }

    #[test]
    fn test_update_from_primals() {
        let mut dashboard = TrustDashboard::new();

        let primals = vec![
            create_test_primal("p1", Some(3), Some("family-a")),
            create_test_primal("p2", Some(2), Some("family-a")),
            create_test_primal("p3", Some(1), Some("family-b")),
            create_test_primal("p4", Some(0), None),
        ];

        dashboard.update_from_primals(&primals);

        assert_eq!(dashboard.trust_summary.total_primals, 4);
        assert_eq!(dashboard.trust_summary.family_count, 3);
        assert_eq!(dashboard.trust_summary.unique_families, 2);
        assert!(dashboard.trust_summary.average_trust.is_some());
        assert!((dashboard.trust_summary.average_trust.unwrap() - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_trust_distribution() {
        let mut dashboard = TrustDashboard::new();

        let primals = vec![
            create_test_primal("p1", Some(3), None),
            create_test_primal("p2", Some(3), None),
            create_test_primal("p3", Some(2), None),
            create_test_primal("p4", Some(1), None),
        ];

        dashboard.update_from_primals(&primals);

        assert_eq!(
            dashboard.trust_summary.trust_distribution.get("Full (3)"),
            Some(&2)
        );
        assert_eq!(
            dashboard
                .trust_summary
                .trust_distribution
                .get("Elevated (2)"),
            Some(&1)
        );
        assert_eq!(
            dashboard
                .trust_summary
                .trust_distribution
                .get("Limited (1)"),
            Some(&1)
        );
    }

    #[test]
    fn test_empty_primals() {
        let mut dashboard = TrustDashboard::new();
        dashboard.update_from_primals(&[]);

        assert_eq!(dashboard.trust_summary.total_primals, 0);
        assert_eq!(dashboard.trust_summary.family_count, 0);
        assert_eq!(dashboard.trust_summary.unique_families, 0);
        assert!(dashboard.trust_summary.average_trust.is_none());
    }
}
