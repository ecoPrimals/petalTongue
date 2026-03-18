// SPDX-License-Identifier: AGPL-3.0-or-later
//! SSE (Server-Sent Events) client for biomeOS `/api/v1/events/stream`.
//!
//! biomeOS exposes ecosystem-level events via an SSE endpoint. This module
//! consumes the stream and converts events into typed `EcosystemEvent`s that
//! the UI can react to in real-time.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Ecosystem-level events from biomeOS SSE `/api/v1/events/stream`.
///
/// These complement the WebSocket `BiomeOSEvent`s (device/niche level) with
/// higher-level ecosystem topology events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EcosystemEvent {
    /// A new primal was discovered in the ecosystem.
    PrimalDiscovered {
        /// Primal identifier.
        primal_id: String,
        /// Human-readable name.
        name: String,
        /// Capabilities provided.
        capabilities: Vec<String>,
    },
    /// A primal's health status changed.
    HealthChanged {
        /// Primal identifier.
        primal_id: String,
        /// Previous health status.
        previous: String,
        /// New health status.
        current: String,
    },
    /// The ecosystem topology changed (primal added/removed/reconnected).
    TopologyChanged {
        /// Affected primal identifier.
        primal_id: String,
        /// Change type: "joined", "left", "reconnected".
        change: String,
    },
    /// A primal joined a family.
    FamilyJoined {
        /// Primal identifier.
        primal_id: String,
        /// Family name.
        family: String,
    },
    /// Trust level between primals was updated.
    TrustUpdated {
        /// Source primal.
        from: String,
        /// Target primal.
        to: String,
        /// New trust level (0.0..=1.0).
        trust_level: f64,
    },
}

/// SSE stream state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseConnectionState {
    /// Not connected.
    Disconnected,
    /// Attempting to connect.
    Connecting,
    /// Connected and receiving events.
    Connected,
    /// Connection failed (will retry).
    Failed,
}

type EventCallback = Box<dyn Fn(EcosystemEvent) + Send + Sync>;

/// SSE event consumer for biomeOS ecosystem events.
pub struct SseEventConsumer {
    endpoint: String,
    state: Arc<RwLock<SseConnectionState>>,
    events: Arc<RwLock<Vec<EcosystemEvent>>>,
    callback: Arc<RwLock<Option<EventCallback>>>,
}

impl SseEventConsumer {
    /// Create a new SSE consumer targeting the given endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            state: Arc::new(RwLock::new(SseConnectionState::Disconnected)),
            events: Arc::new(RwLock::new(Vec::new())),
            callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment / constants.
    #[must_use]
    pub fn from_defaults() -> Self {
        let endpoint = Self::resolve_endpoint();
        Self::new(endpoint)
    }

    /// Current connection state.
    pub async fn state(&self) -> SseConnectionState {
        self.state.read().await.clone()
    }

    /// Set a callback for incoming events.
    pub async fn set_callback<F>(&self, callback: F)
    where
        F: Fn(EcosystemEvent) + Send + Sync + 'static,
    {
        *self.callback.write().await = Some(Box::new(callback));
    }

    /// Drain all buffered events (returns and clears the buffer).
    pub async fn drain_events(&self) -> Vec<EcosystemEvent> {
        let mut events = self.events.write().await;
        std::mem::take(&mut *events)
    }

    /// Start consuming the SSE stream in a background task.
    ///
    /// Returns immediately. The stream is consumed on a tokio task.
    pub fn start(&self) {
        let endpoint = self.endpoint.clone();
        let state = Arc::clone(&self.state);
        let events = Arc::clone(&self.events);
        let callback = Arc::clone(&self.callback);

        tokio::spawn(async move {
            Self::consume_loop(endpoint, state, events, callback).await;
        });
    }

    /// Parse a single SSE `data:` line into an `EcosystemEvent`.
    fn parse_sse_data(data: &str) -> Option<EcosystemEvent> {
        serde_json::from_str(data).ok()
    }

    /// Resolve the SSE endpoint from environment or constants.
    fn resolve_endpoint() -> String {
        if let Ok(url) = std::env::var("BIOMEOS_SSE_ENDPOINT") {
            return url;
        }
        let base = petal_tongue_core::constants::default_web_url();
        format!("{base}/api/v1/events/stream")
    }

