// SPDX-License-Identifier: AGPL-3.0-or-later
//! UI Events - Event-Driven Architecture for Real-Time Updates
//!
//! Defines all events that can occur in the device/niche management UI
//! and provides a centralized event handler for dispatching updates.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Event Sources                                               │
//! │  ├─ biomeOS (WebSocket)                                     │
//! │  ├─ User Actions (drag-and-drop, clicks)                    │
//! │  └─ AI Suggestions                                          │
//! └────────────────────┬────────────────────────────────────────┘
//!                      ↓
//!              UIEventHandler
//!                      ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Event Consumers                                             │
//! │  ├─ DevicePanel (update device list)                        │
//! │  ├─ PrimalPanel (update primal status)                      │
//! │  └─ NicheDesigner (update graph)                            │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::biomeos_integration::{Device, DeviceStatus, Health, NicheTemplate, Primal};

/// UI event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEvent {
    // Device events
    /// New device discovered
    DeviceDiscovered(Device),
    /// Device removed (`device_id`)
    DeviceRemoved(String),
    /// Device status changed (`device_id`, `new_status`)
    DeviceStatusChanged(String, DeviceStatus),
    /// Device usage changed (`device_id`, `new_usage` 0.0-1.0)
    DeviceUsageChanged(String, f64),

    // Primal events
    /// New primal discovered
    PrimalDiscovered(Primal),
    /// Primal removed (`primal_id`)
    PrimalRemoved(String),
    /// Primal health changed (`primal_id`, `new_health`)
    PrimalHealthChanged(String, Health),
    /// Primal load changed (`primal_id`, `new_load` 0.0-1.0)
    PrimalLoadChanged(String, f64),

    // Assignment events
    /// Device assigned to primal (`device_id`, `primal_id`)
    DeviceAssigned(String, String),
    /// Device unassigned from primal (`device_id`, `primal_id`)
    DeviceUnassigned(String, String),

    // Niche events
    /// Niche deployed (`niche_id`, template)
    NicheDeployed(String, NicheTemplate),
    /// Niche removed (`niche_id`)
    NicheRemoved(String),

    // AI suggestion events
    /// AI generated a suggestion
    AISuggestion(Suggestion),
    /// User accepted suggestion (`suggestion_id`)
    SuggestionAccepted(String),
    /// User rejected suggestion (`suggestion_id`)
    SuggestionRejected(String),
}

/// AI suggestion for system optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique suggestion identifier
    pub id: String,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Suggested actions to take
    pub actions: Vec<SuggestedAction>,
}

/// Suggestion type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// Suggest device assignment to primal
    DeviceAssignment,
    /// Suggest niche optimization
    NicheOptimization,
    /// Suggest resource rebalancing
    ResourceRebalancing,
    /// Warning about health issue
    HealthWarning,
}

/// Suggested action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestedAction {
    /// Assign device to primal
    AssignDevice {
        /// Device to assign
        device_id: String,
        /// Primal to receive device
        primal_id: String,
    },
    /// Unassign device from primal
    UnassignDevice {
        /// Device to unassign
        device_id: String,
        /// Primal to lose device
        primal_id: String,
    },
    /// Deploy new niche
    DeployNiche {
        /// Template to use for deployment
        template_id: String,
    },
    /// Remove existing niche
    RemoveNiche {
        /// Niche to remove
        niche_id: String,
    },
}

/// UI event handler - centralized dispatch of events to UI components
///
/// This is the heart of the real-time update system. All events flow through
/// here and are dispatched to the appropriate UI components.
pub struct UIEventHandler {
    // Event subscribers (UI components that need updates)
    device_panel_events: Arc<RwLock<Vec<UIEvent>>>,
    primal_panel_events: Arc<RwLock<Vec<UIEvent>>>,
    niche_designer_events: Arc<RwLock<Vec<UIEvent>>>,

    // Statistics
    events_processed: Arc<RwLock<usize>>,
}

impl UIEventHandler {
    /// Create a new UI event handler
    #[must_use]
    pub fn new() -> Self {
        info!("📡 Creating UI event handler");

        Self {
            device_panel_events: Arc::new(RwLock::new(Vec::new())),
            primal_panel_events: Arc::new(RwLock::new(Vec::new())),
            niche_designer_events: Arc::new(RwLock::new(Vec::new())),
            events_processed: Arc::new(RwLock::new(0)),
        }
    }

