// SPDX-License-Identifier: AGPL-3.0-only
//! ecoPrimals-specific family/lineage adapter
//!
//! This adapter knows how to render genetic lineage (`family_id`, DNA) from
//! the ecoPrimals ecosystem.

use crate::adapter_trait::{NodeDecoration, PropertyAdapter};
use egui::{Color32, Ui};
use petal_tongue_core::property::{Properties, PropertyValue};

/// ecoPrimals family lineage adapter
///
/// Renders `family_id` with ring colors and DNA information.
/// In the future, this will get configuration from the ecosystem's
/// genetic-lineage capability spec.
pub struct EcoPrimalFamilyAdapter {
    // Future: configuration from ecosystem
}

impl EcoPrimalFamilyAdapter {
    /// Create adapter
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Generate a deterministic color from `family_id` string
    fn family_id_to_color(family_id: &str) -> Color32 {
        // Hash the family_id to get a consistent color
        let hash = family_id.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(u32::from(b))
        });

        // Generate RGB from hash
        let r = ((hash >> 16) & 0xFF) as u8;
        let g = ((hash >> 8) & 0xFF) as u8;
        let b = (hash & 0xFF) as u8;

        // Ensure color is visible (not too dark)
        let min_brightness = 80;
        let r = r.max(min_brightness);
        let g = g.max(min_brightness);
        let b = b.max(min_brightness);

        Color32::from_rgb(r, g, b)
    }
}

impl Default for EcoPrimalFamilyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAdapter for EcoPrimalFamilyAdapter {
    fn name(&self) -> &'static str {
        "ecoprimal-family"
    }

    fn handles(&self, property_key: &str) -> bool {
        property_key == "family_id" || property_key == "dna"
    }

    fn render(&self, key: &str, value: &PropertyValue, ui: &mut Ui) {
        match key {
            "family_id" => {
                if let Some(family_id) = value.as_string() {
                    let color = Self::family_id_to_color(family_id);

                    ui.horizontal(|ui| {
                        // Colored dot representing family
                        ui.label(egui::RichText::new("●").color(color).size(16.0));

                        // Family ID
                        ui.label(egui::RichText::new(family_id).color(color));

                        // Copy button
                        if ui.small_button("📋").clicked() {
                            ui.output_mut(|o| o.copied_text = family_id.to_string());
                        }
                    });
                } else {
                    ui.label("Invalid family_id");
                }
            }
            "dna" => {
                if let Some(dna) = value.as_string() {
                    ui.horizontal(|ui| {
                        ui.label("🧬");
                        ui.label(egui::RichText::new(dna).monospace().small());

                        // Copy button
                        if ui.small_button("📋").clicked() {
                            ui.output_mut(|o| o.copied_text = dna.to_string());
                        }
                    });
                } else {
                    ui.label("Invalid DNA");
                }
            }
            _ => {}
        }
    }

    fn node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
        if let Some(PropertyValue::String(family_id)) = properties.get("family_id") {
            let color = Self::family_id_to_color(family_id);
            return Some(NodeDecoration {
                badge: None,
                fill_color: None,
                ring_color: Some(color), // Ring color shows family
                tooltip: Some(format!("Family: {family_id}")),
            });
        }
        None
    }

    fn priority(&self) -> i32 {
        10 // Higher priority than generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_adapter_handles() {
        let adapter = EcoPrimalFamilyAdapter::new();
        assert!(adapter.handles("family_id"));
        assert!(adapter.handles("dna"));
        assert!(!adapter.handles("other_property"));
    }

    #[test]
    fn test_family_id_to_color() {
        let color1 = EcoPrimalFamilyAdapter::family_id_to_color("test123");
        let color2 = EcoPrimalFamilyAdapter::family_id_to_color("test123");
        let color3 = EcoPrimalFamilyAdapter::family_id_to_color("different");

        // Same input = same color
        assert_eq!(color1, color2);

        // Different input = different color
        assert_ne!(color1, color3);
    }

    #[test]
    fn test_node_decoration() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let mut props = Properties::new();
        props.insert(
            "family_id".to_string(),
            PropertyValue::String("family-abc".to_string()),
        );

        let decoration = adapter.node_decoration(&props).unwrap();
        assert!(decoration.ring_color.is_some());
        assert!(decoration.tooltip.is_some());
    }

    #[test]
    fn test_node_decoration_no_family_id() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let props = Properties::new();
        assert!(adapter.node_decoration(&props).is_none());
    }

    #[test]
    fn test_adapter_priority() {
        let adapter = EcoPrimalFamilyAdapter::new();
        assert_eq!(adapter.priority(), 10);
    }

    #[test]
    fn test_adapter_name() {
        let adapter = EcoPrimalFamilyAdapter::new();
        assert_eq!(adapter.name(), "ecoprimal-family");
    }

    #[test]
    fn test_default_impl() {
        let adapter = EcoPrimalFamilyAdapter::default();
        assert!(adapter.handles("family_id"));
    }

    #[test]
    fn test_node_decoration_family_id_as_number() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let mut props = Properties::new();
        props.insert("family_id".to_string(), PropertyValue::Number(42.0));
        assert!(adapter.node_decoration(&props).is_none());
    }

    #[test]
    fn test_node_decoration_family_id_missing() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let mut props = Properties::new();
        props.insert(
            "other_key".to_string(),
            PropertyValue::String("val".to_string()),
        );
        assert!(adapter.node_decoration(&props).is_none());
    }

    #[test]
    fn test_node_decoration_empty_family_id() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let mut props = Properties::new();
        props.insert(
            "family_id".to_string(),
            PropertyValue::String(String::new()),
        );
        let decoration = adapter.node_decoration(&props).unwrap();
        assert!(decoration.ring_color.is_some());
    }

    #[test]
    fn test_family_id_to_color_empty_string() {
        let color = EcoPrimalFamilyAdapter::family_id_to_color("");
        assert_eq!(color, Color32::from_rgb(80, 80, 80));
    }

    #[test]
    fn test_family_id_to_color_min_brightness() {
        let color = EcoPrimalFamilyAdapter::family_id_to_color("a");
        let (r, g, b, _) = color.to_tuple();
        assert!(r >= 80);
        assert!(g >= 80);
        assert!(b >= 80);
    }

    #[test]
    fn test_render_family_id_invalid_value() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("family_id", &PropertyValue::Number(1.0), ui);
            });
        });
    }

    #[test]
    fn test_render_dna_invalid_value() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("dna", &PropertyValue::Boolean(true), ui);
            });
        });
    }

    #[test]
    fn test_render_unknown_key_no_op() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("other", &PropertyValue::String("x".to_string()), ui);
            });
        });
    }

    #[test]
    fn test_render_family_id_valid() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render(
                    "family_id",
                    &PropertyValue::String("fam-xyz".to_string()),
                    ui,
                );
            });
        });
    }

    #[test]
    fn test_render_dna_valid() {
        let adapter = EcoPrimalFamilyAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("dna", &PropertyValue::String("ACTG".to_string()), ui);
            });
        });
    }
}
