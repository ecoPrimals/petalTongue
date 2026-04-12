// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn test_interaction_subscriber_registry_new_and_default() {
    let reg = InteractionSubscriberRegistry::new();
    assert_eq!(reg.subscriber_count(), 0);

    let default = InteractionSubscriberRegistry::default();
    assert_eq!(default.subscriber_count(), 0);
}

#[test]
fn test_subscribe_returns_true_when_new() {
    let mut reg = InteractionSubscriberRegistry::new();
    assert!(reg.subscribe("sub1"));
    assert_eq!(reg.subscriber_count(), 1);
}

#[test]
fn test_subscribe_returns_false_when_already_registered() {
    let mut reg = InteractionSubscriberRegistry::new();
    assert!(reg.subscribe("sub1"));
    assert!(!reg.subscribe("sub1"));
    assert_eq!(reg.subscriber_count(), 1);
}

#[test]
fn test_subscribe_with_filter() {
    let mut reg = InteractionSubscriberRegistry::new();
    assert!(reg.subscribe_with_filter(
        "sub1",
        vec!["select".to_string(), "inspect".to_string()],
        Some("callback".to_string()),
        None,
        None,
    ));
    assert_eq!(reg.callback_method("sub1"), Some("callback"));
}

#[test]
fn test_subscribe_with_filter_duplicate_returns_false() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe_with_filter("sub1", vec!["select".to_string()], None, None, None);
    assert!(!reg.subscribe_with_filter("sub1", vec![], None, None, None));
}

#[test]
fn test_callback_method_none_for_subscriber_without_callback() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");
    assert_eq!(reg.callback_method("sub1"), None);
}

#[test]
fn test_callback_method_none_for_unknown_subscriber() {
    let reg = InteractionSubscriberRegistry::new();
    assert_eq!(reg.callback_method("unknown"), None);
}

#[test]
fn test_pending_callbacks_empty_when_no_events() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe_with_filter("sub1", vec![], Some("cb".to_string()), None, None);
    assert!(reg.pending_callbacks().is_empty());
}

#[test]
fn test_pending_callbacks_includes_subscriber_with_events() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe_with_filter("sub1", vec![], Some("cb".to_string()), None, None);
    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec!["t1".to_string()],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        perspective_id: None,
    };
    reg.broadcast(&event);
    let pending = reg.pending_callbacks();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].0, "sub1");
    assert_eq!(pending[0].1, "cb");
    assert_eq!(pending[0].2.len(), 1);
}

#[test]
fn test_broadcast_respects_event_filter() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe_with_filter("sub1", vec!["select".to_string()], None, None, None);

    let select_event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    reg.broadcast(&select_event);
    let events = reg.poll("sub1");
    assert_eq!(events.len(), 1);

    let inspect_event = InteractionEventNotification {
        event_type: "inspect".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    reg.broadcast(&inspect_event);
    let events2 = reg.poll("sub1");
    assert_eq!(events2.len(), 0, "inspect should be filtered out");
}

#[test]
fn test_broadcast_empty_filter_accepts_all() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");

    let event = InteractionEventNotification {
        event_type: "any_type".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    reg.broadcast(&event);
    let events = reg.poll("sub1");
    assert_eq!(events.len(), 1);
}

#[test]
fn test_poll_returns_events_and_clears_queue() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");
    let event = InteractionEventNotification {
        event_type: "e1".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    reg.broadcast(&event);
    let first = reg.poll("sub1");
    assert_eq!(first.len(), 1);
    let second = reg.poll("sub1");
    assert!(second.is_empty());
}

#[test]
fn test_poll_unknown_subscriber_returns_empty() {
    let mut reg = InteractionSubscriberRegistry::new();
    let events = reg.poll("unknown");
    assert!(events.is_empty());
}

#[test]
fn test_unsubscribe_returns_true_when_exists() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");
    assert!(reg.unsubscribe("sub1"));
    assert_eq!(reg.subscriber_count(), 0);
}

