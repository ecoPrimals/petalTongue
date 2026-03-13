// SPDX-License-Identifier: AGPL-3.0-only
//! Traffic View - Flow Analysis and Sankey Diagram
//!
//! Visualizes data flow and traffic patterns between primals.
//! Implements Phase 4 of the UI specification.

mod types;
mod view;

#[cfg(test)]
mod tests;

pub use types::{ColorScheme, TrafficFlow, TrafficMetrics};
pub use view::{
    TrafficIntent, TrafficView, bezier_control_points, calculate_flow_color, calculate_flow_width,
    prepare_flow_detail, primal_lane_layout,
};