    /// Handle an incoming event
    ///
    /// Dispatches the event to all relevant UI components.
    pub async fn handle_event(&self, event: UIEvent) {
        debug!("📥 Handling event: {:?}", event);

        // Increment counter
        {
            let mut count = self.events_processed.write().await;
            *count += 1;
        }

        // Dispatch to appropriate panels based on event type
        match &event {
            UIEvent::DeviceDiscovered(_)
            | UIEvent::DeviceRemoved(_)
            | UIEvent::DeviceStatusChanged(_, _)
            | UIEvent::DeviceUsageChanged(_, _) => {
                self.notify_device_panel(event.clone()).await;
            }

            UIEvent::PrimalDiscovered(_)
            | UIEvent::PrimalRemoved(_)
            | UIEvent::PrimalHealthChanged(_, _)
            | UIEvent::PrimalLoadChanged(_, _) => {
                self.notify_primal_panel(event.clone()).await;
            }

            UIEvent::DeviceAssigned(_, _) | UIEvent::DeviceUnassigned(_, _) => {
                // Assignment affects both panels
                self.notify_device_panel(event.clone()).await;
                self.notify_primal_panel(event.clone()).await;
            }

            UIEvent::NicheDeployed(_, _) | UIEvent::NicheRemoved(_) => {
                self.notify_niche_designer(event.clone()).await;
            }

            UIEvent::AISuggestion(_)
            | UIEvent::SuggestionAccepted(_)
            | UIEvent::SuggestionRejected(_) => {
                // AI suggestions go to all panels
                self.notify_device_panel(event.clone()).await;
                self.notify_primal_panel(event.clone()).await;
                self.notify_niche_designer(event.clone()).await;
            }
        }
    }

    /// Notify device panel of event
    async fn notify_device_panel(&self, event: UIEvent) {
        let mut events = self.device_panel_events.write().await;
        events.push(event);
    }

    /// Notify primal panel of event
    async fn notify_primal_panel(&self, event: UIEvent) {
        let mut events = self.primal_panel_events.write().await;
        events.push(event);
    }

    /// Notify niche designer of event
    async fn notify_niche_designer(&self, event: UIEvent) {
        let mut events = self.niche_designer_events.write().await;
        events.push(event);
    }

    /// Get pending events for device panel (consumes events)
    pub async fn consume_device_panel_events(&self) -> Vec<UIEvent> {
        let mut events = self.device_panel_events.write().await;
        std::mem::take(&mut *events)
    }

    /// Get pending events for primal panel (consumes events)
    pub async fn consume_primal_panel_events(&self) -> Vec<UIEvent> {
        let mut events = self.primal_panel_events.write().await;
        std::mem::take(&mut *events)
    }

    /// Get pending events for niche designer (consumes events)
    pub async fn consume_niche_designer_events(&self) -> Vec<UIEvent> {
        let mut events = self.niche_designer_events.write().await;
        std::mem::take(&mut *events)
    }

    /// Get total events processed
    pub async fn events_processed(&self) -> usize {
        *self.events_processed.read().await
    }
}

impl Default for UIEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::biomeos_integration::DeviceType;
    use super::*;

    #[tokio::test]
    async fn test_event_handler_creation() {
        let handler = UIEventHandler::new();
        assert_eq!(handler.events_processed().await, 0);
    }

    #[tokio::test]
    async fn test_device_event_dispatch() {
        let handler = UIEventHandler::new();

        let device = Device {
            id: "test-device".to_string(),
            name: "Test Device".to_string(),
            device_type: DeviceType::GPU,
            status: DeviceStatus::Online,
            resource_usage: 0.5,
            assigned_to: None,
            metadata: serde_json::json!({}),
        };

        handler
            .handle_event(UIEvent::DeviceDiscovered(device))
            .await;

        assert_eq!(handler.events_processed().await, 1);

        let events = handler.consume_device_panel_events().await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_primal_event_dispatch() {
        let handler = UIEventHandler::new();

        let primal = Primal {
            id: "test-primal".to_string(),
            name: "Test Primal".to_string(),
            capabilities: vec!["test".to_string()],
            health: Health::Healthy,
            load: 0.3,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        handler
            .handle_event(UIEvent::PrimalDiscovered(primal))
            .await;

        assert_eq!(handler.events_processed().await, 1);

        let events = handler.consume_primal_panel_events().await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_assignment_event_dispatch_to_both_panels() {
        let handler = UIEventHandler::new();

        handler
            .handle_event(UIEvent::DeviceAssigned(
                "device-1".to_string(),
                "primal-1".to_string(),
            ))
            .await;

        assert_eq!(handler.events_processed().await, 1);

        let device_events = handler.consume_device_panel_events().await;
        let primal_events = handler.consume_primal_panel_events().await;

        assert_eq!(device_events.len(), 1, "Device panel should get event");
        assert_eq!(primal_events.len(), 1, "Primal panel should get event");
    }

    #[tokio::test]
    async fn test_event_consumption() {
        let handler = UIEventHandler::new();

        // Add multiple events
        for i in 0..5 {
            handler
                .handle_event(UIEvent::DeviceRemoved(format!("device-{}", i)))
                .await;
        }

        assert_eq!(handler.events_processed().await, 5);

        // First consumption should get all 5
        let events = handler.consume_device_panel_events().await;
        assert_eq!(events.len(), 5);

        // Second consumption should get empty (events are consumed)
        let events = handler.consume_device_panel_events().await;
        assert_eq!(events.len(), 0);
    }
}
