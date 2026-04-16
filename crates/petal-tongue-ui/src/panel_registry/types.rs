// SPDX-License-Identifier: AGPL-3.0-or-later
//! Panel registry types: errors and the [`PanelInstance`] trait.

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
    /// The requested panel type is not registered in the registry
    #[error("Unknown panel type: {0}")]
    UnknownType(String),

    /// Panel instantiation failed (e.g., missing resources, initialization error)
    #[error("Panel creation failed: {0}")]
    CreationFailed(String),

    /// The panel configuration is invalid or missing required fields
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for panel operations
pub type Result<T> = std::result::Result<T, PanelError>;

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
    /// Returns `InputAction` indicating if input was consumed
    fn on_keyboard_event(&mut self, _ctx: &egui::Context) -> crate::focus_manager::InputAction {
        crate::focus_manager::InputAction::Ignored
    }

    /// Handle mouse event
    /// Returns `InputAction` indicating if input was consumed
    fn on_mouse_event(&mut self, _ctx: &egui::Context) -> crate::focus_manager::InputAction {
        crate::focus_manager::InputAction::Ignored
    }

    // ===== Lifecycle Hooks (Phase 4 Evolution) =====

    /// Called when panel is first opened/created
    /// Use this to initialize resources (load assets, create connections, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if resource initialization fails.
    fn on_open(&mut self) -> crate::error::Result<()> {
        Ok(())
    }

    /// Called when panel is about to close
    /// Use this to clean up resources (close files, disconnect, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails.
    fn on_close(&mut self) -> crate::error::Result<()> {
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
    /// Return `PanelAction` to indicate what should happen next
    fn on_error(&mut self, error: &dyn std::error::Error) -> PanelAction {
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
    /// Only called if `can_save_state()` returns true
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    fn save_state(&self) -> crate::error::Result<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }

    /// Restore panel state from JSON
    /// Only called if `can_restore_state()` returns true
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization or state restoration fails.
    fn restore_state(&mut self, _state: serde_json::Value) -> crate::error::Result<()> {
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
