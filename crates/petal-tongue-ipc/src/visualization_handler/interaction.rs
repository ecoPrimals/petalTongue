// SPDX-License-Identifier: AGPL-3.0-only
//! Interaction event subsystem: poll-based subscriptions for springs.
//!
//! Springs call `interaction.subscribe` to register, then `interaction.poll`
//! to drain queued interaction events. Supports callback-based delivery
//! (healthSpring V12 pattern) when `callback_method` is set.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{InteractionApplyRequest, InteractionApplyResponse, Perspective};

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
    pub fn callback_method(&self, subscriber_id: &str) -> Option<&str> {
        self.subscribers
            .get(subscriber_id)
            .and_then(|s| s.callback_method.as_deref())
    }

    /// Get all subscribers that have callback methods and pending events.
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
    pub fn broadcast(&mut self, event: &InteractionEventNotification) {
        for sub in self.subscribers.values_mut() {
            let passes_event_filter =
                sub.event_filter.is_empty() || sub.event_filter.contains(&event.event_type);
            if passes_event_filter {
                sub.queue.push(event.clone());
            }
        }
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
