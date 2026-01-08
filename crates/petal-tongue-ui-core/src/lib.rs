//! Pure Rust UI generation for petalTongue
//!
//! This crate provides platform-agnostic UI generation with ZERO native dependencies.
//! It implements the three-tier UI architecture:
//!
//! - **Tier 1** (this crate): Pure Rust UI generation (terminal, SVG, text, canvas)
//! - **Tier 2**: Native GUI enhancements (egui - optional)
//! - **Tier 3**: Web/advanced (WebAssembly - optional)
//!
//! # Philosophy
//!
//! Just as petalTongue generates pure Rust audio, it generates pure Rust UI.
//! External systems are enhancements, not dependencies.
//!
//! # Features
//!
//! - **Terminal UI**: Interactive TUI using crossterm (works over SSH!)
//! - **SVG Export**: Pure Rust SVG generation (browser-friendly)
//! - **Text Export**: Plain text, JSON, DOT (automation-friendly)
//! - **Canvas Rendering**: Pixel-perfect rendering with tiny-skia (headless-friendly)
//!
//! # Examples
//!
//! ```no_run
//! use petal_tongue_ui_core::{UniversalUI, SvgUI, ExportFormat};
//! use petal_tongue_core::GraphEngine;
//! use std::sync::{Arc, RwLock};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Create graph
//! let graph = Arc::new(RwLock::new(GraphEngine::new()));
//!
//! // Generate SVG (works anywhere!)
//! let ui = SvgUI::new(graph.clone(), 1920, 1080);
//! let svg = ui.render_to_string()?;
//! # Ok(())
//! # }
//! ```

pub mod canvas;
pub mod svg;
pub mod terminal;
pub mod text;
pub mod trait_def;
pub mod utils;

// Re-exports
pub use canvas::CanvasUI;
pub use svg::SvgUI;
pub use terminal::TerminalUI;
pub use text::TextUI;
pub use trait_def::{ExportFormat, UICapability, UniversalUI};
pub use utils::*;

/// Detect the best UI mode for the current environment
pub fn detect_best_ui_mode() -> UIMode {
    let has_display = std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows");

    let is_terminal = atty::is(atty::Stream::Stdout);
    let is_headless = std::env::var("HEADLESS").is_ok()
        || std::env::var("CI").is_ok()
        || std::env::var("PETALTONGUE_HEADLESS").is_ok();

    if is_headless {
        tracing::info!("Detected headless environment");
        UIMode::Headless
    } else if is_terminal && !has_display {
        tracing::info!("Detected terminal environment");
        UIMode::Terminal
    } else if has_display {
        tracing::info!("Detected display environment");
        UIMode::Display
    } else {
        tracing::info!("Defaulting to headless mode");
        UIMode::Headless
    }
}

/// UI execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIMode {
    /// Headless mode (export to files)
    Headless,
    /// Terminal mode (TUI)
    Terminal,
    /// Display mode (can use GUI if available)
    Display,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_mode_detection() {
        // Should always return a valid mode
        let mode = detect_best_ui_mode();
        assert!(matches!(
            mode,
            UIMode::Headless | UIMode::Terminal | UIMode::Display
        ));
    }
}
