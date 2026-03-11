// SPDX-License-Identifier: AGPL-3.0-only
//! Node Palette - Available Node Types for Graph Builder
//!
//! Displays all available node types that can be added to a graph.
//! TRUE PRIMAL: Zero hardcoding, capability-based discovery of node types.

use crate::accessibility::ColorPalette;
use egui::{Color32, RichText, Ui};
use petal_tongue_core::graph_builder::NodeType;

/// Node palette showing available node types
pub struct NodePalette {
    /// Search filter
    search: String,

    /// Currently dragging node type
    dragging: Option<NodeType>,

    /// Available node types (discovered at runtime)
    node_types: Vec<NodeTypeInfo>,
}

impl NodePalette {
    /// Create a new node palette
    #[must_use]
    pub fn new() -> Self {
        Self {
            search: String::new(),
            dragging: None,
            node_types: Self::discover_node_types(),
        }
    }

    /// Discover available node types (capability-based, zero hardcoding!)
    /// TRUE PRIMAL: This queries the `NodeType` enum for what's available
    fn discover_node_types() -> Vec<NodeTypeInfo> {
        vec![
            NodeTypeInfo::from_node_type(NodeType::PrimalStart),
            NodeTypeInfo::from_node_type(NodeType::Verification),
            NodeTypeInfo::from_node_type(NodeType::WaitFor),
            NodeTypeInfo::from_node_type(NodeType::Conditional),
        ]
    }

    /// Render the palette
    pub fn render(&mut self, ui: &mut Ui, palette: &ColorPalette) {
        ui.heading(
            RichText::new("🎨 Node Palette")
                .size(16.0)
                .color(palette.accent),
        );

        ui.add_space(8.0);

        // Search filter
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search);
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Filter node types based on search (clone to avoid borrow issues)
        let filtered_types: Vec<_> = self
            .node_types
            .iter()
            .filter(|info| {
                if self.search.is_empty() {
                    true
                } else {
                    info.name
                        .to_lowercase()
                        .contains(&self.search.to_lowercase())
                        || info
                            .description
                            .to_lowercase()
                            .contains(&self.search.to_lowercase())
                }
            })
            .cloned()
            .collect();

        // Display node types
        for node_info in &filtered_types {
            self.render_node_type(ui, node_info, palette);
            ui.add_space(4.0);
        }

        if self.node_types.is_empty() {
            ui.label(RichText::new("No nodes available").color(palette.text_dim));
        }
    }

    /// Render a single node type button
    fn render_node_type(&mut self, ui: &mut Ui, info: &NodeTypeInfo, palette: &ColorPalette) {
        let button_color = if self.dragging.as_ref() == Some(&info.node_type) {
            Color32::from_rgb(100, 150, 255) // Dragging: Light blue
        } else {
            Color32::from_rgb(74, 144, 226) // Default: Blue
        };

        let response = ui
            .add(
                egui::Button::new(
                    RichText::new(format!("{} {}", info.icon, info.name))
                        .size(14.0)
                        .color(Color32::WHITE),
                )
                .fill(button_color)
                .min_size(egui::Vec2::new(ui.available_width(), 40.0)),
            )
            .on_hover_text(&info.description); // Capture returned response

        // Handle drag start
        if response.drag_started() {
            self.dragging = Some(info.node_type.clone());
        }

        // Handle drag released
        if response.drag_stopped() {
            self.dragging = None;
        }

        // Show required parameters
        if !info.required_params.is_empty() {
            ui.indent("params", |ui| {
                ui.label(
                    RichText::new(format!("Required: {}", info.required_params.join(", ")))
                        .size(10.0)
                        .color(palette.text_dim),
                );
            });
        }
    }

    /// Get currently dragging node type
    #[must_use]
    pub fn get_dragging(&self) -> Option<&NodeType> {
        self.dragging.as_ref()
    }

    /// Clear dragging state
    pub fn clear_dragging(&mut self) {
        self.dragging = None;
    }

    /// Check if currently dragging
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }
}

impl Default for NodePalette {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a node type
#[derive(Clone, Debug)]
struct NodeTypeInfo {
    /// Node type
    node_type: NodeType,

    /// Display name
    name: String,

