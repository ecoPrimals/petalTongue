//! # petalTongue Rich TUI
//!
//! **Universal Terminal User Interface** for biomeOS and ecoPrimals.
//!
//! This crate provides a rich, interactive terminal UI that can:
//! - Run as a standalone UI (like PopOS)
//! - Run on top of existing OS (SSH, headless)
//! - Manage neuralAPI, NUCLEUS, liveSpore
//! - Adapt to any user (human, AI, non-human)
//! - Work in any universe (OS, cloud, fractal)
//!
//! ## Features
//!
//! - **8 Interactive Views**: Dashboard, Topology, Devices, Primals, Logs, neuralAPI, NUCLEUS, LiveSpore
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
#![deny(unsafe_code)]

pub mod app;
pub mod views;
pub mod widgets;
pub mod state;
pub mod events;
pub mod layout;

// Re-exports
pub use app::RichTUI;
pub use state::TUIState;
pub use views::View;

use anyhow::Result;

/// Launch the Rich TUI
///
/// This is the main entry point for the terminal UI.
///
/// # Example
///
/// ```no_run
/// use petal_tongue_tui;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     petal_tongue_tui::launch().await
/// }
/// ```
pub async fn launch() -> Result<()> {
    let mut tui = RichTUI::new().await?;
    tui.run().await
}

/// Launch with custom configuration
pub async fn launch_with_config(config: app::TUIConfig) -> Result<()> {
    let mut tui = RichTUI::with_config(config).await?;
    tui.run().await
}

