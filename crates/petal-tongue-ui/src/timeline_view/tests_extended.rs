// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Extended / property / edge-case tests

use super::{
    EventStatus, TimelineEvent, TimelineIntent, TimelineView, escape_csv, format_events_csv,
    prepare_event_detail, time_to_x, zoom_in, zoom_out,
};

// === Pure function tests (from view.rs) ===

#[test]
fn prepare_event_detail_complete() {
    let event = TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339("2025-01-15T12:30:45.123Z")
            .expect("valid datetime")
            .with_timezone(&chrono::Utc),
        duration_ms: Some(42.5),
        status: EventStatus::Success,
        payload_summary: Some("payload data".to_string()),
    };
    let d = prepare_event_detail(&event);
    assert_eq!(d.status_icon, "✅");
    assert_eq!(d.status_label, "Success");
    assert_eq!(d.from, "alice");
    assert_eq!(d.to, "bob");
    assert_eq!(d.event_type, "discover");
    assert_eq!(d.time_str, "12:30:45.123");
    assert_eq!(d.duration_str.as_deref(), Some("42.50ms"));
    assert_eq!(d.payload.as_deref(), Some("payload data"));
}

#[test]
fn prepare_event_detail_no_optional_fields() {
    let event = TimelineEvent {
        id: "evt2".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "invoke".to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339("2025-06-01T00:00:00Z")
            .expect("valid datetime")
            .with_timezone(&chrono::Utc),
        duration_ms: None,
        status: EventStatus::Failure,
        payload_summary: None,
    };
    let d = prepare_event_detail(&event);
    assert_eq!(d.status_icon, "❌");
    assert_eq!(d.status_label, "Failure");
    assert!(d.duration_str.is_none());
    assert!(d.payload.is_none());
}

#[test]
fn prepare_event_detail_timeout() {
    let event = TimelineEvent {
        id: "evt3".to_string(),
        from: "x".to_string(),
        to: "y".to_string(),
        event_type: "ping".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: Some(5000.0),
        status: EventStatus::Timeout,
        payload_summary: None,
    };
    let d = prepare_event_detail(&event);
    assert_eq!(d.status_icon, "⏱️");
    assert_eq!(d.status_label, "Timeout");
    assert_eq!(d.duration_str.as_deref(), Some("5000.00ms"));
}

#[test]
fn prepare_event_detail_in_progress() {
    let event = TimelineEvent {
        id: "evt4".to_string(),
        from: "x".to_string(),
        to: "y".to_string(),
        event_type: "stream".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::InProgress,
        payload_summary: Some("streaming...".to_string()),
    };
    let d = prepare_event_detail(&event);
    assert_eq!(d.status_icon, "⏳");
    assert_eq!(d.payload.as_deref(), Some("streaming..."));
}

#[test]
fn zoom_in_increases() {
    assert!(zoom_in(1.0) > 1.0);
}

#[test]
fn zoom_in_caps_at_10() {
    assert!((zoom_in(10.0) - 10.0).abs() < f32::EPSILON);
}

#[test]
fn zoom_out_decreases() {
    assert!(zoom_out(1.0) < 1.0);
}

#[test]
fn zoom_out_floors_at_0_1() {
    let result = zoom_out(0.1);
    assert!((result - 0.1).abs() < 0.01);
}

#[test]
fn apply_intents_zoom() {
    let mut view = TimelineView::new();
    let initial = view.zoom();
    view.apply_intents(&[TimelineIntent::ZoomIn]);
    assert!(view.zoom() > initial);
    view.apply_intents(&[TimelineIntent::ZoomOut, TimelineIntent::ZoomOut]);
    assert!(view.zoom() < initial);
}

#[test]
fn apply_intents_select_deselect() {
    let mut view = TimelineView::new();
    view.apply_intents(&[TimelineIntent::SelectEvent("evt1".to_string())]);
    assert_eq!(view.selected_event(), Some("evt1"));
    view.apply_intents(&[TimelineIntent::DeselectEvent]);
    assert!(view.selected_event().is_none());
}

#[test]
fn apply_intents_clear() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "t".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[TimelineIntent::Clear]);
    assert_eq!(view.filtered_event_count(), 0);
}

#[test]
fn time_to_x_start() {
    let x = time_to_x(100.0, 100.0, 200.0, 500.0);
    assert!(x.abs() < f32::EPSILON);
}

#[test]
fn time_to_x_end() {
    let x = time_to_x(200.0, 100.0, 200.0, 500.0);
    assert!((x - 500.0).abs() < f32::EPSILON);
}

#[test]
fn time_to_x_mid() {
    let x = time_to_x(150.0, 100.0, 200.0, 500.0);
    assert!((x - 250.0).abs() < f32::EPSILON);
}

#[test]
fn time_to_x_zero_range() {
    let x = time_to_x(100.0, 100.0, 100.0, 500.0);
    assert!(x.abs() < f32::EPSILON);
}

