// SPDX-License-Identifier: AGPL-3.0-only
//! # petalTongue Rich TUI
//!
//! **Universal Terminal User Interface** for biomeOS and ecoPrimals.
//!
//! This crate provides a rich, interactive terminal UI that can:
//! - Run as a standalone UI (like `PopOS`)
//! - Run on top of existing OS (SSH, headless)
//! - Manage neuralAPI, NUCLEUS, liveSpore
//! - Adapt to any user (human, AI, non-human)
//! - Work in any universe (OS, cloud, fractal)
//!
//! ## Features
//!
//! - **8 Interactive Views**: Dashboard, Topology, Devices, Primals, Logs, neuralAPI, NUCLEUS, `LiveSpore`
//! - **Real-Time Updates**: Live WebSocket/JSON-RPC integration
//! - **Keyboard Navigation**: Full keyboard control
//! - **Mouse Support**: Optional mouse interaction
//! - **ASCII Art Topology**: Beautiful graph visualization
//! - **Pure Rust**: Zero C dependencies
//! - **Accessibility**: Screen reader compatible
//!
//! ## TRUE PRIMAL Principles
//!
//! - **Zero Hardcoding**: Runtime discovery of all capabilities
//! - **Capability-Based**: Adapts to available primals
//! - **Graceful Degradation**: Works with 0-N primals
//! - **Self-Knowledge**: Knows own capabilities
//! - **Universal**: Works in any computational universe

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Application core
pub mod app;
/// Error types
pub mod error;
/// Event system
pub mod events;
/// Layout utilities
pub mod layout;
/// Scene engine bridge: renders `petal-tongue-scene` graphs to ratatui widgets
pub mod scene_bridge;
/// State management
pub mod state;
/// Interactive views
pub mod views;
/// Reusable widgets
pub mod widgets;

// Re-exports for convenience
pub use app::{RichTUI, TUIConfig};
pub use state::{TUIState, View};

/// Launch the TUI with default configuration
///
/// # Example
///
/// ```no_run
/// use petal_tongue_tui::launch;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     launch().await?;
///     Ok(())
/// }
/// ```
pub async fn launch() -> Result<(), error::TuiError> {
    let mut tui = RichTUI::new().await?;
    tui.run().await
}

/// Launch the TUI with custom configuration
///
/// # Example
///
/// ```no_run
/// use petal_tongue_tui::{launch_with_config, TUIConfig};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = TUIConfig {
///         tick_rate: Duration::from_millis(50),
///         mouse_support: true,
///         standalone: false,
///     };
///     launch_with_config(config).await?;
///     Ok(())
/// }
/// ```
pub async fn launch_with_config(config: TUIConfig) -> Result<(), error::TuiError> {
    let mut tui = RichTUI::with_config(config).await?;
    tui.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_config_accepts_standalone() {
        let config = TUIConfig {
            tick_rate: std::time::Duration::from_millis(100),
            mouse_support: false,
            standalone: true,
        };
        assert!(config.standalone);
    }

    #[test]
    fn tui_config_reexport() {
        let _config = TUIConfig::default();
    }

    #[test]
    fn view_reexport() {
        let _ = View::all();
    }
}
