// SPDX-License-Identifier: AGPL-3.0-or-later
//! Timeline View - Event Sequence Visualization
//!
//! Displays temporal sequences of primal interactions with time scrubbing capabilities.
//! Implements Phase 4 of the UI specification.

mod filtering;
mod helpers;
mod types;
mod view;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_extended;
#[cfg(test)]
mod tests_rendering;

pub use helpers::{
    build_primal_lanes, compute_lane_height, escape_csv, event_screen_rect, format_events_csv,
    prepare_event_detail, time_to_x, zoom_in, zoom_out,
};
pub use types::{EventDetailDisplay, EventStatus, TimelineEvent, TimelineIntent};
pub use view::TimelineView;