#[test]
fn test_unsubscribe_returns_false_when_not_exists() {
    let mut reg = InteractionSubscriberRegistry::new();
    assert!(!reg.unsubscribe("unknown"));
}

#[test]
fn test_apply_interaction() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");

    let req = InteractionApplyRequest {
        intent: "select".to_string(),
        targets: vec!["t1".to_string(), "t2".to_string()],
        grammar_id: None,
    };
    let (resp, callbacks) = reg.apply_interaction(&req);
    assert!(resp.accepted);
    assert_eq!(resp.targets_resolved, 2);
    assert_eq!(resp.pending_callbacks, 0);
    assert!(callbacks.is_empty());

    let events = reg.poll("sub1");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "select");
    assert_eq!(events[0].targets.len(), 2);
}

#[test]
fn test_perspectives() {
    let reg = InteractionSubscriberRegistry::new();
    let perspectives = reg.perspectives();
    assert_eq!(perspectives.len(), 1);
    assert_eq!(perspectives[0].id, "default_egui");
    assert_eq!(perspectives[0].modalities, vec!["gui"]);
    assert_eq!(perspectives[0].sync_mode, "shared_selection");
}

#[test]
fn test_interaction_event_notification_serialization() {
    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec!["a".to_string(), "b".to_string()],
        timestamp: "2025-03-10T12:00:00Z".to_string(),
        perspective_id: Some(42),
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let parsed: InteractionEventNotification = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.event_type, event.event_type);
    assert_eq!(parsed.targets, event.targets);
    assert_eq!(parsed.perspective_id, event.perspective_id);
}

#[test]
fn test_pending_callbacks_excludes_subscriber_without_callback() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("no_cb");
    reg.subscribe_with_filter("with_cb", vec![], Some("cb".to_string()), None, None);
    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    reg.broadcast(&event);
    let pending = reg.pending_callbacks();
    assert_eq!(
        pending.len(),
        1,
        "only subscriber with callback should appear"
    );
    assert_eq!(pending[0].0, "with_cb");
}

#[test]
fn test_broadcast_to_multiple_subscribers() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");
    reg.subscribe("sub2");
    let event = InteractionEventNotification {
        event_type: "navigate".to_string(),
        targets: vec!["panel".to_string()],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        perspective_id: Some(1),
    };
    reg.broadcast(&event);
    let e1 = reg.poll("sub1");
    let e2 = reg.poll("sub2");
    assert_eq!(e1.len(), 1, "sub1 should receive event");
    assert_eq!(e2.len(), 1, "sub2 should receive event");
    assert_eq!(e1[0].event_type, e2[0].event_type);
    assert_eq!(e1[0].targets, e2[0].targets);
}

#[test]
fn test_apply_interaction_empty_targets() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("sub1");
    let req = InteractionApplyRequest {
        intent: "focus".to_string(),
        targets: vec![],
        grammar_id: Some("g1".to_string()),
    };
    let (resp, _callbacks) = reg.apply_interaction(&req);
    assert!(resp.accepted);
    assert_eq!(resp.targets_resolved, 0);
    let events = reg.poll("sub1");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "focus");
}

#[test]
fn test_interaction_event_notification_clone() {
    let event = InteractionEventNotification {
        event_type: "inspect".to_string(),
        targets: vec!["id1".to_string()],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        perspective_id: None,
    };
    let cloned = event.clone();
    assert_eq!(event.event_type, cloned.event_type);
    assert_eq!(event.targets, cloned.targets);
}

#[test]
fn test_interaction_event_notification_debug() {
    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    let debug_str = format!("{event:?}");
    assert!(debug_str.contains("InteractionEventNotification"));
    assert!(debug_str.contains("select"));
}

#[test]
fn test_interaction_event_notification_serialization_perspective_none() {
    let event = InteractionEventNotification {
        event_type: "navigate".to_string(),
        targets: vec![],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        perspective_id: None,
    };
    let json = serde_json::to_string(&event).expect("serialize");
    assert!(!json.contains("perspective_id"));
    let parsed: InteractionEventNotification = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.perspective_id, None);
}

