//! Adapter registry for runtime composition of ecosystem-specific UI
//!
//! The registry holds adapters and routes property rendering to the
//! appropriate adapter. If no adapter handles a property, it falls back
//! to generic key-value display.

use crate::adapter_trait::{BoxedAdapter, NodeDecoration};
use egui::{Color32, Ui};
use petal_tongue_core::property::{Properties, PropertyValue};
use std::sync::{Arc, RwLock};

/// Registry of property adapters
///
/// This is the central router that:
/// 1. Holds all registered adapters
/// 2. Finds the right adapter for each property
/// 3. Falls back to generic rendering if no adapter exists
/// 4. Provides thread-safe access (Arc<RwLock>)
///
/// # Example
///
/// ```ignore
/// let mut registry = AdapterRegistry::new();
/// registry.register(Box::new(TrustAdapter::new()));
/// registry.register(Box::new(FamilyAdapter::new()));
///
/// // Later, when rendering:
/// registry.render_property("trust_level", &value, ui);
/// ```
#[derive(Clone)]
pub struct AdapterRegistry {
    adapters: Arc<RwLock<Vec<BoxedAdapter>>>,
}

impl AdapterRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            adapters: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register an adapter
    ///
    /// Adapters are checked in registration order (last registered = checked first)
    /// unless they specify a priority.
    pub fn register(&self, adapter: BoxedAdapter) {
        let mut adapters = self.adapters.write().expect("adapter registry lock poisoned");
        adapters.push(adapter);

        // Sort by priority (highest first)
        adapters.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Render a property with the appropriate adapter or generic fallback
    pub fn render_property(&self, key: &str, value: &PropertyValue, ui: &mut Ui) {
        // Check if we have an adapter for this key
        let has_adapter = {
            let adapters = self.adapters.read().expect("adapter registry lock poisoned");
            adapters.iter().any(|a| a.handles(key))
        };

        if has_adapter {
            // Render with adapter
            let adapters = self.adapters.read().expect("adapter registry lock poisoned");
            if let Some(adapter) = adapters.iter().find(|a| a.handles(key)) {
                adapter.render(key, value, ui);
            }
        } else {
            // Generic fallback
            self.render_generic_property(key, value, ui);
        }
    }

    /// Get node decoration from all adapters
    ///
    /// If multiple adapters provide decorations, they're merged (last wins).
    pub fn get_node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
        let adapters = self.adapters.read().expect("adapter registry lock poisoned");

        let mut decoration: Option<NodeDecoration> = None;

        for adapter in adapters.iter() {
            if let Some(dec) = adapter.node_decoration(properties) {
                // Merge decorations
                if let Some(existing) = &mut decoration {
                    if dec.badge.is_some() {
                        existing.badge = dec.badge;
                    }
                    if dec.fill_color.is_some() {
                        existing.fill_color = dec.fill_color;
                    }
                    if dec.ring_color.is_some() {
                        existing.ring_color = dec.ring_color;
                    }
                    if dec.tooltip.is_some() {
                        existing.tooltip = dec.tooltip;
                    }
                } else {
                    decoration = Some(dec);
                }
            }
        }

        decoration
    }

    /// Generic property rendering (fallback when no adapter exists)
    fn render_generic_property(&self, key: &str, value: &PropertyValue, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Key in gray
            ui.label(egui::RichText::new(format!("{}: ", key)).color(Color32::GRAY));

            // Value based on type
            match value {
                PropertyValue::String(s) => {
                    ui.label(s);
                }
                PropertyValue::Number(n) => {
                    ui.label(format!("{}", n));
                }
                PropertyValue::Boolean(b) => {
                    ui.label(if *b { "✓ true" } else { "✗ false" });
                }
                PropertyValue::Null => {
                    ui.label(egui::RichText::new("null").color(Color32::DARK_GRAY));
                }
                PropertyValue::Object(_) => {
                    ui.label(egui::RichText::new("{...}").color(Color32::YELLOW));
                }
                PropertyValue::Array(arr) => {
                    ui.label(
                        egui::RichText::new(format!("[{} items]", arr.len()))
                            .color(Color32::YELLOW),
                    );
                }
            }
        });
    }

    /// Get count of registered adapters
    pub fn adapter_count(&self) -> usize {
        self.adapters.read().expect("adapter registry lock poisoned").len()
    }

    /// Get names of all registered adapters (for debugging)
    pub fn adapter_names(&self) -> Vec<String> {
        self.adapters
            .read()
            .expect("adapter registry lock poisoned")
            .iter()
            .map(|a| a.name().to_string())
            .collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter_trait::PropertyAdapter;

    struct TestAdapter {
        name: String,
    }

    impl PropertyAdapter for TestAdapter {
        fn name(&self) -> &str {
            &self.name
        }

        fn handles(&self, property_key: &str) -> bool {
            property_key == "test"
        }

        fn render(&self, _key: &str, _value: &PropertyValue, _ui: &mut Ui) {
            // Test implementation
        }
    }

    #[test]
    fn test_registry_registration() {
        let registry = AdapterRegistry::new();
        assert_eq!(registry.adapter_count(), 0);

        registry.register(Box::new(TestAdapter {
            name: "test1".to_string(),
        }));
        assert_eq!(registry.adapter_count(), 1);

        registry.register(Box::new(TestAdapter {
            name: "test2".to_string(),
        }));
        assert_eq!(registry.adapter_count(), 2);
    }

    #[test]
    fn test_registry_adapter_names() {
        let registry = AdapterRegistry::new();
        registry.register(Box::new(TestAdapter {
            name: "alpha".to_string(),
        }));
        registry.register(Box::new(TestAdapter {
            name: "beta".to_string(),
        }));

        let names = registry.adapter_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
    }
}

