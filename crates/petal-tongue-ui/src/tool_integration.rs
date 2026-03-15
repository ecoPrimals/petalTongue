// SPDX-License-Identifier: AGPL-3.0-only
//! Generic tool integration system for petalTongue
//!
//! This module provides a capability-based system for integrating external tools
//! without hardcoding tool-specific knowledge into the application.
//!
//! # Design Principles
//!
//! 1. **No Hardcoded Tool Knowledge**: petalTongue doesn't know about specific tools
//! 2. **Capability-Based**: Tools advertise what they can do
//! 3. **Dynamic Discovery**: Tools are discovered and loaded at runtime
//! 4. **Self-Describing**: Tools provide their own UI and metadata

use egui;

/// Capabilities that a tool can advertise
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolCapability {
    /// Tool provides visual output
    Visual,
    /// Tool provides audio output
    Audio,
    /// Tool accepts text input (seeds, prompts, etc.)
    TextInput,
    /// Tool supports progressive reveal/animation
    Progressive,
    /// Tool can export data
    Export,
    /// Custom capability
    Custom(String),
}

/// Metadata about a tool
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// Tool name (e.g., "`BingoCube`")
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
    /// Capabilities this tool provides
    pub capabilities: Vec<ToolCapability>,
    /// Icon (emoji or path)
    pub icon: String,
    /// Tool repository/source
    pub source: Option<String>,
}

/// Trait that all integrated tools must implement
///
/// This allows petalTongue to work with any tool that implements this interface,
/// without knowing specific details about the tool.
pub trait ToolPanel: Send + Sync {
    /// Get tool metadata
    fn metadata(&self) -> &ToolMetadata;

    /// Check if tool should be shown in the UI
    ///
    /// Tools can decide based on their own state whether they want to be visible
    fn is_visible(&self) -> bool {
        true
    }

    /// Toggle visibility
    fn toggle_visibility(&mut self);

    /// Render the tool's panel
    ///
    /// The tool is responsible for its own UI rendering
    fn render_panel(&mut self, ui: &mut egui::Ui);

    /// Optional: Get tool status message
    fn status_message(&self) -> Option<String> {
        None
    }

