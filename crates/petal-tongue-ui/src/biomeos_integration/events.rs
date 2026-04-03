// SPDX-License-Identifier: AGPL-3.0-or-later
//! Event streaming for real-time biomeOS updates.
//!
//! WebSocket-based event stream for device, primal, and niche events.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
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

/// WebSocket connection wrapper for biomeOS events (state for reconnect + future real client).
struct WebSocketConnection {
    /// WebSocket endpoint URL (e.g., "<ws://localhost:8080/events>")
    endpoint: String,
    /// Whether the last known state is connected (updated by `connect` / [`EventStream::on_connection_lost`]).
    connected: bool,
    /// Count of disconnects since the last successful `connect` (drives exponential backoff).
    consecutive_failures: u32,
}

impl WebSocketConnection {
    const MAX_BACKOFF_SECS: u64 = 60;

    /// Delay to wait before the next `connect` after `consecutive_failures` disconnects.
    #[must_use]
    fn reconnect_delay_after_failures(failures: u32) -> Duration {
        if failures == 0 {
            return Duration::ZERO;
        }
        let pow = failures.saturating_sub(1).min(16);
        let secs = (1u64 << pow).min(Self::MAX_BACKOFF_SECS);
        Duration::from_secs(secs)
    }

    #[must_use]
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn mark_connected(&mut self, endpoint: impl Into<String>) {
        self.endpoint = endpoint.into();
        self.connected = true;
        self.consecutive_failures = 0;
    }
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
    pub(super) async fn connect(&mut self, endpoint: &str) -> Result<()> {
        info!("🔌 Connecting to biomeOS event stream: {}", endpoint);

        let mut ws = self.ws_connection.take().unwrap_or(WebSocketConnection {
            endpoint: String::new(),
            connected: false,
            consecutive_failures: 0,
        });

        let failures_before = ws.consecutive_failures;
        if failures_before > 0 {
            let delay = WebSocketConnection::reconnect_delay_after_failures(failures_before);
            tracing::info!(
                "⏳ Waiting {delay:?} before reconnect (after {failures_before} disconnect(s))"
            );
            tokio::time::sleep(delay).await;
        }

        ws.mark_connected(endpoint);

        info!("✅ Connected to biomeOS event stream at {}", ws.endpoint());
        self.ws_connection = Some(ws);
        debug_assert!(self.is_connected());
        Ok(())
    }

    /// Mark the WebSocket as dropped (e.g. read error or close frame). Next [`Self::connect`] applies exponential backoff.
    ///
    /// Call this from the future `tokio-tungstenite` read loop when the stream ends or errors.
    #[expect(
        dead_code,
        reason = "reserved for the real WebSocket client; backoff state is ready"
    )]
    pub(super) fn on_connection_lost(&mut self) {
        if let Some(ref mut ws) = self.ws_connection {
            tracing::warn!("biomeOS event WebSocket disconnected: {}", ws.endpoint());
            ws.connected = false;
            ws.consecutive_failures = ws.consecutive_failures.saturating_add(1);
        }
    }

    /// Whether the last `connect` left the stream in a connected state (false after [`Self::on_connection_lost`]).
    #[must_use]
    pub(super) fn is_connected(&self) -> bool {
        self.ws_connection.as_ref().is_some_and(|w| w.connected)
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

    #[test]
    fn reconnect_backoff_delay_sequence() {
        use std::time::Duration;

        assert_eq!(
            WebSocketConnection::reconnect_delay_after_failures(0),
            Duration::ZERO
        );
        assert_eq!(
            WebSocketConnection::reconnect_delay_after_failures(1),
            Duration::from_secs(1)
        );
        assert_eq!(
            WebSocketConnection::reconnect_delay_after_failures(2),
            Duration::from_secs(2)
        );
        assert_eq!(
            WebSocketConnection::reconnect_delay_after_failures(3),
            Duration::from_secs(4)
        );
    }
}
