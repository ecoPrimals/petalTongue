// SPDX-License-Identifier: AGPL-3.0-or-later
//! Timeline View - Rendering and extended behavior tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{EventStatus, TimelineEvent, TimelineIntent, TimelineView};

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
    let events = [
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

#[test]
fn timeline_view_render_with_show_details_and_selected_event() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: Some(42.5),
        status: EventStatus::Success,
        payload_summary: Some("test payload".to_string()),
    });
    view.apply_intents(&[TimelineIntent::SelectEvent("evt1".to_string())]);
    assert!(view.show_details());
    assert_eq!(view.selected_event(), Some("evt1"));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
}

#[test]
fn timeline_view_render_with_show_details_false() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[TimelineIntent::ToggleDetails]);
    assert!(!view.show_details());

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            assert!(
                intents.is_empty()
                    || !intents
                        .iter()
                        .any(|i| matches!(i, TimelineIntent::SelectEvent(_)))
            );
        });
    });
}

#[test]
fn timeline_view_render_filtered_events_empty_returns_early() {
    let mut view = TimelineView::new();
    let base = chrono::Utc::now();
    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.set_event_type_filter(Some("invoke".to_string()));
    assert_eq!(view.filtered_event_count(), 0);
    assert!(!view.get_primals_for_test().is_empty());

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
}

#[test]
fn timeline_view_render_details_panel_event_not_found() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "evt1".to_string(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        event_type: "discover".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });
    view.apply_intents(&[TimelineIntent::SelectEvent("nonexistent-id".to_string())]);
    assert!(view.show_details());
    assert_eq!(view.selected_event(), Some("nonexistent-id"));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
}

#[test]
fn timeline_view_render_details_panel_with_duration_and_payload() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "evt-detail".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "invoke".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: Some(123.45),
        status: EventStatus::InProgress,
        payload_summary: Some("payload content".to_string()),
    });
    view.apply_intents(&[TimelineIntent::SelectEvent("evt-detail".to_string())]);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
}

#[test]
fn timeline_view_render_with_events_toolbar_renders() {
    let mut view = TimelineView::new();
    view.add_event(TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    });

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
    assert_eq!(view.filtered_event_count(), 1);
}

#[test]
fn timeline_view_zoom_accessor() {
    let view = TimelineView::new();
    assert!((view.zoom() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn write_events_csv_creates_parent_dir() {
    let view = TimelineView::new();
    let base = chrono::Utc::now();
    let events = [TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "b".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    }];
    let refs: Vec<&TimelineEvent> = events.iter().collect();
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("subdir").join("nested").join("events.csv");
    view.write_events_csv_for_test(&path, refs).expect("write");
    assert!(path.exists());
}
