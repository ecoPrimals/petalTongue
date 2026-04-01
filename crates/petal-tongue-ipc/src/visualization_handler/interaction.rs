// SPDX-License-Identifier: AGPL-3.0-or-later
//! Interaction event subsystem: poll-based subscriptions for springs.
//!
//! Springs call `interaction.subscribe` to register, then `interaction.poll`
//! to drain queued interaction events. Supports callback-based delivery
//! (healthSpring V12 pattern) when `callback_method` is set.

use petal_tongue_core::{SensorEventBatch, SensorEventIpc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{InteractionApplyRequest, InteractionApplyResponse, Perspective};

/// A pending callback dispatch for a subscriber with `callback_method` set.
///
/// The IPC server sends these as JSON-RPC notifications to the subscriber's
/// socket, completing the healthSpring V12 callback pattern (push instead of poll).
#[derive(Debug, Clone)]
pub struct CallbackDispatch {
    /// Subscriber that requested callback delivery.
    pub subscriber_id: String,
    /// JSON-RPC method to invoke on the subscriber's socket.
    pub method: String,
    /// Events to deliver in this callback batch.
    pub events: Vec<InteractionEventNotification>,
}

/// Poll-based registry for interaction event subscribers.
///
/// Springs call `interaction.subscribe` to register, then `interaction.poll`
/// to drain queued interaction events.
#[derive(Default)]
pub struct InteractionSubscriberRegistry {
    subscribers: HashMap<String, InteractionSubscriber>,
}

struct InteractionSubscriber {
    queue: Vec<InteractionEventNotification>,
    event_filter: Vec<String>,
    callback_method: Option<String>,
}

/// A queued interaction event ready for IPC delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEventNotification {
    /// Semantic event type (e.g. "select", "inspect", "navigate").
    pub event_type: String,
    /// Resolved data-space target identifiers.
    pub targets: Vec<String>,
    /// ISO 8601 timestamp of the event.
    pub timestamp: String,
    /// Perspective that originated the event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perspective_id: Option<u64>,
}

impl InteractionSubscriberRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a subscriber. Returns `true` if newly registered.
    pub fn subscribe(&mut self, subscriber_id: &str) -> bool {
        self.subscribe_with_filter(subscriber_id, Vec::new(), None, None)
    }

    /// Register a subscriber with event type filter, callback method, and grammar filter.
    ///
    /// The callback model (healthSpring V12 pattern): when `callback_method` is set,
    /// events are queued for callback delivery rather than poll-only.
    pub fn subscribe_with_filter(
        &mut self,
        subscriber_id: &str,
        event_filter: Vec<String>,
        callback_method: Option<String>,
        _grammar_id: Option<String>,
    ) -> bool {
        use std::collections::hash_map::Entry;
        match self.subscribers.entry(subscriber_id.to_string()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(e) => {
                e.insert(InteractionSubscriber {
                    queue: Vec::new(),
                    event_filter,
                    callback_method,
                });
                true
            }
        }
    }

    /// Get the callback method for a subscriber (if callback-based).
    #[must_use]
    pub fn callback_method(&self, subscriber_id: &str) -> Option<&str> {
        self.subscribers
            .get(subscriber_id)
            .and_then(|s| s.callback_method.as_deref())
    }

    /// Get all subscribers that have callback methods and pending events.
    #[must_use]
    pub fn pending_callbacks(&self) -> Vec<(&str, &str, &[InteractionEventNotification])> {
        self.subscribers
            .iter()
            .filter_map(|(id, sub)| {
                sub.callback_method
                    .as_deref()
                    .filter(|_| !sub.queue.is_empty())
                    .map(|method| (id.as_str(), method, sub.queue.as_slice()))
            })
            .collect()
    }

    /// Remove a subscriber. Returns `true` if the subscriber existed.
    pub fn unsubscribe(&mut self, subscriber_id: &str) -> bool {
        self.subscribers.remove(subscriber_id).is_some()
    }

    /// Push an event to all active subscribers, respecting event type filters.
    ///
    /// Returns a list of `(subscriber_id, callback_method, events)` for
    /// subscribers that have a `callback_method` set, so the caller can
    /// dispatch JSON-RPC notifications proactively instead of waiting for poll.
    pub fn broadcast(&mut self, event: &InteractionEventNotification) -> Vec<CallbackDispatch> {
        let mut callbacks = Vec::new();
        for (id, sub) in &mut self.subscribers {
            let passes_event_filter =
                sub.event_filter.is_empty() || sub.event_filter.contains(&event.event_type);
            if passes_event_filter {
                sub.queue.push(event.clone());

                if let Some(method) = &sub.callback_method {
                    callbacks.push(CallbackDispatch {
                        subscriber_id: id.clone(),
                        method: method.clone(),
                        events: vec![event.clone()],
                    });
                }
            }
        }
        callbacks
    }

    /// Drain queued events for a subscriber, returning them.
    pub fn poll(&mut self, subscriber_id: &str) -> Vec<InteractionEventNotification> {
        self.subscribers
            .get_mut(subscriber_id)
            .map(|sub| std::mem::take(&mut sub.queue))
            .unwrap_or_default()
    }

    /// Number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Apply an interaction intent and broadcast to subscribers.
    pub fn apply_interaction(&mut self, req: &InteractionApplyRequest) -> InteractionApplyResponse {
        let event = InteractionEventNotification {
            event_type: req.intent.clone(),
            targets: req.targets.clone(),
            timestamp: {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                format!("{}Z", now.as_secs())
            },
            perspective_id: None,
        };
        self.broadcast(&event);
        InteractionApplyResponse {
            accepted: true,
            targets_resolved: req.targets.len(),
        }
    }

    /// Return available perspectives for this visualization.
    #[must_use]
    pub fn perspectives(&self) -> Vec<Perspective> {
        vec![Perspective {
            id: "default_egui".to_string(),
            modalities: vec!["gui".to_string()],
            selection: Vec::new(),
            sync_mode: "shared_selection".to_string(),
        }]
    }
}

