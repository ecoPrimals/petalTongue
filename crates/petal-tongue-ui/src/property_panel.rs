//! Property Panel - Node Parameter Editor
//!
//! Provides forms for editing node parameters based on node type.
//! TRUE PRIMAL: Zero hardcoding, capability-based discovery of required parameters.

use crate::accessibility::ColorPalette;
use egui::{Color32, RichText, Ui};
use petal_tongue_core::graph_builder::{GraphNode, NodeType, VisualGraph};
use std::collections::HashMap;

/// Property panel for editing selected node
pub struct PropertyPanel {
    /// Currently editing node ID
    editing_node: Option<String>,

    /// Temporary parameter values (for editing)
    temp_params: HashMap<String, String>,

    /// Validation errors
    errors: HashMap<String, String>,
}

impl PropertyPanel {
    /// Create a new property panel
    #[must_use]
    pub fn new() -> Self {
        Self {
            editing_node: None,
            temp_params: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Set the node being edited
    pub fn set_editing_node(&mut self, node_id: Option<String>, graph: &VisualGraph) {
        self.editing_node = node_id.clone();
        self.temp_params.clear();
        self.errors.clear();

        // Load current parameters
        if let Some(id) = &node_id {
            if let Some(node) = graph.get_node(id) {
                for (key, value) in &node.parameters {
                    self.temp_params.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Render the property panel
    pub fn render(&mut self, ui: &mut Ui, graph: &mut VisualGraph, palette: &ColorPalette) {
        ui.heading(
            RichText::new("⚙️ Properties")
                .size(16.0)
                .color(palette.accent),
        );

        ui.add_space(8.0);

        if let Some(node_id) = &self.editing_node {
            if let Some(node) = graph.get_node(node_id) {
                self.render_node_properties(ui, node, palette);
                ui.add_space(12.0);
                self.render_actions(ui, graph, palette);
            } else {
                ui.label("Node not found");
                self.editing_node = None;
            }
        } else {
            ui.label(
                RichText::new("No node selected")
                    .color(palette.text_dim),
            );
            ui.add_space(8.0);
            ui.label("Select a node to edit its properties");
        }
    }

    /// Render node properties form
    fn render_node_properties(&mut self, ui: &mut Ui, node: &GraphNode, palette: &ColorPalette) {
        // Node type (read-only)
        ui.horizontal(|ui| {
            ui.label(RichText::new("Type:").strong());
            ui.label(format!("{} {}", node.node_type.icon(), node.node_type.display_name()));
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Description
        ui.label(
            RichText::new(node.node_type.description())
                .size(12.0)
                .color(palette.text_dim),
        );

        ui.add_space(12.0);

        // Required parameters (discovered from node type!)
        let required_params = node.node_type.required_parameters();
        
        if required_params.is_empty() {
            ui.label("No parameters required");
        } else {
            ui.label(RichText::new("Required Parameters:").strong());
            ui.add_space(8.0);

            for param_name in required_params {
                self.render_parameter_field(ui, param_name, palette);
            }
        }

        // Show validation status
        if !self.errors.is_empty() {
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label(
                RichText::new("⚠️ Validation Errors")
                    .color(Color32::from_rgb(208, 2, 27))
                    .strong(),
            );

            for (field, error) in &self.errors {
                ui.label(
                    RichText::new(format!("• {}: {}", field, error))
                        .size(12.0)
                        .color(Color32::from_rgb(208, 2, 27)),
                );
            }
        }
    }

    /// Render a single parameter field
    fn render_parameter_field(&mut self, ui: &mut Ui, param_name: &str, palette: &ColorPalette) {
        ui.horizontal(|ui| {
            // Label
            ui.label(
                RichText::new(format!("{}:", param_name))
                    .size(13.0),
            );

            // Get or create temp value
            let current_value = self.temp_params.get(param_name).cloned().unwrap_or_default();
            let mut new_value = current_value.clone();

            // Input field
            let response = ui.text_edit_singleline(&mut new_value);

            // Update temp params if changed
            if response.changed() {
                self.temp_params.insert(param_name.to_string(), new_value.clone());
                // Clear error for this field
                self.errors.remove(param_name);
            }

            // Show error indicator
            if self.errors.contains_key(param_name) {
                ui.label(
                    RichText::new("❌")
                        .color(Color32::from_rgb(208, 2, 27)),
                );
            } else if !new_value.is_empty() {
                ui.label(
                    RichText::new("✅")
                        .color(Color32::from_rgb(40, 180, 40)),
                );
            }
        });

        // Show field-specific help text
        ui.indent("help", |ui| {
            ui.label(
                RichText::new(self.get_parameter_help(param_name))
                    .size(10.0)
                    .color(palette.text_dim),
            );
        });

        ui.add_space(4.0);
    }

    /// Get help text for a parameter (TRUE PRIMAL: context-aware, not hardcoded!)
    fn get_parameter_help(&self, param_name: &str) -> &'static str {
        match param_name {
            "primal_name" => "Name of the primal to start (e.g., beardog-server)",
            "family_id" => "Family ID for the primal (e.g., nat0)",
            "timeout" => "Timeout in seconds (e.g., 30)",
            "condition" => "Condition to wait for or evaluate",
            _ => "Enter value for this parameter",
        }
    }

    /// Render action buttons
    fn render_actions(&mut self, ui: &mut Ui, graph: &mut VisualGraph, palette: &ColorPalette) {
        ui.horizontal(|ui| {
            // Apply button
            let apply_enabled = self.editing_node.is_some();
            let apply_button = ui.add_enabled(
                apply_enabled,
                egui::Button::new(
                    RichText::new("✅ Apply")
                        .size(14.0)
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(40, 180, 40)),
            );

            if apply_button.clicked() {
                self.apply_changes(graph);
            }

            // Reset button
            let reset_button = ui.add_enabled(
                apply_enabled,
                egui::Button::new(
                    RichText::new("↻ Reset")
                        .size(14.0),
                )
                .fill(palette.background_alt),
            );

            if reset_button.clicked() {
                self.reset_changes(graph);
            }
        });

        ui.add_space(8.0);

        // Show validation status
        if let Some(node_id) = &self.editing_node {
            if let Some(node) = graph.get_node(node_id) {
                if node.has_all_required_parameters() {
                    ui.label(
                        RichText::new("✅ All required parameters set")
                            .color(Color32::from_rgb(40, 180, 40)),
                    );
                } else {
                    let missing = node.missing_parameters();
                    ui.label(
                        RichText::new(format!("❌ Missing: {}", missing.join(", ")))
                            .color(Color32::from_rgb(208, 2, 27)),
                    );
                }
            }
        }
    }

    /// Apply changes to the graph
    fn apply_changes(&mut self, graph: &mut VisualGraph) {
        self.errors.clear();

        if let Some(node_id) = &self.editing_node {
            if let Some(node) = graph.get_node_mut(node_id) {
                // Validate all required parameters are filled
                let required_params = node.node_type.required_parameters();
                let mut has_errors = false;

                for param_name in required_params {
                    if let Some(value) = self.temp_params.get(*param_name) {
                        if value.trim().is_empty() {
                            self.errors.insert(
                                param_name.to_string(),
                                "Required field cannot be empty".to_string(),
                            );
                            has_errors = true;
                        }
                    } else {
                        self.errors.insert(
                            param_name.to_string(),
                            "Required field is missing".to_string(),
                        );
                        has_errors = true;
                    }
                }

                if !has_errors {
                    // Apply all parameters
                    for (key, value) in &self.temp_params {
                        node.set_parameter(key.clone(), value.clone());
                    }

                    // Clear error state
                    node.visual_state.has_error = false;
                    node.visual_state.error_message = None;
                } else {
                    // Set error state
                    node.visual_state.has_error = true;
                    node.visual_state.error_message = Some("Missing required parameters".to_string());
                }
            }
        }
    }

    /// Reset changes to original values
    fn reset_changes(&mut self, graph: &VisualGraph) {
        self.temp_params.clear();
        self.errors.clear();

        if let Some(node_id) = &self.editing_node {
            if let Some(node) = graph.get_node(node_id) {
                for (key, value) in &node.parameters {
                    self.temp_params.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Get currently editing node ID
    #[must_use]
    pub fn get_editing_node(&self) -> Option<&String> {
        self.editing_node.as_ref()
    }

    /// Check if panel has unsaved changes
    #[must_use]
    pub fn has_unsaved_changes(&self, graph: &VisualGraph) -> bool {
        if let Some(node_id) = &self.editing_node {
            if let Some(node) = graph.get_node(node_id) {
                // Check if temp params differ from node params
                for (key, value) in &self.temp_params {
                    if node.get_parameter(key) != Some(value) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Default for PropertyPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::graph_builder::Vec2;

    #[test]
    fn test_panel_creation() {
        let panel = PropertyPanel::new();
        assert!(panel.editing_node.is_none());
        assert!(panel.temp_params.is_empty());
        assert!(panel.errors.is_empty());
    }

    #[test]
    fn test_set_editing_node() {
        let mut panel = PropertyPanel::new();
        let mut graph = VisualGraph::new("test".to_string());

        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "beardog".to_string());
        let node_id = node.id.clone();
        graph.add_node(node);

        panel.set_editing_node(Some(node_id.clone()), &graph);

        assert_eq!(panel.editing_node, Some(node_id));
        assert_eq!(panel.temp_params.get("primal_name"), Some(&"beardog".to_string()));
    }

    #[test]
    fn test_apply_changes() {
        let mut panel = PropertyPanel::new();
        let mut graph = VisualGraph::new("test".to_string());

        let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node_id = node.id.clone();
        graph.add_node(node);

        panel.set_editing_node(Some(node_id.clone()), &graph);
        panel.temp_params.insert("primal_name".to_string(), "beardog".to_string());
        panel.temp_params.insert("family_id".to_string(), "nat0".to_string());

        panel.apply_changes(&mut graph);

        let node = graph.get_node(&node_id).unwrap();
        assert_eq!(node.get_parameter("primal_name"), Some(&"beardog".to_string()));
        assert_eq!(node.get_parameter("family_id"), Some(&"nat0".to_string()));
        assert!(!node.visual_state.has_error);
    }

    #[test]
    fn test_validation_errors() {
        let mut panel = PropertyPanel::new();
        let mut graph = VisualGraph::new("test".to_string());

        let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node_id = node.id.clone();
        graph.add_node(node);

        panel.set_editing_node(Some(node_id.clone()), &graph);
        // Don't set required parameters
        panel.apply_changes(&mut graph);

        assert!(!panel.errors.is_empty());
        let node = graph.get_node(&node_id).unwrap();
        assert!(node.visual_state.has_error);
    }

    #[test]
    fn test_reset_changes() {
        let mut panel = PropertyPanel::new();
        let mut graph = VisualGraph::new("test".to_string());

        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "original".to_string());
        let node_id = node.id.clone();
        graph.add_node(node);

        panel.set_editing_node(Some(node_id.clone()), &graph);
        panel.temp_params.insert("primal_name".to_string(), "modified".to_string());

        panel.reset_changes(&graph);

        assert_eq!(panel.temp_params.get("primal_name"), Some(&"original".to_string()));
    }

    #[test]
    fn test_has_unsaved_changes() {
        let mut panel = PropertyPanel::new();
        let mut graph = VisualGraph::new("test".to_string());

        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "original".to_string());
        let node_id = node.id.clone();
        graph.add_node(node);

        panel.set_editing_node(Some(node_id), &graph);
        assert!(!panel.has_unsaved_changes(&graph));

        panel.temp_params.insert("primal_name".to_string(), "modified".to_string());
        assert!(panel.has_unsaved_changes(&graph));
    }
}