#[test]
fn test_apply_interaction_no_subscribers() {
    let mut reg = InteractionSubscriberRegistry::new();
    let req = InteractionApplyRequest {
        intent: "select".to_string(),
        targets: vec!["t1".to_string()],
        grammar_id: None,
    };
    let (resp, callbacks) = reg.apply_interaction(&req);
    assert!(resp.accepted);
    assert_eq!(resp.targets_resolved, 1);
    assert!(callbacks.is_empty());
    assert_eq!(reg.subscriber_count(), 0);
}

#[test]
fn test_apply_interaction_with_callback_subscriber() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("poll-only");
    reg.subscribe_with_filter(
        "push-sub",
        vec![],
        Some("spring.on_interaction".to_string()),
        None,
        Some("/tmp/push-sub.sock".to_string()),
    );

    let req = InteractionApplyRequest {
        intent: "select".to_string(),
        targets: vec!["node-1".to_string()],
        grammar_id: None,
    };
    let (resp, callbacks) = reg.apply_interaction(&req);
    assert!(resp.accepted);
    assert_eq!(resp.targets_resolved, 1);
    assert_eq!(resp.pending_callbacks, 1);
    assert_eq!(callbacks.len(), 1);
    assert_eq!(callbacks[0].subscriber_id, "push-sub");
    assert_eq!(callbacks[0].method, "spring.on_interaction");
    assert_eq!(
        callbacks[0].callback_socket.as_deref(),
        Some("/tmp/push-sub.sock")
    );
}

#[test]
fn test_pending_callbacks_multiple_mixed() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe_with_filter(
        "with_cb_events",
        vec![],
        Some("cb2".to_string()),
        None,
        None,
    );
    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };
    reg.broadcast(&event);
    reg.subscribe_with_filter("with_cb_empty", vec![], Some("cb1".to_string()), None, None);
    let pending = reg.pending_callbacks();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].0, "with_cb_events");
    assert_eq!(pending[0].2.len(), 1);
}

#[test]
fn test_subscribe_with_filter_grammar_id_ignored() {
    let mut reg = InteractionSubscriberRegistry::new();
    assert!(reg.subscribe_with_filter(
        "sub1",
        vec!["select".to_string()],
        Some("cb".to_string()),
        Some("grammar-1".to_string()),
        None,
    ));
    assert_eq!(reg.callback_method("sub1"), Some("cb"));
}

#[test]
fn test_broadcast_returns_callback_dispatches() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe_with_filter("poll-sub", vec![], None, None, None);
    reg.subscribe_with_filter(
        "cb-sub",
        vec![],
        Some("spring.on_interaction".to_string()),
        None,
        None,
    );

    let event = InteractionEventNotification {
        event_type: "select".to_string(),
        targets: vec!["node-1".to_string()],
        timestamp: "2026-04-01T00:00:00Z".to_string(),
        perspective_id: None,
    };

    let callbacks = reg.broadcast(&event);

    assert_eq!(
        callbacks.len(),
        1,
        "only the callback subscriber should produce a dispatch"
    );
    assert_eq!(callbacks[0].subscriber_id, "cb-sub");
    assert_eq!(callbacks[0].method, "spring.on_interaction");
    assert_eq!(callbacks[0].events.len(), 1);
    assert_eq!(callbacks[0].events[0].event_type, "select");

    // poll-sub should still have the event queued for poll
    let polled = reg.poll("poll-sub");
    assert_eq!(polled.len(), 1);
}

#[test]
fn test_broadcast_no_callbacks_when_none_registered() {
    let mut reg = InteractionSubscriberRegistry::new();
    reg.subscribe("poll-only");

    let event = InteractionEventNotification {
        event_type: "hover".to_string(),
        targets: vec![],
        timestamp: String::new(),
        perspective_id: None,
    };

    let callbacks = reg.broadcast(&event);
    assert!(callbacks.is_empty());
}