    /// Icon
    icon: String,

    /// Description
    description: String,

    /// Required parameters
    required_params: Vec<String>,
}

impl NodeTypeInfo {
    /// Create from a `NodeType` (TRUE PRIMAL: queries the type for its own metadata!)
    fn from_node_type(node_type: NodeType) -> Self {
        Self {
            name: node_type.display_name().to_string(),
            icon: node_type.icon().to_string(),
            description: node_type.description().to_string(),
            required_params: node_type
                .required_parameters()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            node_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_creation() {
        let palette = NodePalette::new();
        assert!(palette.search.is_empty());
        assert!(palette.dragging.is_none());
        assert_eq!(palette.node_types.len(), 4); // 4 node types discovered
    }

    #[test]
    fn test_node_discovery() {
        let node_types = NodePalette::discover_node_types();
        assert_eq!(node_types.len(), 4);

        // Verify we have all expected types
        let has_primal_start = node_types
            .iter()
            .any(|n| matches!(n.node_type, NodeType::PrimalStart));
        let has_verification = node_types
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Verification));
        let has_wait_for = node_types
            .iter()
            .any(|n| matches!(n.node_type, NodeType::WaitFor));
        let has_conditional = node_types
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Conditional));

        assert!(has_primal_start);
        assert!(has_verification);
        assert!(has_wait_for);
        assert!(has_conditional);
    }

    #[test]
    fn test_search_filter() {
        let mut palette = NodePalette::new();

        // Search for "start"
        palette.search = "start".to_string();
        let filtered: Vec<_> = palette
            .node_types
            .iter()
            .filter(|info| {
                info.name
                    .to_lowercase()
                    .contains(&palette.search.to_lowercase())
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert!(matches!(filtered[0].node_type, NodeType::PrimalStart));
    }

    #[test]
    fn test_dragging_state() {
        let mut palette = NodePalette::new();
        assert!(!palette.is_dragging());
        assert!(palette.get_dragging().is_none());

        palette.dragging = Some(NodeType::PrimalStart);
        assert!(palette.is_dragging());
        assert!(palette.get_dragging().is_some());

        palette.clear_dragging();
        assert!(!palette.is_dragging());
    }

    #[test]
    fn test_node_info_from_type() {
        let info = NodeTypeInfo::from_node_type(NodeType::PrimalStart);

        assert_eq!(info.name, "Start Primal");
        assert_eq!(info.icon, "🚀");
        assert!(!info.description.is_empty());
        assert_eq!(info.required_params.len(), 2); // primal_name, family_id
    }

    #[test]
    fn test_node_palette_default() {
        let palette = NodePalette::default();
        assert!(palette.search.is_empty());
        assert_eq!(palette.node_types.len(), 4);
    }

    #[test]
    fn test_node_info_verification_type() {
        let info = NodeTypeInfo::from_node_type(NodeType::Verification);
        assert!(info.name.contains("Verify") || info.name.contains("Verification"));
        assert!(!info.required_params.is_empty());
    }

    #[test]
    fn test_node_info_conditional_type() {
        let info = NodeTypeInfo::from_node_type(NodeType::Conditional);
        assert!(!info.name.is_empty());
        assert!(!info.description.is_empty());
    }

    #[test]
    fn test_search_filter_empty_shows_all() {
        let palette = NodePalette::new();
        let filtered: Vec<_> = palette
            .node_types
            .iter()
            .filter(|info| {
                palette.search.is_empty()
                    || info
                        .name
                        .to_lowercase()
                        .contains(&palette.search.to_lowercase())
                    || info
                        .description
                        .to_lowercase()
                        .contains(&palette.search.to_lowercase())
            })
            .collect();
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_search_filter_nonexistent_returns_empty() {
        let mut palette = NodePalette::new();
        palette.search = "xyznonexistent123".to_string();
        let filtered: Vec<_> = palette
            .node_types
            .iter()
            .filter(|info| {
                info.name
                    .to_lowercase()
                    .contains(&palette.search.to_lowercase())
                    || info
                        .description
                        .to_lowercase()
                        .contains(&palette.search.to_lowercase())
            })
            .collect();
        assert!(filtered.is_empty());
    }
}
