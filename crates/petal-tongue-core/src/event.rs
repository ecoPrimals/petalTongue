// SPDX-License-Identifier: AGPL-3.0-only
//! # Event System
//!
//! Coordinates events across multiple modalities.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// Engine Event
///
/// Events broadcast to all modalities for coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EngineEvent {
    /// Graph structure changed
    GraphUpdated {
        /// Added node IDs
        added_nodes: Vec<String>,
        /// Removed node IDs
        removed_nodes: Vec<String>,
        /// Added edge count
        added_edges: usize,
        /// Removed edge count
        removed_edges: usize,
    },

    /// Selection changed
    SelectionChanged {
        /// Currently selected node IDs
        selected: HashSet<String>,
    },

    /// View changed
    ViewChanged {
        /// Viewport center X
        center_x: f32,
        /// Viewport center Y
        center_y: f32,
        /// Zoom level
        zoom: f32,
    },

    /// User interaction
    UserInteraction {
        /// Modality that initiated interaction
        modality: String,
        /// Action performed
        action: String,
        /// Optional target (node ID)
        target: Option<String>,
    },

    /// State update
    StateUpdate {
        /// State key
        key: String,
        /// State value (JSON)
        value: serde_json::Value,
    },

    /// Modality started
    ModalityStarted {
        /// Modality name
        name: String,
    },

    /// Modality stopped
    ModalityStopped {
        /// Modality name
        name: String,
    },

    /// Shutdown signal
    Shutdown,

    /// Awakening stage transition
    AwakeningStage {
        /// New stage
        stage: String,
        /// Stage message
        message: String,
    },

    /// Awakening visual frame
    AwakeningVisual {
        /// Frame type (flower state)
        frame_type: String,
        /// Frame index
        frame: usize,
    },

    /// Awakening audio event
    AwakeningAudio {
        /// Audio layer name
        layer: String,
        /// Action (start/stop)
        action: String,
    },

    /// Awakening text message
    AwakeningText {
        /// Text message
        message: String,
    },

    /// Primal discovered during awakening
    PrimalDiscovered {
        /// Primal name
        name: String,
        /// Discovery index
        index: usize,
    },

    /// Interaction engine event (semantic intent resolved to data)
    Interaction {
        /// Semantic event type (select, focus, navigate, etc.)
        event_type: String,
        /// Resolved data-space targets as JSON.
        targets: Vec<serde_json::Value>,
        /// Which perspective originated this.
        perspective_id: u64,
    },
}

/// Event Bus
///
/// Broadcasts events to all subscribers (modalities).
pub struct EventBus {
    /// Broadcast channel
    tx: broadcast::Sender<EngineEvent>,
    /// Active subscriber count
    subscriber_count: Arc<RwLock<usize>>,
}

impl EventBus {
    /// Create new event bus
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000);
        Self {
            tx,
            subscriber_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Subscribe to events
    pub async fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        let mut count = self.subscriber_count.write().await;
        *count += 1;
        self.tx.subscribe()
    }

    /// Unsubscribe (called when receiver is dropped)
    pub async fn unsubscribe(&self) {
        let mut count = self.subscriber_count.write().await;
        if *count > 0 {
            *count -= 1;
        }
    }

    /// Broadcast event to all subscribers
    #[expect(
        clippy::unused_async,
        reason = "async for future broadcast::Sender API"
    )]
    pub async fn broadcast(&self, event: EngineEvent) -> Result<usize, String> {
        self.tx
            .send(event)
            .map_err(|e| format!("Failed to broadcast event: {e}"))
    }

    /// Get subscriber count
    pub async fn subscriber_count(&self) -> usize {
        *self.subscriber_count.read().await
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_event_broadcast() {
        let bus = EventBus::new();

        // Subscribe
        let mut rx1 = bus.subscribe().await;
        let mut rx2 = bus.subscribe().await;

        assert_eq!(bus.subscriber_count().await, 2);

        // Broadcast event
        let event = EngineEvent::GraphUpdated {
            added_nodes: vec!["node1".into()],
            removed_nodes: vec![],
            added_edges: 1,
            removed_edges: 0,
        };

        let result = bus.broadcast(event.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // 2 receivers

        // Both receivers should get the event (with timeout to prevent hangs)
        let received1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        let received2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");

        match (&received1, &received2) {
            (
                EngineEvent::GraphUpdated {
                    added_nodes: a1, ..
                },
                EngineEvent::GraphUpdated {
                    added_nodes: a2, ..
                },
            ) => {
                assert_eq!(a1, a2);
                assert_eq!(a1.len(), 1);
            }
            _ => unreachable!("expected GraphUpdated events"),
        }
    }

    #[tokio::test]
    async fn test_subscriber_lifecycle() {
        let bus = EventBus::new();

        {
            let _rx1 = bus.subscribe().await;
            assert_eq!(bus.subscriber_count().await, 1);

            {
                let _rx2 = bus.subscribe().await;
                assert_eq!(bus.subscriber_count().await, 2);
            }
            // rx2 dropped, but we need to manually call unsubscribe
            // In real usage, modalities would call this in their Drop impl
        }
    }

    #[tokio::test]
    async fn test_selection_event() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe().await;

        let mut selected = HashSet::new();
        selected.insert("node1".to_string());

        let event = EngineEvent::SelectionChanged {
            selected: selected.clone(),
        };
        bus.broadcast(event).await.unwrap();

        let received = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        match received {
            EngineEvent::SelectionChanged { selected: s } => {
                assert_eq!(s.len(), 1);
                assert!(s.contains("node1"));
            }
            _ => unreachable!("expected SelectionChanged event"),
        }
    }
}