    /// Optional: Handle tool-specific actions
    ///
    /// # Errors
    ///
    /// Returns `Err` if the action fails or is not supported.
    fn handle_action(&mut self, _action: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Manager for all integrated tools
///
/// This is what the application holds - a collection of tool panels that can be
/// dynamically added, removed, and rendered without the app knowing specifics.
pub struct ToolManager {
    tools: Vec<Box<dyn ToolPanel>>,
}

impl ToolManager {
    /// Create a new tool manager
    #[must_use]
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool
    ///
    /// Tools can be added at runtime based on discovery
    pub fn register_tool(&mut self, tool: Box<dyn ToolPanel>) {
        tracing::info!(
            "Registered tool: {} v{} with capabilities: {:?}",
            tool.metadata().name,
            tool.metadata().version,
            tool.metadata().capabilities
        );
        self.tools.push(tool);
    }

    /// Get all registered tools
    #[must_use]
    pub fn tools(&self) -> &[Box<dyn ToolPanel>] {
        &self.tools
    }

    /// Get mutable reference to tools
    pub fn tools_mut(&mut self) -> &mut [Box<dyn ToolPanel>] {
        &mut self.tools
    }

    /// Find a tool by name
    #[must_use]
    pub fn find_tool(&self, name: &str) -> Option<&dyn ToolPanel> {
        self.tools
            .iter()
            .find(|t| t.metadata().name == name)
            .map(AsRef::as_ref)
    }

    /// Find a tool by name (mutable)
    pub fn find_tool_mut(&mut self, name: &str) -> Option<&mut Box<dyn ToolPanel>> {
        self.tools.iter_mut().find(|t| t.metadata().name == name)
    }

    /// Get tools with specific capability
    #[must_use]
    pub fn tools_with_capability(&self, capability: &ToolCapability) -> Vec<&dyn ToolPanel> {
        self.tools
            .iter()
            .filter(|t| t.metadata().capabilities.contains(capability))
            .map(AsRef::as_ref)
            .collect()
    }

    /// Render tools menu (list of available tools)
    pub fn render_tools_menu(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("🔧 Available Tools").size(16.0));
        ui.separator();

        if self.tools.is_empty() {
            ui.label(egui::RichText::new("No tools registered").color(egui::Color32::GRAY));
            return;
        }

        for tool in &mut self.tools {
            // Clone metadata to avoid borrow issues
            let icon = tool.metadata().icon.clone();
            let name = tool.metadata().name.clone();
            let description = tool.metadata().description.clone();
            let is_visible = tool.is_visible();

            ui.horizontal(|ui| {
                ui.label(&icon);

                let button_text = if is_visible {
                    format!("✓ {name}")
                } else {
                    name.clone()
                };

                if ui.button(button_text).clicked() {
                    tool.toggle_visibility();
                }

                ui.label(
                    egui::RichText::new(&description)
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            });

            if let Some(status) = tool.status_message() {
                ui.label(
                    egui::RichText::new(format!("  ℹ {status}"))
                        .size(11.0)
                        .color(egui::Color32::LIGHT_BLUE),
                );
            }
        }
    }

    /// Get the currently visible tool (if only one should be shown at a time)
    pub fn visible_tool(&mut self) -> Option<&mut Box<dyn ToolPanel>> {
        self.tools.iter_mut().find(|t| t.is_visible())
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTool {
        metadata: ToolMetadata,
        visible: bool,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                metadata: ToolMetadata {
                    name: name.to_string(),
                    description: "Mock tool".to_string(),
                    version: "0.1.0".to_string(),
                    capabilities: vec![ToolCapability::Visual],
                    icon: "🔧".to_string(),
                    source: None,
                },
                visible: false,
            }
        }
    }

    impl ToolPanel for MockTool {
        fn metadata(&self) -> &ToolMetadata {
            &self.metadata
        }

        fn is_visible(&self) -> bool {
            self.visible
        }

        fn toggle_visibility(&mut self) {
            self.visible = !self.visible;
        }

        fn render_panel(&mut self, _ui: &mut egui::Ui) {
            // Mock implementation
        }
    }

    #[test]
    fn test_tool_registration() {
        let mut manager = ToolManager::new();
        assert_eq!(manager.tools().len(), 0);

        manager.register_tool(Box::new(MockTool::new("TestTool")));
        assert_eq!(manager.tools().len(), 1);
    }

    #[test]
    fn test_find_tool() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new("Tool1")));
        manager.register_tool(Box::new(MockTool::new("Tool2")));

        assert!(manager.find_tool("Tool1").is_some());
        assert!(manager.find_tool("Tool2").is_some());
        assert!(manager.find_tool("Tool3").is_none());
    }

    #[test]
    fn test_capability_filtering() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new("VisualTool")));

        let visual_tools = manager.tools_with_capability(&ToolCapability::Visual);
        assert_eq!(visual_tools.len(), 1);

        let audio_tools = manager.tools_with_capability(&ToolCapability::Audio);
        assert_eq!(audio_tools.len(), 0);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut tool = MockTool::new("TestTool");
        assert!(!tool.is_visible());
        tool.toggle_visibility();
        assert!(tool.is_visible());
        tool.toggle_visibility();
        assert!(!tool.is_visible());
    }

    #[test]
    fn test_handle_action_default() {
        let mut tool = MockTool::new("TestTool");
        assert!(tool.handle_action("unknown_action").is_ok());
    }

    #[test]
    fn test_visible_tool() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new("Tool1")));
        assert!(manager.visible_tool().is_none());

        let mut tool = MockTool::new("VisibleTool");
        tool.toggle_visibility();
        manager.register_tool(Box::new(tool));
        assert!(manager.visible_tool().is_some());
    }

    #[test]
    fn test_tool_manager_default() {
        let manager = ToolManager::default();
        assert!(manager.tools().is_empty());
    }

    #[test]
    fn test_find_tool_mut() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new("EditableTool")));
        let tool = manager.find_tool_mut("EditableTool");
        assert!(tool.is_some());
        let tool = manager.find_tool_mut("Nonexistent");
        assert!(tool.is_none());
    }

    #[test]
    fn test_tools_with_multiple_capabilities() {
        let mut tool = MockTool::new("MultiCap");
        tool.metadata.capabilities = vec![
            ToolCapability::Visual,
            ToolCapability::Audio,
            ToolCapability::Export,
        ];
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(tool));

        assert_eq!(
            manager.tools_with_capability(&ToolCapability::Visual).len(),
            1
        );
        assert_eq!(
            manager.tools_with_capability(&ToolCapability::Audio).len(),
            1
        );
        assert_eq!(
            manager.tools_with_capability(&ToolCapability::Export).len(),
            1
        );
        assert_eq!(
            manager
                .tools_with_capability(&ToolCapability::TextInput)
                .len(),
            0
        );
    }

    #[test]
    fn test_tool_capability_custom() {
        assert_eq!(
            ToolCapability::Custom("x".to_string()),
            ToolCapability::Custom("x".to_string())
        );
        assert_ne!(
            ToolCapability::Custom("a".to_string()),
            ToolCapability::Custom("b".to_string())
        );
    }

    #[test]
    fn test_tool_metadata_source() {
        let mut tool = MockTool::new("WithSource");
        tool.metadata.source = Some("https://example.com".to_string());
        assert_eq!(tool.metadata.source.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn test_status_message_default() {
        let tool = MockTool::new("NoStatus");
        assert!(tool.status_message().is_none());
    }

    #[test]
    fn test_tools_mut_empty() {
        let mut manager = ToolManager::new();
        let tools = manager.tools_mut();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tools_mut_modify() {
        let mut manager = ToolManager::new();
        manager.register_tool(Box::new(MockTool::new("T1")));
        let tools = manager.tools_mut();
        assert_eq!(tools.len(), 1);
    }

    #[test]
    fn test_tool_capability_variants_eq() {
        assert_eq!(ToolCapability::Visual, ToolCapability::Visual);
        assert_eq!(ToolCapability::Audio, ToolCapability::Audio);
        assert_ne!(ToolCapability::Visual, ToolCapability::Audio);
        assert_eq!(ToolCapability::TextInput, ToolCapability::TextInput);
        assert_eq!(ToolCapability::Progressive, ToolCapability::Progressive);
        assert_eq!(ToolCapability::Export, ToolCapability::Export);
    }

    #[test]
    fn test_tool_capability_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        ToolCapability::Visual.hash(&mut hasher);
        let _ = hasher.finish();
    }

    #[test]
    fn test_find_tool_empty_manager() {
        let manager = ToolManager::new();
        assert!(manager.find_tool("Any").is_none());
    }

    #[test]
    fn test_visible_tool_multiple_registered() {
        let mut manager = ToolManager::new();
        let mut t1 = MockTool::new("T1");
        t1.toggle_visibility();
        manager.register_tool(Box::new(t1));
        manager.register_tool(Box::new(MockTool::new("T2")));
        let visible = manager.visible_tool();
        assert!(visible.is_some());
        assert_eq!(visible.unwrap().metadata().name, "T1");
    }
}