#[test]
fn time_to_x_clamps_before_start() {
    let x = time_to_x(50.0, 100.0, 200.0, 500.0);
    assert!(x.abs() < f32::EPSILON);
}

#[test]
fn time_to_x_clamps_after_end() {
    let x = time_to_x(250.0, 100.0, 200.0, 500.0);
    assert!((x - 500.0).abs() < f32::EPSILON);
}

#[test]
fn escape_csv_plain() {
    assert_eq!(escape_csv("hello"), "hello");
}

#[test]
fn escape_csv_with_comma() {
    assert_eq!(escape_csv("a,b"), "\"a,b\"");
}

#[test]
fn escape_csv_with_quote() {
    assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
}

#[test]
fn escape_csv_with_newline() {
    assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
}

#[test]
fn format_events_csv_empty() {
    let events: Vec<&TimelineEvent> = Vec::new();
    let csv = format_events_csv(events);
    assert_eq!(
        csv,
        "id,from,to,event_type,timestamp,duration_ms,status,payload_summary\n"
    );
}

#[test]
fn format_events_csv_single_event() {
    let event = TimelineEvent {
        id: "ev1".to_string(),
        from: "primal-a".to_string(),
        to: "primal-b".to_string(),
        event_type: "message".to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
            .expect("valid datetime")
            .with_timezone(&chrono::Utc),
        duration_ms: Some(42.5),
        status: EventStatus::Success,
        payload_summary: Some("summary".to_string()),
    };
    let csv = format_events_csv(vec![&event]);
    assert!(
        csv.starts_with("id,from,to,event_type,timestamp,duration_ms,status,payload_summary\n")
    );
    assert!(csv.contains("ev1"));
    assert!(csv.contains("primal-a"));
    assert!(csv.contains("primal-b"));
    assert!(csv.contains("message"));
    assert!(csv.contains("42.5"));
    assert!(csv.contains("Success"));
    assert!(csv.contains("summary"));
}

#[test]
fn format_events_csv_escaped_payload() {
    let event = TimelineEvent {
        id: "ev1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "msg".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: Some("line1,line2\n\"quoted\"".to_string()),
    };
    let csv = format_events_csv(vec![&event]);
    assert!(csv.contains("\"line1,line2"));
    assert!(csv.contains("\"\"quoted\"\""));
}

#[test]
fn escape_csv_with_carriage_return() {
    assert_eq!(escape_csv("a\rb"), "\"a\rb\"");
}

#[test]
fn apply_intents_export_csv_no_panic() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "test".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[TimelineIntent::ExportCsv]);
}

#[test]
fn filtered_event_count_with_type_filter() {
    let mut view = TimelineView::new();
    let base = chrono::Utc::now();
    view.add_event(TimelineEvent {
        id: "1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "discover".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "2".to_string(),
        from: "b".to_string(),
        to: "c".to_string(),
        event_type: "invoke".to_string(),
        timestamp: base + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    assert_eq!(view.filtered_event_count(), 2);
    view.set_event_type_filter(Some("discover".to_string()));
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn filtered_event_count_with_primal_filter() {
    let mut view = TimelineView::new();
    let base = chrono::Utc::now();
    view.add_event(TimelineEvent {
        id: "1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "2".to_string(),
        from: "bob".to_string(),
        to: "charlie".to_string(),
        event_type: "x".to_string(),
        timestamp: base + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.set_primal_filter(Some("alice".to_string()));
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn filtered_event_count_with_time_range() {
    let mut view = TimelineView::new();
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    view.add_event(TimelineEvent {
        id: "1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base - chrono::Duration::seconds(30),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "2".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "3".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base + chrono::Duration::seconds(30),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.set_time_range(
        Some(base - chrono::Duration::seconds(10)),
        Some(base + chrono::Duration::seconds(10)),
    );
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn get_primals_for_test_returns_sorted_unique() {
    let mut view = TimelineView::new();
    let base = chrono::Utc::now();
    view.add_event(TimelineEvent {
        id: "1".to_string(),
        from: "charlie".to_string(),
        to: "alice".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "2".to_string(),
        from: "bob".to_string(),
        to: "charlie".to_string(),
        event_type: "x".to_string(),
        timestamp: base + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    let primals = view.get_primals_for_test();
    assert_eq!(primals, vec!["alice", "bob", "charlie"]);
}

#[test]
fn event_ids_ordered_by_timestamp() {
    let mut view = TimelineView::new();
    let base = chrono::Utc::now();
    view.add_event(TimelineEvent {
        id: "second".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "first".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    let ids = view.event_ids_ordered();
    assert_eq!(ids, vec!["first", "second"]);
}

#[test]
fn timeline_view_render_headless() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: Some(42.5),
        status: EventStatus::Success,
        payload_summary: Some("test".to_string()),
    });

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
}

#[test]
fn timeline_view_render_empty_headless() {
    let mut view = TimelineView::new();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            assert!(intents.is_empty());
        });
    });
}
