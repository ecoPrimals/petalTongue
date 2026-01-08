//! # petalTongue Modalities
//! 
//! Concrete implementations of GUI modalities for petalTongue.
//! 
//! ## Available Modalities
//! 
//! ### Tier 1: Always Available (Zero Dependencies)
//! - **TerminalGUI** - ASCII visualization in terminal ✅
//! - **SVGGUI** - Vector export to SVG files ✅
//! - **JSONGUI** - Data export to JSON files (planned)
//! 
//! ### Tier 2: Default Available (Minimal Dependencies)
//! - **PNGGUI** - Raster export to PNG files ✅
//! 
//! ### Tier 3: Enhancement (Optional)
//! - **EguiGUI** - Native GUI with egui (planned refactor)
//! 
//! ## Philosophy
//! 
//! Each modality is a complete, self-contained representation of the
//! topology data. They receive events from the EventBus and can render
//! independently or simultaneously.

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod png_gui;
pub mod svg_gui;
pub mod terminal_gui;

// Re-exports
pub use png_gui::PNGGUI;
pub use svg_gui::SVGGUI;
pub use terminal_gui::TerminalGUI;

#[cfg(feature = "egui-gui")]
pub mod egui_gui;

#[cfg(feature = "egui-gui")]
pub use egui_gui::EguiGUI;

