// SPDX-License-Identifier: AGPL-3.0-or-later
//! Metrics Dashboard - Real-time System and Neural API Metrics
//!
//! Displays CPU, memory, uptime, and Neural API statistics with sparklines.
//! Updates automatically every 5 seconds with fresh data from Neural API.

mod render;
mod state;

#[cfg(test)]
mod tests;

pub use state::MetricsDashboard;
