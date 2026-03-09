// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Event filtering and data extraction
//!
//! Filtering logic for timeline events by type, primal, and time range.

use chrono::{DateTime, Utc};
use std::collections::HashSet;

use super::types::TimelineEvent;

/// Filter events by event type, primal, and time range.
#[must_use]
pub fn filtered_events<'a>(
    events: &'a [TimelineEvent],
    event_type_filter: &Option<String>,
    primal_filter: &Option<String>,
    time_range_start: &Option<DateTime<Utc>>,
    time_range_end: &Option<DateTime<Utc>>,
) -> Vec<&'a TimelineEvent> {
    events
        .iter()
        .filter(|e| {
            // Apply event type filter
            if let Some(filter) = event_type_filter
                && &e.event_type != filter
            {
                return false;
            }

            // Apply primal filter
            if let Some(filter) = primal_filter
                && &e.from != filter
                && &e.to != filter
            {
                return false;
            }

            // Apply time range filter
            if let Some(start) = time_range_start
                && e.timestamp < *start
            {
                return false;
            }
            if let Some(end) = time_range_end
                && e.timestamp > *end
            {
                return false;
            }

            true
        })
        .collect()
}

/// Get list of unique primals involved in events.
#[must_use]
pub fn get_primals(events: &[TimelineEvent]) -> Vec<String> {
    let mut primals = HashSet::new();
    for event in events {
        primals.insert(event.from.clone());
        primals.insert(event.to.clone());
    }
    let mut primal_vec: Vec<_> = primals.into_iter().collect();
    primal_vec.sort();
    primal_vec
}
