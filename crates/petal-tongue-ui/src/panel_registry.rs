//! Panel Registry - Dynamic panel type registration and instantiation
//!
//! This module provides the infrastructure for registering custom panel types
//! and creating panel instances from scenario configuration.
//!
//! # Evolution Note
//! This system emerged from implementing Doom (Gap #1 in DOOM_GAP_LOG.md).
//! We needed a way to map `"doom_game"` in JSON to `DoomPanel` creation.

use crate::scenario::CustomPanelConfig;
use std::collections::HashMap;
use std::sync::Arc;

/// Action a panel wants to take after an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelAction {
    /// Continue normal operation
    Continue,
    
    /// Request to close the panel
    Close,
    
    /// Request to restart the panel
    Restart,
}

/// Error type for panel operations
#[derive(Debug, thiserror::Error)]
pub enum PanelError {
    #[error("Unknown panel type: {0}")]
    UnknownType(String),
    
    #[error("Panel creation failed: {0}")]
    CreationFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, PanelError>;

/// Factory for creating panel instances
///
/// Each custom panel type implements this trait to enable
/// registration and instantiation from scenarios.
pub trait PanelFactory: Send + Sync {
    /// Get the panel type identifier (e.g., "doom_game")
    fn panel_type(&self) -> &str;
    
    /// Create a new panel instance from configuration
    fn create(&self, config: &CustomPanelConfig) -> Result<Box<dyn PanelInstance>>;
    
    /// Get human-readable description
    fn description(&self) -> &str {
        "Custom panel"
    }
}

/// A panel instance that can be rendered
///
/// This trait defines the minimum interface that all custom panels must implement.
/// As we discover more requirements (lifecycle, input handling, etc.), we'll extend this.
pub trait PanelInstance: Send {
    /// Render the panel to egui
    fn render(&mut self, ui: &mut egui::Ui);
    
    /// Get panel title
    fn title(&self) -> &str;
    
    /// Optional: Update panel state (called each frame)
    fn update(&mut self) {}
    
    /// Optional: Handle panel-specific events
    fn on_event(&mut self, _event: &egui::Event) {}
    
    // ===== Input Focus Methods (Phase 3 Evolution) =====
    
    /// Does this panel want keyboard input?
    fn wants_keyboard_input(&self) -> bool {
        false
    }
    
    /// Does this panel want mouse input?
    fn wants_mouse_input(&self) -> bool {
        false
    }
    
    /// Does this panel want exclusive input? (like games)
    fn wants_exclusive_input(&self) -> bool {
        false
    }
    
    /// Input priority (0-255, higher = gets input first)
    fn input_priority(&self) -> u8 {
        5 // Medium priority
    }
    
    /// Handle keyboard event
    /// Returns InputAction indicating if input was consumed
    fn on_keyboard_event(&mut self, _ctx: &egui::Context) -> crate::focus_manager::InputAction {
        crate::focus_manager::InputAction::Ignored
    }
    
    /// Handle mouse event
    /// Returns InputAction indicating if input was consumed
    fn on_mouse_event(&mut self, _ctx: &egui::Context) -> crate::focus_manager::InputAction {
        crate::focus_manager::InputAction::Ignored
    }
    
    // ===== Lifecycle Hooks (Phase 4 Evolution) =====
    
    /// Called when panel is first opened/created
    /// Use this to initialize resources (load assets, create connections, etc.)
    fn on_open(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    
    /// Called when panel is about to close
    /// Use this to clean up resources (close files, disconnect, etc.)
    fn on_close(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    
    /// Called when panel is paused (e.g., window minimized, panel hidden)
    /// Use this to pause expensive operations
    fn on_pause(&mut self) {
        // Default: do nothing
    }
    
    /// Called when panel is resumed after being paused
    /// Use this to resume operations
    fn on_resume(&mut self) {
        // Default: do nothing
    }
    
    /// Called when panel encounters an error
    /// Return PanelAction to indicate what should happen next
    fn on_error(&mut self, error: &anyhow::Error) -> PanelAction {
        tracing::error!("Panel '{}' error: {}", self.title(), error);
        PanelAction::Continue // Default: log and continue
    }
    
    // ===== State Persistence =====
    
    /// Can this panel save its state?
    fn can_save_state(&self) -> bool {
        false
    }
    
    /// Can this panel restore from saved state?
    fn can_restore_state(&self) -> bool {
        false
    }
    
    /// Save panel state to JSON
    /// Only called if can_save_state() returns true
    fn save_state(&self) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
    
    /// Restore panel state from JSON
    /// Only called if can_restore_state() returns true
    fn restore_state(&mut self, _state: serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }
    
    // ===== Panel Queries =====
    
    /// Is this panel closable by the user?
    fn is_closable(&self) -> bool {
        true
    }
    
    /// Is this panel pausable?
    fn is_pausable(&self) -> bool {
        true
    }
}

/// Registry of available panel types
pub struct PanelRegistry {
    factories: HashMap<String, Arc<dyn PanelFactory>>,
}

impl PanelRegistry {
    /// Create a new panel registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
    
    /// Register a panel factory
    pub fn register(&mut self, factory: Arc<dyn PanelFactory>) {
        let panel_type = factory.panel_type().to_string();
        tracing::info!(
            "Registering panel type: {} - {}",
            panel_type,
            factory.description()
        );
        self.factories.insert(panel_type, factory);
    }
    
    /// Create a panel instance from configuration
    pub fn create(&self, config: &CustomPanelConfig) -> Result<Box<dyn PanelInstance>> {
        let factory = self.factories
            .get(&config.panel_type)
            .ok_or_else(|| PanelError::UnknownType(config.panel_type.clone()))?;
        
        tracing::info!("Creating panel: {} (type: {})", config.title, config.panel_type);
        factory.create(config)
    }
    
    /// Get list of registered panel types
    pub fn available_types(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }
    
    /// Check if a panel type is registered
    pub fn has_type(&self, panel_type: &str) -> bool {
        self.factories.contains_key(panel_type)
    }
}

impl Default for PanelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockPanelFactory;
    
    impl PanelFactory for MockPanelFactory {
        fn panel_type(&self) -> &str {
            "mock_panel"
        }
        
        fn create(&self, _config: &CustomPanelConfig) -> Result<Box<dyn PanelInstance>> {
            Ok(Box::new(MockPanel))
        }
    }
    
    struct MockPanel;
    
    impl PanelInstance for MockPanel {
        fn render(&mut self, ui: &mut egui::Ui) {
            ui.label("Mock Panel");
        }
        
        fn title(&self) -> &str {
            "Mock"
        }
    }
    
    #[test]
    fn test_panel_registration() {
        let mut registry = PanelRegistry::new();
        registry.register(Arc::new(MockPanelFactory));
        
        assert!(registry.has_type("mock_panel"));
        assert!(!registry.has_type("unknown"));
    }
    
    #[test]
    fn test_panel_creation() {
        let mut registry = PanelRegistry::new();
        registry.register(Arc::new(MockPanelFactory));
        
        let config = CustomPanelConfig {
            panel_type: "mock_panel".to_string(),
            title: "Test".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        
        let panel = registry.create(&config).unwrap();
        assert_eq!(panel.title(), "Mock");
    }
    
    #[test]
    fn test_unknown_panel_type() {
        let registry = PanelRegistry::new();
        
        let config = CustomPanelConfig {
            panel_type: "unknown".to_string(),
            title: "Test".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        
        assert!(registry.create(&config).is_err());
    }
}

