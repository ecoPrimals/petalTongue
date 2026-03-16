// SPDX-License-Identifier: AGPL-3.0-or-later
//! ecoPrimals-specific trust adapter
//!
//! This adapter knows how to render trust levels from the ecoPrimals ecosystem.
//! Configuration comes FROM the ecosystem (via capability discovery), not hardcoded.

use crate::adapter_trait::{NodeDecoration, PropertyAdapter};
use egui::{Color32, Ui};
use petal_tongue_core::property::{Properties, PropertyValue};
use serde::{Deserialize, Serialize};

/// Configuration for trust level rendering (from ecosystem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    /// Minimum trust level (inclusive)
    pub min_level: u8,
    /// Maximum trust level (inclusive)
    pub max_level: u8,
    /// Human-readable names for each trust level
    pub level_names: Vec<String>,
    /// Emoji indicators for each trust level
    pub level_emojis: Vec<String>,
    /// Hex color strings (e.g. `"#808080"`) for each trust level
    pub level_colors: Vec<String>,
}

impl Default for TrustConfig {
    fn default() -> Self {
        // Default config for ecoPrimals (until we have discovery)
        Self {
            min_level: 0,
            max_level: 3,
            level_names: vec![
                "None".to_string(),
                "Limited".to_string(),
                "Elevated".to_string(),
                "Full".to_string(),
            ],
            level_emojis: vec![
                "⚫".to_string(),
                "🟡".to_string(),
                "🟠".to_string(),
                "🟢".to_string(),
            ],
            level_colors: vec![
                "#808080".to_string(), // Gray
                "#FFD700".to_string(), // Gold
                "#FF8C00".to_string(), // Dark orange
                "#32CD32".to_string(), // Lime green
            ],
        }
    }
}

/// ecoPrimals trust level adapter
///
/// Renders trust levels with emojis, names, and colors.
/// Configuration comes from the ecosystem's trust-management capability spec.
pub struct EcoPrimalTrustAdapter {
    config: TrustConfig,
    colors: Vec<Color32>, // Parsed from hex
}

impl EcoPrimalTrustAdapter {
    /// Create adapter with default configuration
    ///
    /// This is temporary - in the future, config will come from ecosystem discovery
    #[must_use]
    pub fn new() -> Self {
        Self::from_config(TrustConfig::default())
    }

    /// Create adapter from trust configuration
    #[must_use]
    pub fn from_config(config: TrustConfig) -> Self {
        // Parse hex colors to egui Color32
        let colors = config
            .level_colors
            .iter()
            .map(|hex| parse_hex_color(hex).unwrap_or(Color32::GRAY))
            .collect();

        Self { config, colors }
    }

    /// Create from ecosystem capability spec (future)
    pub fn from_capability_spec(spec: &serde_json::Value) -> Option<Self> {
        // Parse spec into TrustConfig
        serde_json::from_value(spec.clone())
            .ok()
            .map(Self::from_config)
    }

    const fn get_level_index(&self, level: u8) -> Option<usize> {
        if level >= self.config.min_level && level <= self.config.max_level {
            Some((level - self.config.min_level) as usize)
        } else {
            None
        }
    }
}

impl Default for EcoPrimalTrustAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAdapter for EcoPrimalTrustAdapter {
    fn name(&self) -> &'static str {
        "ecoprimal-trust"
    }

    fn handles(&self, property_key: &str) -> bool {
        property_key == "trust_level"
    }

    fn render(&self, _key: &str, value: &PropertyValue, ui: &mut Ui) {
        if let Some(level) = value.as_u8() {
            if let Some(idx) = self.get_level_index(level) {
                ui.horizontal(|ui| {
                    // Emoji
                    ui.label(&self.config.level_emojis[idx]);

                    // Name with color
                    ui.colored_label(self.colors[idx], &self.config.level_names[idx]);

                    // Level number (subtle)
                    ui.label(
                        egui::RichText::new(format!("({level})"))
                            .color(Color32::DARK_GRAY)
                            .small(),
                    );
                });
            } else {
                ui.label(format!("Unknown trust level: {level}"));
            }
        } else {
            ui.label("Invalid trust level value");
        }
    }

    fn node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
        if let Some(PropertyValue::Number(level)) = properties.get("trust_level") {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "trust_level is 0-5, clamped before cast"
            )]
            let level = level.clamp(0.0, 255.0) as u8;
            if let Some(idx) = self.get_level_index(level) {
                return Some(NodeDecoration {
                    badge: Some(self.config.level_emojis[idx].clone()),
                    fill_color: Some(self.colors[idx]),
                    ring_color: None,
                    tooltip: Some(format!(
                        "Trust: {} ({})",
                        self.config.level_names[idx], level
                    )),
                });
            }
        }
        None
    }

    fn priority(&self) -> i32 {
        10 // Higher priority than generic
    }
}

