// SPDX-License-Identifier: AGPL-3.0-or-later
//! Interaction event subsystem: dual-mode subscriptions with push delivery (PT-06).
//!
//! Springs call `interaction.subscribe` to register, then either:
//! - **Poll**: call `interaction.poll` to drain queued events (always available), or
//! - **Push**: receive JSON-RPC notifications at `callback_socket` (when set with
//!   `callback_method`), completing the healthSpring V12 callback pattern.
//!
//! ## Delivery model
//!
//! Events are always queued for poll. When `callback_method` and `callback_socket`
//! are both set on a subscription, `broadcast()` additionally returns
//! [`CallbackDispatch`] entries. The RPC handler sends these through the push
//! delivery channel (`push_delivery` module), which writes JSON-RPC notifications
//! to subscriber sockets as a background task.
//!
//! `apply_interaction` captures dispatches and returns a `pending_callbacks` count
//! in the response. Subscribers whose socket is unreachable can still poll.
//!
//! ## Zero-copy
//!
//! Events are wrapped in `Arc` in subscriber queues so that a single broadcast
//! shares one allocation across N subscribers instead of cloning N times.
//! `poll()` uses `Arc::unwrap_or_clone` to return owned events without copying
//! when the subscriber holds the last reference.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
    /// Socket path for push delivery (UDS or `host:port` for TCP).
    /// When `None`, the subscriber is poll-only even with `callback_method`.
    pub callback_socket: Option<String>,
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
    queue: Vec<Arc<InteractionEventNotification>>,
    event_filter: Vec<String>,
    callback_method: Option<String>,
    callback_socket: Option<String>,
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
        self.subscribe_with_filter(subscriber_id, Vec::new(), None, None, None)
    }

    /// Register a subscriber with event type filter, callback method, and grammar filter.
    ///
    /// The callback model (healthSpring V12 pattern): when `callback_method` is set
    /// together with `callback_socket`, events are pushed as JSON-RPC notifications
    /// to the subscriber's socket. Without `callback_socket`, subscribers use poll.
    pub fn subscribe_with_filter(
        &mut self,
        subscriber_id: &str,
        event_filter: Vec<String>,
        callback_method: Option<String>,
        _grammar_id: Option<String>,
        callback_socket: Option<String>,
    ) -> bool {
        use std::collections::hash_map::Entry;
        match self.subscribers.entry(subscriber_id.to_string()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(e) => {
                e.insert(InteractionSubscriber {
                    queue: Vec::new(),
                    event_filter,
                    callback_method,
                    callback_socket,
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
    pub fn pending_callbacks(&self) -> Vec<(&str, &str, Vec<&InteractionEventNotification>)> {
        self.subscribers
            .iter()
            .filter_map(|(id, sub)| {
                sub.callback_method
                    .as_deref()
                    .filter(|_| !sub.queue.is_empty())
                    .map(|method| {
                        let events: Vec<&InteractionEventNotification> =
                            sub.queue.iter().map(AsRef::as_ref).collect();
                        (id.as_str(), method, events)
                    })
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
        let shared = Arc::new(event.clone());
        let mut callbacks = Vec::new();
        for (id, sub) in &mut self.subscribers {
            let passes_event_filter =
                sub.event_filter.is_empty() || sub.event_filter.contains(&event.event_type);
            if passes_event_filter {
                sub.queue.push(Arc::clone(&shared));

                if let Some(method) = &sub.callback_method {
                    callbacks.push(CallbackDispatch {
                        subscriber_id: id.clone(),
                        method: method.clone(),
                        events: vec![event.clone()],
                        callback_socket: sub.callback_socket.clone(),
                    });
                }
            }
        }
        callbacks
    }

    /// Drain queued events for a subscriber, returning owned copies.
    ///
    /// Events are stored as `Arc` internally for zero-copy broadcast sharing;
    /// `poll` unwraps or clones as needed.
    pub fn poll(&mut self, subscriber_id: &str) -> Vec<InteractionEventNotification> {
        self.subscribers
            .get_mut(subscriber_id)
            .map(|sub| {
                std::mem::take(&mut sub.queue)
                    .into_iter()
                    .map(Arc::unwrap_or_clone)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Apply an interaction intent and broadcast to subscribers (PT-06).
    ///
    /// Returns the response AND any pending callback dispatches for push delivery.
    /// The caller (RPC handler) is responsible for sending the callback dispatches
    /// as JSON-RPC notifications to subscriber sockets.
    pub fn apply_interaction(
        &mut self,
        req: &InteractionApplyRequest,
    ) -> (InteractionApplyResponse, Vec<CallbackDispatch>) {
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
        let callbacks = self.broadcast(&event);
        let response = InteractionApplyResponse {
            accepted: true,
            targets_resolved: req.targets.len(),
            pending_callbacks: callbacks.len(),
        };
        (response, callbacks)
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
}
