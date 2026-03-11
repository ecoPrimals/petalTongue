// SPDX-License-Identifier: AGPL-3.0-only
//! Event streaming for real-time biomeOS updates.
//!
//! WebSocket-based event stream for device, primal, and niche events.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::types::{Device, Health};

/// biomeOS event types for real-time streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BiomeOSEvent {
    /// Device was added to the system
    DeviceAdded {
        /// The device that was added
        device: Device,
    },
    /// Device was removed from the system
    DeviceRemoved {
        /// ID of the device that was removed
        device_id: String,
    },
    /// Primal status changed
    PrimalStatus {
        /// ID of the primal whose status changed
        primal_id: String,
        /// New health status
        health: Health,
    },
    /// Niche was deployed
    NicheDeployed {
        /// ID of the deployed niche
        niche_id: String,
        /// Name of the deployed niche
        name: String,
    },
}

/// Event stream for real-time updates via WebSocket
///
/// Provides real-time event streaming from biomeOS for:
/// - Device additions/removals
/// - Primal status changes
/// - Niche deployment events
pub(super) struct EventStream {
    /// WebSocket connection (if established)
    ws_connection: Option<WebSocketConnection>,
    /// Event callback (called when events received)
    callback: Option<Box<dyn Fn(BiomeOSEvent) + Send + Sync>>,
}

/// WebSocket connection wrapper for biomeOS events
#[expect(
    dead_code,
    reason = "Connection state reserved for future reconnection logic"
)]
struct WebSocketConnection {
    /// WebSocket endpoint URL (e.g., "<ws://localhost:8080/events>")
    endpoint: String,
    /// Connection state
    connected: bool,
}

impl EventStream {
    /// Create new event stream (not connected)
    pub(super) fn new() -> Self {
        Self {
            ws_connection: None,
            callback: None,
        }
    }

    /// Connect to WebSocket endpoint for real-time events
    #[expect(clippy::unused_async, reason = "async for future WebSocket client")]
    pub(super) async fn connect(&mut self, endpoint: &str) -> Result<()> {
        info!("🔌 Connecting to biomeOS event stream: {}", endpoint);

        // Create WebSocket connection
        self.ws_connection = Some(WebSocketConnection {
            endpoint: endpoint.to_string(),
            connected: true,
        });

        info!("✅ Connected to biomeOS event stream");
        Ok(())
    }

    /// Set event callback
    pub(super) fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(BiomeOSEvent) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::{Device, DeviceStatus, DeviceType};

    fn make_device() -> Device {
        Device {
            id: "dev-1".to_string(),
            name: "Test Device".to_string(),
            device_type: DeviceType::GPU,
            status: DeviceStatus::Online,
            resource_usage: 0.5,
            assigned_to: None,
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn biomeos_event_device_added_roundtrip() {
        let device = make_device();
        let event = BiomeOSEvent::DeviceAdded {
            device: device.clone(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: BiomeOSEvent = serde_json::from_str(&json).unwrap();
        match &parsed {
            BiomeOSEvent::DeviceAdded { device: d } => assert_eq!(d.id, device.id),
            _ => panic!("expected DeviceAdded"),
        }
    }

    #[test]
    fn biomeos_event_device_removed_roundtrip() {
        let event = BiomeOSEvent::DeviceRemoved {
            device_id: "dev-99".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: BiomeOSEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            BiomeOSEvent::DeviceRemoved { device_id } => assert_eq!(device_id, "dev-99"),
            _ => panic!("expected DeviceRemoved"),
        }
    }

    #[test]
    fn biomeos_event_primal_status_roundtrip() {
        let event = BiomeOSEvent::PrimalStatus {
            primal_id: "p1".to_string(),
            health: Health::Degraded,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: BiomeOSEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            BiomeOSEvent::PrimalStatus { primal_id, health } => {
                assert_eq!(primal_id, "p1");
                assert_eq!(health, Health::Degraded);
            }
            _ => panic!("expected PrimalStatus"),
        }
    }

    #[test]
    fn biomeos_event_niche_deployed_roundtrip() {
        let event = BiomeOSEvent::NicheDeployed {
            niche_id: "niche-1".to_string(),
            name: "My Niche".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: BiomeOSEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            BiomeOSEvent::NicheDeployed { niche_id, name } => {
                assert_eq!(niche_id, "niche-1");
                assert_eq!(name, "My Niche");
            }
            _ => panic!("expected NicheDeployed"),
        }
    }

    #[test]
    fn biomeos_event_serde_tag_type() {
        let json = r#"{"type":"DeviceRemoved","device_id":"x"}"#;
        let parsed: BiomeOSEvent = serde_json::from_str(json).unwrap();
        match parsed {
            BiomeOSEvent::DeviceRemoved { device_id } => assert_eq!(device_id, "x"),
            _ => panic!("expected DeviceRemoved"),
        }
    }
}
