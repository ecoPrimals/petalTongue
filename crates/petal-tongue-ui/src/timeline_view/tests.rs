// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Unit tests

use chrono::Utc;

use super::types::{EventStatus, TimelineEvent};
use super::view::TimelineView;

#[test]
fn test_timeline_view_creation() {
    let view = TimelineView::new();
    assert_eq!(view.filtered_event_count(), 0);
    assert!(view.get_primals_for_test().is_empty());
}

#[test]
fn test_add_event() {
    let mut view = TimelineView::new();

    let event = TimelineEvent {
        id: "evt1".to_string(),
        from: "primal1".to_string(),
        to: "primal2".to_string(),
        event_type: "discover".to_string(),
        timestamp: Utc::now(),
        duration_ms: Some(10.5),
        status: EventStatus::Success,
        payload_summary: None,
    };

    view.add_event(event);
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn test_clear_events() {
    let mut view = TimelineView::new();

    // Add some events
    for i in 0..5 {
        view.add_event(TimelineEvent {
            id: format!("evt{i}"),
            from: "primal1".to_string(),
            to: "primal2".to_string(),
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });
    }

    assert_eq!(view.filtered_event_count(), 5);

    view.clear();
    assert_eq!(view.filtered_event_count(), 0);
    assert!(view.get_primals_for_test().is_empty());
}

#[test]
fn test_get_primals() {
    let mut view = TimelineView::new();

    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    view.add_event(TimelineEvent {
        id: "evt2".to_string(),
        from: "bob".to_string(),
        to: "charlie".to_string(),
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    let primals = view.get_primals_for_test();
    assert_eq!(primals.len(), 3);
    assert!(primals.contains(&"alice".to_string()));
    assert!(primals.contains(&"bob".to_string()));
    assert!(primals.contains(&"charlie".to_string()));
}

#[test]
fn test_event_status_colors() {
    assert_ne!(EventStatus::Success.color(), EventStatus::Failure.color());
    assert_ne!(
        EventStatus::Success.color(),
        EventStatus::InProgress.color()
    );
    assert_ne!(EventStatus::Failure.color(), EventStatus::Timeout.color());
}

#[test]
fn test_event_sorting() {
    let mut view = TimelineView::new();

    let now = Utc::now();

    // Add events out of order
    view.add_event(TimelineEvent {
        id: "evt3".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: now + chrono::Duration::seconds(3),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: now + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    view.add_event(TimelineEvent {
        id: "evt2".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: now + chrono::Duration::seconds(2),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    // Events should be sorted by timestamp
    assert_eq!(view.event_ids_ordered(), ["evt1", "evt2", "evt3"]);
}

#[test]
fn test_event_type_filter() {
    let mut view = TimelineView::new();
    let now = Utc::now();

    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "discover".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "evt2".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "invoke".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    assert_eq!(view.filtered_event_count(), 2);
    view.set_event_type_filter(Some("discover".to_string()));
    assert_eq!(view.filtered_event_count(), 1);
    view.set_event_type_filter(None);
    assert_eq!(view.filtered_event_count(), 2);
}

#[test]
fn test_primal_filter() {
    let mut view = TimelineView::new();
    let now = Utc::now();

    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "test".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "evt2".to_string(),
        from: "bob".to_string(),
        to: "charlie".to_string(),
        event_type: "test".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    view.set_primal_filter(Some("alice".to_string()));
    assert_eq!(view.filtered_event_count(), 1);
    view.set_primal_filter(Some("bob".to_string()));
    assert_eq!(view.filtered_event_count(), 2);
    view.set_primal_filter(None);
    assert_eq!(view.filtered_event_count(), 2);
}

#[test]
fn test_event_status_icons() {
    assert_eq!(EventStatus::Success.icon(), "✅");
    assert_eq!(EventStatus::Failure.icon(), "❌");
    assert_eq!(EventStatus::InProgress.icon(), "⏳");
    assert_eq!(EventStatus::Timeout.icon(), "⏱️");
}

#[test]
fn test_time_range_filter() {
    let mut view = TimelineView::new();
    let now = Utc::now();
    let start = now - chrono::Duration::seconds(10);
    let end = now + chrono::Duration::seconds(10);

    // Add events: one before range, one in range, one after range
    view.add_event(TimelineEvent {
        id: "evt_before".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: now - chrono::Duration::seconds(20),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "evt_in".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "evt_after".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: now + chrono::Duration::seconds(20),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    assert_eq!(view.filtered_event_count(), 3);

    view.set_time_range(Some(start), Some(end));
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn test_set_time_range() {
    let mut view = TimelineView::new();
    let start = Utc::now() - chrono::Duration::hours(1);
    let end = Utc::now();
    view.set_time_range(Some(start), Some(end));
    // Just verify it doesn't panic
    assert_eq!(view.filtered_event_count(), 0);
}

#[test]
fn test_default_timeline_view() {
    let view = TimelineView::default();
    assert_eq!(view.filtered_event_count(), 0);
}
