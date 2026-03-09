// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Event Sequence Visualization
//!
//! Displays temporal sequences of primal interactions with time scrubbing capabilities.
//! Implements Phase 4 of the UI specification.

mod filtering;
mod types;
mod view;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{EventStatus, TimelineEvent};
pub use view::TimelineView;
