// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traffic View - Flow Analysis and Sankey Diagram
//!
//! Visualizes data flow and traffic patterns between primals.
//! Implements Phase 4 of the UI specification.

mod helpers;
mod render;
mod types;
mod view;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_extended;

pub use helpers::{
    bezier_control_points, calculate_flow_color, calculate_flow_width, prepare_flow_detail,
    primal_lane_layout,
};
pub use types::{ColorScheme, TrafficFlow, TrafficIntent, TrafficMetrics};
pub use view::TrafficView;
