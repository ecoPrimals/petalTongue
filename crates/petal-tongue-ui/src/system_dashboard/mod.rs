// SPDX-License-Identifier: AGPL-3.0-or-later
//! System Dashboard Sidebar
//!
//! Compact live system metrics always visible in the main UI.
//! Now with multimodal output (visual + audio + text).

mod panels;
mod state;

#[cfg(test)]
mod tests;

// Re-export public API
pub use state::SystemDashboard;