/// Registry for sensor event stream subscribers.
///
/// External primals subscribe to raw sensor events (pointer, keys, scroll)
/// for engagement analysis (ludoSpring) and AI adaptation (Squirrel).
#[derive(Default)]
pub struct SensorStreamRegistry {
    subscribers: HashMap<String, SensorStreamSubscriber>,
    next_id: u64,
}

struct SensorStreamSubscriber {
    queue: Vec<SensorEventIpc>,
}

impl SensorStreamRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new subscriber. Returns the assigned subscription ID.
    pub fn subscribe(&mut self) -> String {
        self.next_id += 1;
        let id = format!("sensor-sub-{}", self.next_id);
        self.subscribers
            .insert(id.clone(), SensorStreamSubscriber { queue: Vec::new() });
        id
    }

    /// Remove a subscriber. Returns `true` if the subscriber existed.
    pub fn unsubscribe(&mut self, subscription_id: &str) -> bool {
        self.subscribers.remove(subscription_id).is_some()
    }

    /// Push sensor events to all active subscribers.
    pub fn broadcast(&mut self, events: &[SensorEventIpc]) {
        if events.is_empty() {
            return;
        }
        for sub in self.subscribers.values_mut() {
            sub.queue.extend_from_slice(events);
        }
    }

    /// Drain queued events for a subscriber as a batch.
    pub fn poll(&mut self, subscription_id: &str) -> SensorEventBatch {
        let events = self
            .subscribers
            .get_mut(subscription_id)
            .map(|sub| std::mem::take(&mut sub.queue))
            .unwrap_or_default();
        SensorEventBatch {
            subscription_id: subscription_id.to_string(),
            events,
        }
    }

    /// Number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

