// SPDX-License-Identifier: AGPL-3.0-only
//! ecoPrimals-specific capability icon adapter
//!
//! Maps capability strings to emoji icons. Configuration comes from
//! ecosystem discovery in the future.

use crate::adapter_trait::PropertyAdapter;
use egui::Ui;
use petal_tongue_core::property::{Properties, PropertyValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for capability icon mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityIconConfig {
    /// Map from capability keyword to emoji
    pub icon_map: HashMap<String, String>,
    /// Default icon for unknown capabilities
    pub default_icon: String,
}

impl Default for CapabilityIconConfig {
    fn default() -> Self {
        let mut icon_map = HashMap::new();

        // ecoPrimals default mappings
        icon_map.insert("security".to_string(), "🔒".to_string());
        icon_map.insert("storage".to_string(), "💾".to_string());
        icon_map.insert("ai".to_string(), "🧠".to_string());
        icon_map.insert("compute".to_string(), "⚡".to_string());
        icon_map.insert("network".to_string(), "🌐".to_string());
        icon_map.insert("database".to_string(), "🗄️".to_string());
        icon_map.insert("visualization".to_string(), "📊".to_string());
        icon_map.insert("audio".to_string(), "🔊".to_string());
        icon_map.insert("discovery".to_string(), "🔍".to_string());
        icon_map.insert("trust".to_string(), "🤝".to_string());
        icon_map.insert("entropy".to_string(), "🎲".to_string());

        Self {
            icon_map,
            default_icon: "⚙️".to_string(),
        }
    }
}

/// ecoPrimals capability icon adapter
///
/// Maps capability strings to emojis for visual representation.
pub struct EcoPrimalCapabilityAdapter {
    config: CapabilityIconConfig,
}

impl EcoPrimalCapabilityAdapter {
    /// Create adapter with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::from_config(CapabilityIconConfig::default())
    }

    /// Create adapter from configuration
    #[must_use]
    pub fn from_config(config: CapabilityIconConfig) -> Self {
        Self { config }
    }

    /// Get icon for a capability string
    fn get_icon(&self, capability: &str) -> &str {
        // Try exact match first
        if let Some(icon) = self.config.icon_map.get(capability) {
            return icon;
        }

        // Try substring matches (e.g., "ai/model" matches "ai")
        let capability_lower = capability.to_lowercase();
        for (keyword, icon) in &self.config.icon_map {
            if capability_lower.contains(&keyword.to_lowercase()) {
                return icon;
            }
        }

        // Default
        &self.config.default_icon
    }
}

impl Default for EcoPrimalCapabilityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAdapter for EcoPrimalCapabilityAdapter {
    fn name(&self) -> &'static str {
        "ecoprimal-capabilities"
    }

    fn handles(&self, property_key: &str) -> bool {
        property_key == "capabilities"
    }

    fn render(&self, _key: &str, value: &PropertyValue, ui: &mut Ui) {
        if let Some(capabilities) = value.as_array() {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(format!("Capabilities ({})", capabilities.len())).strong(),
                );

                ui.spacing_mut().item_spacing.y = 4.0;

                for cap in capabilities {
                    if let Some(cap_str) = cap.as_string() {
                        ui.horizontal(|ui| {
                            ui.label(self.get_icon(cap_str));
                            ui.label(cap_str);
                        });
                    }
                }
            });
        } else {
            ui.label("Invalid capabilities value");
        }
    }

    fn node_decoration(
        &self,
        _properties: &Properties,
    ) -> Option<crate::adapter_trait::NodeDecoration> {
        // Capabilities don't provide node decoration
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
    fn test_capability_adapter_handles() {
        let adapter = EcoPrimalCapabilityAdapter::new();
        assert!(adapter.handles("capabilities"));
        assert!(!adapter.handles("other_property"));
    }

    #[test]
    fn test_get_icon_exact_match() {
        let adapter = EcoPrimalCapabilityAdapter::new();
        assert_eq!(adapter.get_icon("security"), "🔒");
        assert_eq!(adapter.get_icon("storage"), "💾");
        assert_eq!(adapter.get_icon("ai"), "🧠");
    }

    #[test]
    fn test_get_icon_substring_match() {
        let adapter = EcoPrimalCapabilityAdapter::new();
        assert_eq!(adapter.get_icon("security/encryption"), "🔒");
        assert_eq!(adapter.get_icon("ai/model"), "🧠");
        assert_eq!(adapter.get_icon("storage/persistent"), "💾");
    }

    #[test]
    fn test_get_icon_default() {
        let adapter = EcoPrimalCapabilityAdapter::new();
        assert_eq!(adapter.get_icon("unknown-capability"), "⚙️");
    }

    #[test]
    fn test_custom_config() {
        let mut config = CapabilityIconConfig::default();
        config
            .icon_map
            .insert("custom".to_string(), "🎯".to_string());

        let adapter = EcoPrimalCapabilityAdapter::from_config(config);
        assert_eq!(adapter.get_icon("custom"), "🎯");
    }
}
