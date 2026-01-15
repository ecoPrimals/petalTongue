//! Input focus management for panels
//!
//! This module provides explicit input routing to panels, solving the problem
//! of "who gets keyboard input when multiple panels are active?"
//!
//! ## Architecture
//!
//! - **FocusManager**: Tracks which panel has focus
//! - **Focus Stack**: Priority-based input routing
//! - **Input Actions**: Panels declare if they consumed input
//! - **Exclusive Mode**: Games can request exclusive input

use std::collections::HashMap;

/// Result of input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    /// Panel consumed the input, don't pass to others
    Consumed,
    
    /// Panel didn't handle the input, try next panel
    Ignored,
    
    /// Global shortcut, all panels should see it
    Global,
}

/// Input focus manager
///
/// Manages which panel receives keyboard and mouse input.
/// Implements a focus stack for priority-based routing.
pub struct FocusManager {
    /// Currently focused panel ID
    focused_panel: Option<String>,
    
    /// Focus stack (highest priority first)
    focus_stack: Vec<String>,
    
    /// Panel input preferences
    panel_preferences: HashMap<String, PanelInputPreferences>,
}

/// Panel input preferences
#[derive(Debug, Clone)]
pub struct PanelInputPreferences {
    /// Panel wants keyboard input
    pub wants_keyboard: bool,
    
    /// Panel wants mouse input  
    pub wants_mouse: bool,
    
    /// Panel wants exclusive input (blocks all other panels)
    pub wants_exclusive: bool,
    
    /// Panel priority (higher = gets input first)
    pub priority: u8,
}

