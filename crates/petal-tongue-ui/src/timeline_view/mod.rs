// SPDX-License-Identifier: AGPL-3.0-only
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

pub use helpers::{
    escape_csv, format_events_csv, prepare_event_detail, time_to_x, zoom_in, zoom_out,
};
pub use types::{EventDetailDisplay, EventStatus, TimelineEvent, TimelineIntent};
pub use view::TimelineView;
