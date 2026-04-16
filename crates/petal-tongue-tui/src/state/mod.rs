// SPDX-License-Identifier: AGPL-3.0-or-later
//! TUI State Management
//!
//! Central state for the Rich TUI. All state is managed here,
//! with zero global state or unsafe code.

mod tui_state;
mod types;

#[cfg(test)]
mod tests;

pub use tui_state::TUIState;
pub use types::{LogLevel, LogMessage, SystemStatus, TUIStats, View};
