// SPDX-License-Identifier: AGPL-3.0-or-later
//! Main TUI Application
//!
//! Core application logic for the Rich TUI.
//! Zero unsafe code, pure async, capability-based.

mod config;
mod render;
mod tui;
mod update;

#[cfg(test)]
mod tests;

pub use config::TUIConfig;
pub use tui::RichTUI;
