// SPDX-License-Identifier: AGPL-3.0-or-later
//! Property adapter trait for ecosystem-specific rendering
//!
//! This trait allows ecosystem-specific adapters to interpret and render
//! generic properties. petalTongue core has zero knowledge of what properties
//! mean - adapters provide that knowledge at runtime.

use egui::{Color32, Ui};
use petal_tongue_core::property::{Properties, PropertyValue};

/// Visual decoration that an adapter can provide for a node
#[derive(Debug, Clone)]
pub struct NodeDecoration {
    /// Optional emoji badge to show on node
    pub badge: Option<String>,
    /// Optional color to fill node with
    pub fill_color: Option<Color32>,
    /// Optional color for node ring/border
    pub ring_color: Option<Color32>,
    /// Optional text to show as tooltip
    pub tooltip: Option<String>,
}

/// Trait for adapters that know how to render ecosystem-specific properties
///
/// # Philosophy
///
/// petalTongue core is universal and knows nothing about specific ecosystems.
/// Adapters bridge this gap by:
/// 1. Declaring which property keys they handle
/// 2. Rendering those properties with ecosystem-specific interface
/// 3. Providing visual decorations (badges, colors) for nodes
/// 4. Getting configuration FROM the ecosystem, not hardcoded
///
/// # Example
///
/// ```ignore
/// struct TrustAdapter {
///     level_names: Vec<String>,  // From ecosystem
///     level_colors: Vec<Color32>, // From ecosystem
/// }
///
/// impl PropertyAdapter for TrustAdapter {
///     fn handles(&self, key: &str) -> bool {
///         key == "trust_level"
///     }
///
///     fn render(&self, key: &str, value: &PropertyValue, ui: &mut Ui) {
///         if let Some(level) = value.as_u8() {
///             let name = &self.level_names[level as usize];
///             let color = self.level_colors[level as usize];
///             ui.colored_label(color, name);
///         }
///     }
/// }
/// ```
pub trait PropertyAdapter: Send + Sync {
    /// Name of this adapter (for debugging/logging)
    fn name(&self) -> &str;

    /// Check if this adapter handles a given property key
    ///
    /// Returns true if this adapter knows how to render this property.
    fn handles(&self, property_key: &str) -> bool;

    /// Render a property in the interface
    ///
    /// Called when presenting node details. The adapter can use any
    /// egui widgets to render the property value.
    fn render(&self, property_key: &str, value: &PropertyValue, ui: &mut Ui);

    /// Provide visual decoration for a node based on its properties
    ///
    /// This is called when rendering nodes in the graph. Adapters can
    /// return badges, colors, or tooltips to enhance the visualization.
    fn node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
        // Default: no decoration
        let _ = properties;
        None
    }

    /// Priority for this adapter (higher = checked first)
    ///
    /// Useful when multiple adapters might handle the same key.
    fn priority(&self) -> i32 {
        0
    }
}

/// Type-erased adapter for storage in collections
pub type BoxedAdapter = Box<dyn PropertyAdapter>;

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAdapter;

    impl PropertyAdapter for TestAdapter {
        fn name(&self) -> &'static str {
            "test"
        }

        fn handles(&self, property_key: &str) -> bool {
            property_key == "test_prop"
        }

        fn render(&self, _key: &str, value: &PropertyValue, ui: &mut Ui) {
            if let Some(s) = value.as_string() {
                ui.label(s);
            }
        }
    }

    #[test]
    fn test_adapter_handles() {
        let adapter = TestAdapter;
        assert!(adapter.handles("test_prop"));
        assert!(!adapter.handles("other_prop"));
    }

    #[test]
    fn test_adapter_name() {
        let adapter = TestAdapter;
        assert_eq!(adapter.name(), "test");
    }

    #[test]
    fn test_default_decoration() {
        let adapter = TestAdapter;
        let props = Properties::new();
        assert!(adapter.node_decoration(&props).is_none());
    }

    #[test]
    fn test_default_priority() {
        let adapter = TestAdapter;
        assert_eq!(adapter.priority(), 0);
    }

    struct PriorityAdapter;

    impl PropertyAdapter for PriorityAdapter {
        fn name(&self) -> &'static str {
            "priority"
        }

        fn handles(&self, key: &str) -> bool {
            key == "priority_prop"
        }

        fn render(&self, _key: &str, _value: &PropertyValue, _ui: &mut Ui) {}
    }

    #[test]
    fn test_adapter_priority_override() {
        let adapter = PriorityAdapter;
        assert_eq!(adapter.priority(), 0);
    }

    #[test]
    fn test_node_decoration_default_with_props() {
        let adapter = TestAdapter;
        let mut props = Properties::new();
        props.insert("key".to_string(), PropertyValue::String("val".to_string()));
        assert!(adapter.node_decoration(&props).is_none());
    }
}
