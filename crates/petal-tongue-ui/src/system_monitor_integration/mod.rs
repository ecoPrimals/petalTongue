// SPDX-License-Identifier: AGPL-3.0-or-later
//! System Monitor Integration
//!
//! Real-time system resource monitoring via /proc parsing (ecoBin v3.0 compliant).
//! Demonstrates petalTongue integrating with external monitoring tool.
//! ALL DATA IS LIVE - timestamps and indicators prove it!
//!
//! Architecture: headless-first. All computation lives in pure functions
//! (`threshold_color`, `compute_sparkline_points`, `prepare_cpu_display`,
//! `prepare_memory_display`) that produce testable `DisplayState` structs.
//! Render methods are thin egui widget calls with zero logic.

#![expect(
    clippy::cast_precision_loss,
    reason = "system monitor metrics use numeric casts for display; precision loss acceptable"
)]

mod display_compute;
mod display_state;
mod tool;

#[cfg(test)]
mod tests;

pub use display_compute::{
    compute_sparkline_points, format_gb, prepare_cpu_display, prepare_memory_display,
    threshold_color,
};
pub use display_state::{CpuDisplayState, MemoryDisplayState};
pub use tool::SystemMonitorTool;
