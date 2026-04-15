// SPDX-License-Identifier: AGPL-3.0-or-later
//! Timeline View - Lane layout, rendering, and screen-rect extended tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{
    EventStatus, TimelineEvent, TimelineView, build_primal_lanes, compute_lane_height,
    event_screen_rect,
};

// === View state, lanes, rendering, and screen layout ===

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
fn event_screen_rect_same_lane() {
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    let event = TimelineEvent {
        id: "e1".to_string(),
        from: "a".to_string(),
        to: "a".to_string(),
        event_type: "x".to_string(),
        timestamp: base,
        duration_ms: None,
        status: EventStatus::Success,
        payload_summary: None,
    };
    let mut primal_lanes = std::collections::HashMap::new();
    primal_lanes.insert("a".to_string(), 0);
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
    assert_eq!(w, 8.0);
    assert_eq!(h, 8.0);
    assert!((x - 96.0).abs() < 1.0);
    assert!((0.0..30.0).contains(&y));
}

#[test]
fn event_screen_rect_reversed_lanes() {
    let base = chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
        .expect("valid")
        .with_timezone(&chrono::Utc);
    let event = TimelineEvent {
        id: "e1".to_string(),
        from: "b".to_string(),
        to: "a".to_string(),
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
    let (_x, _y, w, h) = rect.expect("rect");
    assert_eq!(w, 8.0);
    assert_eq!(h, 28.0);
}
