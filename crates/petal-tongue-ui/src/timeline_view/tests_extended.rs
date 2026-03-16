// SPDX-License-Identifier: AGPL-3.0-or-later
//! Timeline View - Extended / property / edge-case tests

use super::{
    EventStatus, TimelineEvent, TimelineIntent, TimelineView, build_primal_lanes,
    compute_lane_height, escape_csv, event_screen_rect, format_events_csv, prepare_event_detail,
    time_to_x, zoom_in, zoom_out,
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

#[test]
fn event_status_color_rgba_success() {
    assert_eq!(EventStatus::Success.color_rgba(), [76, 175, 80, 255]);
}

#[test]
fn event_status_color_rgba_failure() {
    assert_eq!(EventStatus::Failure.color_rgba(), [244, 67, 54, 255]);
}

#[test]
fn event_status_color_rgba_in_progress() {
    assert_eq!(EventStatus::InProgress.color_rgba(), [255, 152, 0, 255]);
}

#[test]
fn event_status_color_rgba_timeout() {
    assert_eq!(EventStatus::Timeout.color_rgba(), [156, 39, 176, 255]);
}

#[test]
fn build_primal_lanes_empty() {
    let events: Vec<TimelineEvent> = vec![];
    let lanes = build_primal_lanes(&events);
    assert!(lanes.is_empty());
}

#[test]
fn build_primal_lanes_single_primal() {
    let base = chrono::Utc::now();
    let events = vec![TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "a".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    }];
    let lanes = build_primal_lanes(&events);
    assert_eq!(lanes.len(), 1);
    assert_eq!(lanes.get("a"), Some(&0));
}

#[test]
fn build_primal_lanes_multiple_primals() {
    let base = chrono::Utc::now();
    let events = vec![
        TimelineEvent {
            id: "e1".to_string(),
            from: "charlie".to_string(),
            to: "alice".to_string(),
            event_type: "x".to_string(),
            timestamp: base,
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        },
        TimelineEvent {
            id: "e2".to_string(),
            from: "bob".to_string(),
            to: "charlie".to_string(),
            event_type: "x".to_string(),
            timestamp: base + chrono::Duration::seconds(1),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        },
    ];
    let lanes = build_primal_lanes(&events);
    assert_eq!(lanes.len(), 3);
    assert_eq!(lanes.get("alice"), Some(&0));
    assert_eq!(lanes.get("bob"), Some(&1));
    assert_eq!(lanes.get("charlie"), Some(&2));
}

#[test]
fn compute_lane_height_zero_lanes() {
    let h = compute_lane_height(100.0, 0);
    assert!((h - 100.0).abs() < f32::EPSILON);
}

#[test]
fn compute_lane_height_one_lane() {
    let h = compute_lane_height(100.0, 1);
    assert!((h - 50.0).abs() < f32::EPSILON);
}

#[test]
fn compute_lane_height_ten_lanes() {
    let h = compute_lane_height(110.0, 10);
    assert!((h - 10.0).abs() < f32::EPSILON);
}

#[test]
fn event_screen_rect_at_start() {
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    let event = TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    };
    let mut primal_lanes = std::collections::HashMap::new();
    primal_lanes.insert("a".to_string(), 0);
    primal_lanes.insert("b".to_string(), 1);
    let start_ms = base.timestamp_millis() as f64;
    let end_ms = (base + chrono::Duration::seconds(1)).timestamp_millis() as f64;
    let rect = event_screen_rect(
        &event,
        start_ms,
        end_ms,
        (0.0, 0.0),
        100.0,
        20.0,
        &primal_lanes,
    );
    let (x, y, w, h) = rect.expect("rect");
    assert!((x - 96.0).abs() < 1.0);
    assert!((y - 16.0).abs() < 1.0);
    assert_eq!(w, 8.0);
    assert_eq!(h, 28.0);
}

#[test]
fn event_screen_rect_at_end() {
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    let event = TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    };
    let mut primal_lanes = std::collections::HashMap::new();
    primal_lanes.insert("a".to_string(), 0);
    primal_lanes.insert("b".to_string(), 1);
    let start_ms = base.timestamp_millis() as f64;
    let end_ms = (base + chrono::Duration::seconds(1)).timestamp_millis() as f64;
    let rect = event_screen_rect(
        &event,
        start_ms,
        end_ms,
        (0.0, 0.0),
        100.0,
        20.0,
        &primal_lanes,
    );
    let (x, _y, w, _h) = rect.expect("rect");
    assert!((x - 196.0).abs() < 1.0);
    assert_eq!(w, 8.0);
}