/// Parse hex color string to Color32
fn parse_hex_color(hex: &str) -> Option<Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color32::from_rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_adapter_handles() {
        let adapter = EcoPrimalTrustAdapter::new();
        assert!(adapter.handles("trust_level"));
        assert!(!adapter.handles("other_property"));
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_hex_color("#FF0000").unwrap();
        assert_eq!(color, Color32::from_rgb(255, 0, 0));

        let color = parse_hex_color("#00FF00").unwrap();
        assert_eq!(color, Color32::from_rgb(0, 255, 0));

        let color = parse_hex_color("#0000FF").unwrap();
        assert_eq!(color, Color32::from_rgb(0, 0, 255));
    }

    #[test]
    fn test_get_level_index() {
        let adapter = EcoPrimalTrustAdapter::new();
        assert_eq!(adapter.get_level_index(0), Some(0));
        assert_eq!(adapter.get_level_index(1), Some(1));
        assert_eq!(adapter.get_level_index(2), Some(2));
        assert_eq!(adapter.get_level_index(3), Some(3));
        assert_eq!(adapter.get_level_index(4), None);
    }

    #[test]
    fn test_node_decoration() {
        let adapter = EcoPrimalTrustAdapter::new();
        let mut props = Properties::new();
        props.insert("trust_level".to_string(), PropertyValue::Number(2.0));

        let decoration = adapter.node_decoration(&props).unwrap();
        assert_eq!(decoration.badge, Some("🟠".to_string()));
        assert!(decoration.tooltip.is_some());
    }

    #[test]
    fn test_from_config() {
        let config = TrustConfig {
            min_level: 0,
            max_level: 2,
            level_names: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            level_emojis: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            level_colors: vec![
                "#FF0000".to_string(),
                "#00FF00".to_string(),
                "#0000FF".to_string(),
            ],
        };
        let adapter = EcoPrimalTrustAdapter::from_config(config);
        assert_eq!(adapter.get_level_index(0), Some(0));
        assert_eq!(adapter.get_level_index(2), Some(2));
        assert_eq!(adapter.get_level_index(3), None);
    }

    #[test]
    fn test_from_capability_spec_valid() {
        let spec = serde_json::json!({
            "min_level": 0,
            "max_level": 1,
            "level_names": ["Low", "High"],
            "level_emojis": ["L", "H"],
            "level_colors": ["#111111", "#222222"]
        });
        let adapter = EcoPrimalTrustAdapter::from_capability_spec(&spec).unwrap();
        assert!(adapter.handles("trust_level"));
    }

    #[test]
    fn test_from_capability_spec_invalid() {
        let spec = serde_json::json!({"invalid": true});
        assert!(EcoPrimalTrustAdapter::from_capability_spec(&spec).is_none());
    }

    #[test]
    fn test_parse_hex_color_with_hash() {
        let color = parse_hex_color("#FFFFFF").unwrap();
        assert_eq!(color, Color32::from_rgb(255, 255, 255));
    }

    #[test]
    fn test_parse_hex_color_without_hash() {
        let color = parse_hex_color("FF00FF").unwrap();
        assert_eq!(color, Color32::from_rgb(255, 0, 255));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert!(parse_hex_color("").is_none());
        assert!(parse_hex_color("FF").is_none());
        assert!(parse_hex_color("GGGGGG").is_none());
    }

    #[test]
    fn test_node_decoration_out_of_range() {
        let adapter = EcoPrimalTrustAdapter::new();
        let mut props = Properties::new();
        props.insert("trust_level".to_string(), PropertyValue::Number(99.0));
        assert!(adapter.node_decoration(&props).is_none());
    }

    #[test]
    fn test_node_decoration_clamped() {
        let adapter = EcoPrimalTrustAdapter::new();
        let mut props = Properties::new();
        props.insert("trust_level".to_string(), PropertyValue::Number(1.5));
        let decoration = adapter.node_decoration(&props).unwrap();
        assert_eq!(decoration.badge, Some("🟡".to_string()));
    }

    #[test]
    fn test_render_invalid_trust_level() {
        let adapter = EcoPrimalTrustAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("trust_level", &PropertyValue::String("bad".to_string()), ui);
            });
        });
    }

    #[test]
    fn test_render_unknown_level() {
        let adapter = EcoPrimalTrustAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("trust_level", &PropertyValue::Number(10.0), ui);
            });
        });
    }

    #[test]
    fn test_render_valid_level() {
        let adapter = EcoPrimalTrustAdapter::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                adapter.render("trust_level", &PropertyValue::Number(3.0), ui);
            });
        });
    }

    #[test]
    fn test_default_config() {
        let config = TrustConfig::default();
        assert_eq!(config.min_level, 0);
        assert_eq!(config.max_level, 3);
        assert_eq!(config.level_names.len(), 4);
    }

    #[test]
    fn test_adapter_name_priority() {
        let adapter = EcoPrimalTrustAdapter::new();
        assert_eq!(adapter.name(), "ecoprimal-trust");
        assert_eq!(adapter.priority(), 10);
    }

    #[test]
    fn test_config_invalid_hex_fallback() {
        let config = TrustConfig {
            min_level: 0,
            max_level: 0,
            level_names: vec!["X".to_string()],
            level_emojis: vec!["X".to_string()],
            level_colors: vec!["invalid".to_string()],
        };
        let adapter = EcoPrimalTrustAdapter::from_config(config);
        let mut props = Properties::new();
        props.insert("trust_level".to_string(), PropertyValue::Number(0.0));
        let decoration = adapter.node_decoration(&props).unwrap();
        assert!(decoration.fill_color.is_some());
    }
}