impl Default for PanelInputPreferences {
    fn default() -> Self {
        Self {
            wants_keyboard: false,
            wants_mouse: false,
            wants_exclusive: false,
            priority: 5, // Medium priority
        }
    }
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            focused_panel: None,
            focus_stack: Vec::new(),
            panel_preferences: HashMap::new(),
        }
    }
    
    /// Register a panel with its input preferences
    pub fn register_panel(&mut self, id: String, prefs: PanelInputPreferences) {
        // Insert into focus stack based on priority
        let insert_pos = self.focus_stack.iter()
            .position(|panel_id| {
                self.panel_preferences.get(panel_id)
                    .map(|p| p.priority < prefs.priority)
                    .unwrap_or(true)
            })
            .unwrap_or(self.focus_stack.len());
        
        self.focus_stack.insert(insert_pos, id.clone());
        self.panel_preferences.insert(id, prefs);
    }
    
    /// Unregister a panel
    pub fn unregister_panel(&mut self, id: &str) {
        self.focus_stack.retain(|panel_id| panel_id != id);
        self.panel_preferences.remove(id);
        
        if self.focused_panel.as_deref() == Some(id) {
            self.focused_panel = None;
        }
    }
    
    /// Set focused panel explicitly
    pub fn set_focus(&mut self, id: Option<String>) {
        self.focused_panel = id;
    }
    
    /// Get currently focused panel
    pub fn focused_panel(&self) -> Option<&str> {
        self.focused_panel.as_deref()
    }
    
    /// Get panels that want keyboard input (in priority order)
    pub fn keyboard_interested_panels(&self) -> Vec<&str> {
        self.focus_stack.iter()
            .filter(|id| {
                self.panel_preferences.get(*id)
                    .map(|p| p.wants_keyboard)
                    .unwrap_or(false)
            })
            .map(|s| s.as_str())
            .collect()
    }
    
    /// Get panels that want mouse input (in priority order)
    pub fn mouse_interested_panels(&self) -> Vec<&str> {
        self.focus_stack.iter()
            .filter(|id| {
                self.panel_preferences.get(*id)
                    .map(|p| p.wants_mouse)
                    .unwrap_or(false)
            })
            .map(|s| s.as_str())
            .collect()
    }
    
    /// Check if any panel wants exclusive input
    pub fn has_exclusive_panel(&self) -> bool {
        self.panel_preferences.values()
            .any(|prefs| prefs.wants_exclusive)
    }
    
    /// Get exclusive panel (if any)
    pub fn exclusive_panel(&self) -> Option<&str> {
        self.focus_stack.iter()
            .find(|id| {
                self.panel_preferences.get(*id)
                    .map(|p| p.wants_exclusive)
                    .unwrap_or(false)
            })
            .map(|s| s.as_str())
    }
    
    /// Update panel preferences
    pub fn update_preferences(&mut self, id: &str, prefs: PanelInputPreferences) {
        if let Some(existing) = self.panel_preferences.get_mut(id) {
            *existing = prefs.clone();
            
            // Re-sort focus stack by priority
            self.focus_stack.sort_by(|a, b| {
                let a_priority = self.panel_preferences.get(a).map(|p| p.priority).unwrap_or(0);
                let b_priority = self.panel_preferences.get(b).map(|p| p.priority).unwrap_or(0);
                b_priority.cmp(&a_priority) // Higher priority first
            });
        }
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_focus_manager_creation() {
        let manager = FocusManager::new();
        assert_eq!(manager.focused_panel(), None);
        assert_eq!(manager.keyboard_interested_panels().len(), 0);
    }
    
    #[test]
    fn test_register_panel() {
        let mut manager = FocusManager::new();
        
        let prefs = PanelInputPreferences {
            wants_keyboard: true,
            wants_mouse: false,
            wants_exclusive: false,
            priority: 5,
        };
        
        manager.register_panel("doom".to_string(), prefs);
        
        let panels = manager.keyboard_interested_panels();
        assert_eq!(panels.len(), 1);
        assert_eq!(panels[0], "doom");
    }
    
    #[test]
    fn test_priority_ordering() {
        let mut manager = FocusManager::new();
        
        // Register panels with different priorities
        manager.register_panel("low".to_string(), PanelInputPreferences {
            wants_keyboard: true,
            priority: 3,
            ..Default::default()
        });
        
        manager.register_panel("high".to_string(), PanelInputPreferences {
            wants_keyboard: true,
            priority: 8,
            ..Default::default()
        });
        
        manager.register_panel("medium".to_string(), PanelInputPreferences {
            wants_keyboard: true,
            priority: 5,
            ..Default::default()
        });
        
        let panels = manager.keyboard_interested_panels();
        assert_eq!(panels.len(), 3);
        assert_eq!(panels[0], "high");
        assert_eq!(panels[1], "medium");
        assert_eq!(panels[2], "low");
    }
    
    #[test]
    fn test_exclusive_panel() {
        let mut manager = FocusManager::new();
        
        manager.register_panel("doom".to_string(), PanelInputPreferences {
            wants_keyboard: true,
            wants_exclusive: true,
            priority: 10,
            ..Default::default()
        });
        
        manager.register_panel("graph".to_string(), PanelInputPreferences {
            wants_keyboard: true,
            wants_exclusive: false,
            priority: 5,
            ..Default::default()
        });
        
        assert!(manager.has_exclusive_panel());
        assert_eq!(manager.exclusive_panel(), Some("doom"));
    }
    
    #[test]
    fn test_unregister_panel() {
        let mut manager = FocusManager::new();
        
        manager.register_panel("doom".to_string(), PanelInputPreferences {
            wants_keyboard: true,
            ..Default::default()
        });
        
        manager.set_focus(Some("doom".to_string()));
        assert_eq!(manager.focused_panel(), Some("doom"));
        
        manager.unregister_panel("doom");
        assert_eq!(manager.focused_panel(), None);
        assert_eq!(manager.keyboard_interested_panels().len(), 0);
    }
    
    #[test]
    fn test_update_preferences() {
        let mut manager = FocusManager::new();
        
        manager.register_panel("doom".to_string(), PanelInputPreferences {
            wants_keyboard: false,
            priority: 5,
            ..Default::default()
        });
        
        assert_eq!(manager.keyboard_interested_panels().len(), 0);
        
        // Update to want keyboard
        manager.update_preferences("doom", PanelInputPreferences {
            wants_keyboard: true,
            priority: 5,
            ..Default::default()
        });
        
        assert_eq!(manager.keyboard_interested_panels().len(), 1);
    }
    
    #[test]
    fn test_focus_set_and_get() {
        let mut manager = FocusManager::new();
        
        manager.register_panel("doom".to_string(), PanelInputPreferences::default());
        
        manager.set_focus(Some("doom".to_string()));
        assert_eq!(manager.focused_panel(), Some("doom"));
        
        manager.set_focus(None);
        assert_eq!(manager.focused_panel(), None);
    }
}

