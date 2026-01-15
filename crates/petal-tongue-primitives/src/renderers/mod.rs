//! Renderer Implementations
//!
//! This module contains the actual renderer implementations for different modalities.

#[cfg(feature = "egui")]
pub mod egui_tree;

#[cfg(feature = "ratatui")]
pub mod ratatui_tree;

#[cfg(feature = "egui")]
pub mod egui_table;

#[cfg(feature = "ratatui")]
pub mod ratatui_table;

#[cfg(feature = "egui")]
pub mod egui_panel;

#[cfg(feature = "ratatui")]
pub mod ratatui_panel;

#[cfg(feature = "egui")]
pub mod egui_command_palette;

#[cfg(feature = "ratatui")]
pub mod ratatui_command_palette;

#[cfg(feature = "egui")]
pub mod egui_form;

#[cfg(feature = "ratatui")]
pub mod ratatui_form;

// Re-exports for convenience
#[cfg(feature = "egui")]
pub use egui_tree::EguiTreeRenderer;

#[cfg(feature = "ratatui")]
pub use ratatui_tree::RatatuiTreeRenderer;

#[cfg(feature = "egui")]
pub use egui_table::EguiTableRenderer;

#[cfg(feature = "ratatui")]
pub use ratatui_table::RatatuiTableRenderer;

#[cfg(feature = "egui")]
pub use egui_panel::EguiPanelRenderer;

#[cfg(feature = "ratatui")]
pub use ratatui_panel::RatatuiPanelRenderer;

#[cfg(feature = "egui")]
pub use egui_command_palette::EguiCommandPaletteRenderer;

#[cfg(feature = "ratatui")]
pub use ratatui_command_palette::RatatuiCommandPaletteRenderer;

#[cfg(feature = "egui")]
pub use egui_form::EguiFormRenderer;

#[cfg(feature = "ratatui")]
pub use ratatui_form::RatatuiFormRenderer;
