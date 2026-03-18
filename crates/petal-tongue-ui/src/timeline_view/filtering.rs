// SPDX-License-Identifier: AGPL-3.0-or-later
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
    event_type_filter: Option<&String>,
    primal_filter: Option<&String>,
    time_range_start: Option<&DateTime<Utc>>,
    time_range_end: Option<&DateTime<Utc>>,
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

#[cfg(test)]
mod tests {
    use super::super::types::{EventStatus, TimelineEvent};
    use super::*;

    fn mock_event(
        id: &str,
        from: &str,
        to: &str,
        event_type: &str,
        timestamp: DateTime<Utc>,
    ) -> TimelineEvent {
        TimelineEvent {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            event_type: event_type.to_string(),
            timestamp,
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        }
    }

    #[test]
    fn filtered_events_no_filters_returns_all() {
        let now = Utc::now();
        let events = vec![
            mock_event("1", "a", "b", "discover", now),
            mock_event("2", "b", "c", "invoke", now),
        ];
        let result = filtered_events(&events, None, None, None, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filtered_events_event_type_filter() {
        let now = Utc::now();
        let events = vec![
            mock_event("1", "a", "b", "discover", now),
            mock_event("2", "b", "c", "invoke", now),
        ];
        let result = filtered_events(&events, Some(&"discover".to_string()), None, None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_type, "discover");
    }

    #[test]
    fn filtered_events_primal_filter() {
        let now = Utc::now();
        let events = vec![
            mock_event("1", "alice", "bob", "test", now),
            mock_event("2", "bob", "charlie", "test", now),
        ];
        let result = filtered_events(&events, None, Some(&"alice".to_string()), None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].from, "alice");
    }

    #[test]
    fn filtered_events_time_range() {
        let base = Utc::now();
        let events = vec![
            mock_event("1", "a", "b", "test", base - chrono::Duration::seconds(20)),
            mock_event("2", "a", "b", "test", base),
            mock_event("3", "a", "b", "test", base + chrono::Duration::seconds(20)),
        ];
        let start = base - chrono::Duration::seconds(10);
        let end = base + chrono::Duration::seconds(10);
        let result = filtered_events(&events, None, None, Some(&start), Some(&end));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "2");
    }

    #[test]
    fn get_primals_unique_sorted() {
        let now = Utc::now();
        let events = vec![
            mock_event("1", "charlie", "alice", "test", now),
            mock_event("2", "bob", "charlie", "test", now),
        ];
        let primals = get_primals(&events);
        assert_eq!(primals, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn get_primals_empty() {
        let events: Vec<TimelineEvent> = vec![];
        let primals = get_primals(&events);
        assert!(primals.is_empty());
    }
}