#[test]
fn event_screen_rect_unknown_primal_none() {
    let base = chrono::Utc::now();
    let event = TimelineEvent {
        id: "e1".to_string(),
        from: "unknown".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    };
    let mut primal_lanes = std::collections::HashMap::new();
    primal_lanes.insert("a".to_string(), 0);
    primal_lanes.insert("b".to_string(), 1);
    let rect = event_screen_rect(&event, 0.0, 1000.0, (0.0, 0.0), 100.0, 20.0, &primal_lanes);
    assert!(rect.is_none());
}

#[test]
fn filtered_events_no_filters() {
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
    let filtered = view.filtered_events_for_test();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn filtered_events_type_and_primal_combined() {
    let mut view = TimelineView::new();
    let base = chrono::Utc::now();
    view.add_event(TimelineEvent {
        id: "1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "2".to_string(),
        from: "bob".to_string(),
        to: "charlie".to_string(),
        event_type: "invoke".to_string(),
        timestamp: base + chrono::Duration::seconds(1),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.set_event_type_filter(Some("discover".to_string()));
    view.set_primal_filter(Some("alice".to_string()));
    let filtered = view.filtered_events_for_test();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "1");
}

#[test]
fn filtered_events_time_range_excludes_outside() {
    let mut view = TimelineView::new();
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    view.add_event(TimelineEvent {
        id: "before".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base - chrono::Duration::seconds(60),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.add_event(TimelineEvent {
        id: "in".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.set_time_range(
        Some(base - chrono::Duration::seconds(10)),
        Some(base + chrono::Duration::seconds(10)),
    );
    let filtered = view.filtered_events_for_test();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "in");
}

#[test]
fn apply_intents_zoom_in() {
    let mut view = TimelineView::new();
    let z0 = view.zoom();
    view.apply_intents(&[TimelineIntent::ZoomIn]);
    assert!(view.zoom() > z0);
}

#[test]
fn apply_intents_zoom_out() {
    let mut view = TimelineView::new();
    let z0 = view.zoom();
    view.apply_intents(&[TimelineIntent::ZoomOut]);
    assert!(view.zoom() < z0);
}

#[test]
fn apply_intents_toggle_details() {
    let mut view = TimelineView::new();
    let d0 = view.show_details();
    view.apply_intents(&[TimelineIntent::ToggleDetails]);
    assert_ne!(view.show_details(), d0);
}

#[test]
fn apply_intents_select_event() {
    let mut view = TimelineView::new();
    view.apply_intents(&[TimelineIntent::SelectEvent("evt-1".to_string())]);
    assert_eq!(view.selected_event(), Some("evt-1"));
}

#[test]
fn apply_intents_deselect_event() {
    let mut view = TimelineView::new();
    view.apply_intents(&[TimelineIntent::SelectEvent("x".to_string())]);
    view.apply_intents(&[TimelineIntent::DeselectEvent]);
    assert!(view.selected_event().is_none());
}

#[test]
fn apply_intents_clear_via_intent() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[TimelineIntent::Clear]);
    assert_eq!(view.filtered_event_count(), 0);
}

#[test]
fn export_csv_path_returns_path() {
    let view = TimelineView::new();
    let path = view.export_csv_path_for_test();
    assert!(path.to_string_lossy().contains("timeline"));
    assert!(path.to_string_lossy().contains("csv"));
}

#[test]
fn write_events_csv_mock_data() {
    let view = TimelineView::new();
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    let events = vec![
        TimelineEvent {
            id: "ev1".to_string(),
            from: "alice".to_string(),
            to: "bob".to_string(),
            event_type: "discover".to_string(),
            timestamp: base,
            duration_ms: Some(42.5),
            status: EventStatus::Success,
            payload_summary: Some("summary".to_string()),
        },
        TimelineEvent {
            id: "ev2".to_string(),
            from: "bob".to_string(),
            to: "charlie".to_string(),
            event_type: "invoke".to_string(),
            timestamp: base + chrono::Duration::seconds(1),
            duration_ms: None,
            status: EventStatus::Failure,
            payload_summary: None,
        },
    ];
    let refs: Vec<&TimelineEvent> = events.iter().collect();
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("test_events.csv");
    view.write_events_csv_for_test(&path, refs).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    assert!(content.contains("ev1"));
    assert!(content.contains("ev2"));
    assert!(content.contains("alice"));
    assert!(content.contains("discover"));
    assert!(content.contains("Success"));
    assert!(content.contains("Failure"));
}