    async fn consume_loop(
        endpoint: String,
        state: Arc<RwLock<SseConnectionState>>,
        events: Arc<RwLock<Vec<EcosystemEvent>>>,
        callback: Arc<RwLock<Option<EventCallback>>>,
    ) {
        use futures_util::StreamExt;

        *state.write().await = SseConnectionState::Connecting;
        info!("SSE: connecting to {}", endpoint);

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(0))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("SSE: failed to build HTTP client: {e}");
                *state.write().await = SseConnectionState::Failed;
                return;
            }
        };

        let response = match client
            .get(&endpoint)
            .header("Accept", "text/event-stream")
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                warn!("SSE: server returned {}", r.status());
                *state.write().await = SseConnectionState::Failed;
                return;
            }
            Err(e) => {
                debug!("SSE: connection failed (biomeOS may not be running): {e}");
                *state.write().await = SseConnectionState::Failed;
                return;
            }
        };

        *state.write().await = SseConnectionState::Connected;
        info!("SSE: connected to {}", endpoint);

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);

                    while let Some(pos) = buffer.find("\n\n") {
                        let message = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();

                        for line in message.lines() {
                            if let Some(data) = line
                                .strip_prefix("data: ")
                                .or_else(|| line.strip_prefix("data:"))
                                && let Some(event) = Self::parse_sse_data(data.trim())
                            {
                                let cb = callback.read().await;
                                if let Some(ref f) = *cb {
                                    f(event.clone());
                                }
                                drop(cb);
                                events.write().await.push(event);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("SSE: stream error: {e}");
                    *state.write().await = SseConnectionState::Failed;
                    return;
                }
            }
        }

        *state.write().await = SseConnectionState::Disconnected;
        info!("SSE: stream ended");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_event_primal_discovered_roundtrip() {
        let event = EcosystemEvent::PrimalDiscovered {
            primal_id: "air-1".to_string(),
            name: "airSpring".to_string(),
            capabilities: vec!["science.et0".to_string()],
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EcosystemEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            EcosystemEvent::PrimalDiscovered { primal_id, .. } => {
                assert_eq!(primal_id, "air-1");
            }
            _ => panic!("expected PrimalDiscovered"),
        }
    }

    #[test]
    fn ecosystem_event_health_changed_roundtrip() {
        let event = EcosystemEvent::HealthChanged {
            primal_id: "health-1".to_string(),
            previous: "healthy".to_string(),
            current: "degraded".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EcosystemEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            EcosystemEvent::HealthChanged {
                previous, current, ..
            } => {
                assert_eq!(previous, "healthy");
                assert_eq!(current, "degraded");
            }
            _ => panic!("expected HealthChanged"),
        }
    }

    #[test]
    fn ecosystem_event_topology_changed_roundtrip() {
        let event = EcosystemEvent::TopologyChanged {
            primal_id: "neural-1".to_string(),
            change: "joined".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EcosystemEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            EcosystemEvent::TopologyChanged { change, .. } => {
                assert_eq!(change, "joined");
            }
            _ => panic!("expected TopologyChanged"),
        }
    }

    #[test]
    fn ecosystem_event_family_joined_roundtrip() {
        let event = EcosystemEvent::FamilyJoined {
            primal_id: "ground-1".to_string(),
            family: "groundSpring".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EcosystemEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            EcosystemEvent::FamilyJoined { family, .. } => {
                assert_eq!(family, "groundSpring");
            }
            _ => panic!("expected FamilyJoined"),
        }
    }

    #[test]
    fn ecosystem_event_trust_updated_roundtrip() {
        let event = EcosystemEvent::TrustUpdated {
            from: "a".to_string(),
            to: "b".to_string(),
            trust_level: 0.95,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EcosystemEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            EcosystemEvent::TrustUpdated { trust_level, .. } => {
                assert!((trust_level - 0.95).abs() < f64::EPSILON);
            }
            _ => panic!("expected TrustUpdated"),
        }
    }

    #[test]
    fn parse_sse_data_valid() {
        let data = r#"{"type":"PrimalDiscovered","primal_id":"x","name":"X","capabilities":["a"]}"#;
        let event = SseEventConsumer::parse_sse_data(data);
        assert!(event.is_some());
    }

    #[test]
    fn parse_sse_data_invalid_returns_none() {
        assert!(SseEventConsumer::parse_sse_data("not json").is_none());
    }

    #[test]
    fn sse_consumer_construction() {
        let consumer = SseEventConsumer::new("http://localhost:8080/api/v1/events/stream");
        assert_eq!(
            consumer.endpoint,
            "http://localhost:8080/api/v1/events/stream"
        );
    }

    #[test]
    fn parse_sse_data_with_data_prefix() {
        let data = r#"{"type":"PrimalDiscovered","primal_id":"x","name":"X","capabilities":["a"]}"#;
        let event = SseEventConsumer::parse_sse_data(data);
        assert!(event.is_some());
        let ev = event.unwrap();
        match ev {
            EcosystemEvent::PrimalDiscovered { primal_id, .. } => assert_eq!(primal_id, "x"),
            _ => panic!("expected PrimalDiscovered"),
        }
    }

    #[test]
    fn sse_connection_state_variants() {
        assert_eq!(
            SseConnectionState::Disconnected,
            SseConnectionState::Disconnected
        );
        assert_ne!(
            SseConnectionState::Disconnected,
            SseConnectionState::Connected
        );
    }

    #[test]
    fn ecosystem_event_serde_tag() {
        let json =
            r#"{"type":"HealthChanged","primal_id":"p1","previous":"ok","current":"degraded"}"#;
        let parsed: EcosystemEvent = serde_json::from_str(json).unwrap();
        match parsed {
            EcosystemEvent::HealthChanged { current, .. } => assert_eq!(current, "degraded"),
            _ => panic!("expected HealthChanged"),
        }
    }

    #[test]
    fn sse_from_defaults_creates_consumer() {
        let consumer = SseEventConsumer::from_defaults();
        assert!(!consumer.endpoint.is_empty());
        assert!(consumer.endpoint.contains("stream"));
    }

    #[tokio::test]
    async fn sse_consumer_initial_state_is_disconnected() {
        let consumer = SseEventConsumer::new("http://localhost:0/nope");
        assert_eq!(consumer.state().await, SseConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn sse_consumer_drain_empty() {
        let consumer = SseEventConsumer::new("http://localhost:0/nope");
        let events = consumer.drain_events().await;
        assert!(events.is_empty());
    }
}
