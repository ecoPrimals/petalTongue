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
    ///
    /// Stored as `Arc` to share the broadcast allocation with subscriber queues
    /// instead of cloning the full event per callback.
    pub events: Vec<Arc<InteractionEventNotification>>,
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
                        events: vec![Arc::clone(&shared)],
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
#[path = "interaction_tests.rs"]
mod tests;