#[cfg(test)]
mod tests {
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
        ));
        assert_eq!(reg.callback_method("sub1"), Some("callback"));
    }

    #[test]
    fn test_subscribe_with_filter_duplicate_returns_false() {
        let mut reg = InteractionSubscriberRegistry::new();
        reg.subscribe_with_filter("sub1", vec!["select".to_string()], None, None);
        assert!(!reg.subscribe_with_filter("sub1", vec![], None, None));
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
        reg.subscribe_with_filter("sub1", vec![], Some("cb".to_string()), None);
        assert!(reg.pending_callbacks().is_empty());
    }

    #[test]
    fn test_pending_callbacks_includes_subscriber_with_events() {
        let mut reg = InteractionSubscriberRegistry::new();
        reg.subscribe_with_filter("sub1", vec![], Some("cb".to_string()), None);
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
        reg.subscribe_with_filter("sub1", vec!["select".to_string()], None, None);

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
        let resp = reg.apply_interaction(&req);
        assert!(resp.accepted);
        assert_eq!(resp.targets_resolved, 2);

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
        let parsed: InteractionEventNotification =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.event_type, event.event_type);
        assert_eq!(parsed.targets, event.targets);
        assert_eq!(parsed.perspective_id, event.perspective_id);
    }

    #[test]
    fn test_pending_callbacks_excludes_subscriber_without_callback() {
        let mut reg = InteractionSubscriberRegistry::new();
        reg.subscribe("no_cb");
        reg.subscribe_with_filter("with_cb", vec![], Some("cb".to_string()), None);
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
        let resp = reg.apply_interaction(&req);
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

    // === SensorStreamRegistry tests ===

    #[test]
    fn test_sensor_stream_registry_new() {
        let reg = SensorStreamRegistry::new();
        assert_eq!(reg.subscriber_count(), 0);
    }

    #[test]
    fn test_sensor_stream_subscribe() {
        let mut reg = SensorStreamRegistry::new();
        let id = reg.subscribe();
        assert!(id.starts_with("sensor-sub-"));
        assert_eq!(reg.subscriber_count(), 1);
    }

    #[test]
    fn test_sensor_stream_subscribe_unique_ids() {
        let mut reg = SensorStreamRegistry::new();
        let id1 = reg.subscribe();
        let id2 = reg.subscribe();
        assert_ne!(id1, id2);
        assert_eq!(reg.subscriber_count(), 2);
    }

    #[test]
    fn test_sensor_stream_unsubscribe() {
        let mut reg = SensorStreamRegistry::new();
        let id = reg.subscribe();
        assert!(reg.unsubscribe(&id));
        assert_eq!(reg.subscriber_count(), 0);
    }

    #[test]
    fn test_sensor_stream_unsubscribe_nonexistent() {
        let mut reg = SensorStreamRegistry::new();
        assert!(!reg.unsubscribe("nonexistent"));
    }

    #[test]
    fn test_sensor_stream_broadcast_and_poll() {
        let mut reg = SensorStreamRegistry::new();
        let id = reg.subscribe();
        let events = vec![
            SensorEventIpc::PointerMove {
                x: 10.0,
                y: 20.0,
                timestamp_ms: 100,
            },
            SensorEventIpc::Click {
                x: 10.0,
                y: 20.0,
                button: "left".to_string(),
                timestamp_ms: 150,
            },
        ];
        reg.broadcast(&events);
        let batch = reg.poll(&id);
        assert_eq!(batch.subscription_id, id);
        assert_eq!(batch.events.len(), 2);
    }

    #[test]
    fn test_sensor_stream_poll_clears_queue() {
        let mut reg = SensorStreamRegistry::new();
        let id = reg.subscribe();
        reg.broadcast(&[SensorEventIpc::Scroll {
            delta_x: 0.0,
            delta_y: 1.0,
            timestamp_ms: 200,
        }]);
        let first = reg.poll(&id);
        assert_eq!(first.events.len(), 1);
        let second = reg.poll(&id);
        assert!(second.events.is_empty());
    }

    #[test]
    fn test_sensor_stream_broadcast_empty_is_noop() {
        let mut reg = SensorStreamRegistry::new();
        let id = reg.subscribe();
        reg.broadcast(&[]);
        let batch = reg.poll(&id);
        assert!(batch.events.is_empty());
    }

    #[test]
    fn test_sensor_stream_broadcast_to_multiple() {
        let mut reg = SensorStreamRegistry::new();
        let id1 = reg.subscribe();
        let id2 = reg.subscribe();
        reg.broadcast(&[SensorEventIpc::KeyPress {
            key: "A".to_string(),
            modifiers: petal_tongue_core::KeyModifiersIpc::default(),
            timestamp_ms: 300,
        }]);
        assert_eq!(reg.poll(&id1).events.len(), 1);
        assert_eq!(reg.poll(&id2).events.len(), 1);
    }

    #[test]
    fn test_sensor_stream_poll_unknown_subscriber() {
        let mut reg = SensorStreamRegistry::new();
        let batch = reg.poll("unknown");
        assert!(batch.events.is_empty());
        assert_eq!(batch.subscription_id, "unknown");
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
        let parsed: InteractionEventNotification =
            serde_json::from_str(&json).expect("deserialize");
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
        let resp = reg.apply_interaction(&req);
        assert!(resp.accepted);
        assert_eq!(resp.targets_resolved, 1);
        assert_eq!(reg.subscriber_count(), 0);
    }

    #[test]
    fn test_pending_callbacks_multiple_mixed() {
        let mut reg = InteractionSubscriberRegistry::new();
        reg.subscribe_with_filter("with_cb_events", vec![], Some("cb2".to_string()), None);
        let event = InteractionEventNotification {
            event_type: "select".to_string(),
            targets: vec![],
            timestamp: String::new(),
            perspective_id: None,
        };
        reg.broadcast(&event);
        reg.subscribe_with_filter("with_cb_empty", vec![], Some("cb1".to_string()), None);
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
        ));
        assert_eq!(reg.callback_method("sub1"), Some("cb"));
    }

    #[test]
    fn test_broadcast_returns_callback_dispatches() {
        let mut reg = InteractionSubscriberRegistry::new();
        reg.subscribe_with_filter("poll-sub", vec![], None, None);
        reg.subscribe_with_filter(
            "cb-sub",
            vec![],
            Some("spring.on_interaction".to_string()),
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
}
