// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensor event stream registry for raw input subscriptions.
//!
//! External primals subscribe to raw sensor events (pointer, keys, scroll)
//! for engagement analysis and AI adaptation by consumer primals/springs.
//! This is distinct from the semantic interaction event system in `interaction`:
//! sensor streams carry hardware-level input; interaction events carry
//! data-space intents.

use petal_tongue_core::{SensorEventBatch, SensorEventIpc};
use std::collections::HashMap;

/// Registry for sensor event stream subscribers.
///
/// External primals subscribe to raw sensor events (pointer, keys, scroll)
/// for engagement analysis and AI adaptation by consumer primals/springs.
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
}
