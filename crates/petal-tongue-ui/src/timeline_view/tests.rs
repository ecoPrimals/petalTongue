// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Unit tests

use chrono::Utc;

use super::{
    EventStatus, TimelineEvent, TimelineIntent, TimelineView, escape_csv, format_events_csv,
    time_to_x, zoom_in, zoom_out,
};

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

#[test]
fn test_event_status_all_variants() {
    assert_eq!(EventStatus::Success.icon(), "✅");
    assert_eq!(EventStatus::Failure.icon(), "❌");
    assert_eq!(EventStatus::InProgress.icon(), "⏳");
    assert_eq!(EventStatus::Timeout.icon(), "⏱️");
}

#[test]
fn test_event_with_payload() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        duration_ms: Some(42.5),
        status: EventStatus::Success,
        payload_summary: Some("payload data".to_string()),
    });
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn test_event_without_duration() {
    let event = TimelineEvent {
        id: "evt1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: Utc::now(),
        duration_ms: None,
        status: EventStatus::Failure,
        payload_summary: None,
    };
    assert!(event.duration_ms.is_none());
    assert!(event.payload_summary.is_none());
}

#[test]
fn test_combined_filters() {
    let mut view = TimelineView::new();
    let now = Utc::now();

    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "evt2".to_string(),
        from: "bob".to_string(),
        to: "charlie".to_string(),
        event_type: "invoke".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    view.set_event_type_filter(Some("discover".to_string()));
    view.set_primal_filter(Some("alice".to_string()));
    assert_eq!(view.filtered_event_count(), 1);

    view.set_primal_filter(Some("charlie".to_string()));
    assert_eq!(view.filtered_event_count(), 0);
}

#[test]
fn test_event_ids_ordered_empty() {
    let view = TimelineView::new();
    assert!(view.event_ids_ordered().is_empty());
}

#[test]
fn test_multiple_primals() {
    let mut view = TimelineView::new();
    let now = Utc::now();

    for i in 0..5 {
        view.add_event(TimelineEvent {
            id: format!("evt{i}"),
            from: format!("primal{i}"),
            to: format!("primal{}", i + 1),
            event_type: "test".to_string(),
            timestamp: now + chrono::Duration::seconds(i),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });
    }

    let primals = view.get_primals_for_test();
    assert_eq!(primals.len(), 6);
}

// === Additional coverage: pure functions, intents, edge cases ===

#[test]
fn test_time_to_x_negative_time() {
    let x = time_to_x(50.0, 100.0, 200.0, 500.0);
    assert!(x.abs() < f32::EPSILON);
}

#[test]
fn test_escape_csv_carriage_return() {
    assert_eq!(escape_csv("line1\rline2"), "\"line1\rline2\"");
}

#[test]
fn test_format_events_csv_multiple_events() {
    let now = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&Utc);
    let events = vec![
        TimelineEvent {
            id: "ev1".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            event_type: "msg".to_string(),
            timestamp: now,
            duration_ms: Some(10.0),
            status: EventStatus::Success,
            payload_summary: Some("p1".to_string()),
        },
        TimelineEvent {
            id: "ev2".to_string(),
            from: "b".to_string(),
            to: "c".to_string(),
            event_type: "invoke".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
            duration_ms: None,
            status: EventStatus::Failure,
            payload_summary: None,
        },
    ];
    let refs: Vec<&TimelineEvent> = events.iter().collect();
    let csv = format_events_csv(refs);
    assert!(csv.contains("ev1"));
    assert!(csv.contains("ev2"));
    assert!(csv.contains("msg"));
    assert!(csv.contains("invoke"));
    assert!(csv.contains("Success"));
    assert!(csv.contains("Failure"));
    assert!(csv.contains("p1"));
}

#[test]
fn test_apply_intents_toggle_details() {
    let mut view = TimelineView::new();
    assert!(view.show_details());
    view.apply_intents(&[TimelineIntent::ToggleDetails]);
    assert!(!view.show_details());
    view.apply_intents(&[TimelineIntent::ToggleDetails]);
    assert!(view.show_details());
}

#[test]
fn test_apply_intents_combined() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "t".to_string(),
        timestamp: Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[
        TimelineIntent::SelectEvent("e1".to_string()),
        TimelineIntent::ZoomIn,
        TimelineIntent::ZoomOut,
    ]);
    assert_eq!(view.selected_event(), Some("e1"));
}

#[test]
fn test_apply_intents_export_csv_no_panic() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "t".to_string(),
        timestamp: Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[TimelineIntent::ExportCsv]);
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn test_selected_event_accessor() {
    let mut view = TimelineView::new();
    assert!(view.selected_event().is_none());
    view.apply_intents(&[TimelineIntent::SelectEvent("evt-123".to_string())]);
    assert_eq!(view.selected_event(), Some("evt-123"));
}

#[test]
fn test_zoom_in_caps_at_10() {
    let z = zoom_in(10.0);
    assert!((z - 10.0).abs() < 0.01);
}

#[test]
fn test_zoom_out_floors_at_0_1() {
    let z = zoom_out(0.12);
    assert!(z >= 0.09 && z <= 0.11);
}

#[test]
fn test_timeline_intent_equality() {
    assert_eq!(TimelineIntent::Clear, TimelineIntent::Clear);
    assert_ne!(TimelineIntent::Clear, TimelineIntent::ZoomIn);
}

#[test]
fn test_filtered_event_count_empty() {
    let view = TimelineView::new();
    assert_eq!(view.filtered_event_count(), 0);
}

#[test]
fn test_filtered_event_count_single() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "t".to_string(),
        timestamp: Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn test_filter_all_events_excluded() {
    let mut view = TimelineView::new();
    let now = Utc::now();
    view.add_event(TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "discover".to_string(),
        timestamp: now,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.set_event_type_filter(Some("invoke".to_string()));
    assert_eq!(view.filtered_event_count(), 0);
}
