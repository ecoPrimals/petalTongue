// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
//! Real-time telemetry and event streaming
//!
//! This crate provides telemetry collection, aggregation, and event streaming
//! for the petalTongue visualization system.
//!
//! # Design Philosophy
//!
//! - **Real-time**: Events streamed as they occur
//! - **Aggregation**: Metrics computed on-the-fly
//! - **Subscriber pattern**: Multiple consumers of telemetry
//! - **Non-blocking**: Async event processing
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Source    │ (BiomeOS, Primals)
//! └──────┬──────┘
//!        │ events
//!        ▼
//! ┌─────────────────┐
//! │ Event Collector │
//! └──────┬──────────┘
//!        │
//!        ├──> Buffer
//!        ├──> Aggregator
//!        └──> Subscribers
//! ```

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Telemetry event types
#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryEvent {
    /// Primal discovered in the ecosystem
    PrimalDiscovered {
        /// Primal identifier
        primal_id: String,
        /// Primal type (e.g., "compute", "storage")
        primal_type: String,
        /// Available capabilities
        capabilities: Vec<String>,
        /// Endpoint URL
        endpoint: String,
        /// Timestamp
        timestamp: SystemTime,
    },

    /// Primal disappeared from the ecosystem
    PrimalDisappeared {
        /// Primal identifier
        primal_id: String,
        /// Timestamp
        timestamp: SystemTime,
    },

    /// Health status updated
    HealthUpdate {
        /// Primal identifier
        primal_id: String,
        /// Health status (healthy, degraded, unhealthy)
        health: String,
        /// Timestamp
        timestamp: SystemTime,
    },

    /// API call between primals
    ApiCall {
        /// Source primal
        from: String,
        /// Target primal
        to: String,
        /// Capability invoked
        capability: String,
        /// Latency in milliseconds
        latency_ms: f64,
        /// HTTP status code
        status_code: u16,
        /// Timestamp
        timestamp: SystemTime,
    },

    /// Data transfer between primals
    DataTransfer {
        /// Source primal
        from: String,
        /// Target primal
        to: String,
        /// Bytes transferred
        bytes: u64,
        /// Data type
        data_type: String,
        /// Timestamp
        timestamp: SystemTime,
    },
}

/// Aggregated telemetry metrics
#[derive(Debug, Clone, Default)]
pub struct TelemetryMetrics {
    /// Total number of primals seen
    pub total_primals: usize,
    /// Currently active primals
    pub active_primals: usize,
    /// Total API calls
    pub total_api_calls: u64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// Total bytes transferred
    pub total_bytes: u64,
    /// Events per second
    pub events_per_second: f64,
    /// Per-primal metrics
    pub primal_metrics: HashMap<String, PrimalMetrics>,
}

/// Per-primal aggregated metrics
#[derive(Debug, Clone, Default)]
pub struct PrimalMetrics {
    /// API calls made by this primal
    pub calls_made: u64,
    /// API calls received by this primal
    pub calls_received: u64,
    /// Average latency for this primal's calls
    pub avg_latency_ms: f64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
}

/// Trait for telemetry subscribers
pub trait TelemetrySubscriber: Send + Sync {
    /// Called when a new event is received
    fn on_event(&mut self, event: &TelemetryEvent);

    /// Called when metrics are updated (every aggregation interval)
    fn on_metrics_update(&mut self, metrics: &TelemetryMetrics) {
        let _ = metrics; // Default: no-op
    }
}

/// Telemetry collector and aggregator
pub struct TelemetryCollector {
    /// Event buffer
    buffer: Arc<RwLock<VecDeque<TelemetryEvent>>>,
    /// Aggregated metrics
    metrics: Arc<RwLock<TelemetryMetrics>>,
    /// Subscribers
    subscribers: Arc<RwLock<Vec<Box<dyn TelemetrySubscriber>>>>,
    /// Maximum buffer size
    max_buffer_size: usize,
    /// Aggregation interval
    aggregation_interval: Duration,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    ///
    /// # Arguments
    ///
    /// * `max_buffer_size` - Maximum number of events to buffer
    /// * `aggregation_interval` - How often to compute metrics
    #[must_use]
    pub fn new(max_buffer_size: usize, aggregation_interval: Duration) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_buffer_size))),
            metrics: Arc::new(RwLock::new(TelemetryMetrics::default())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            max_buffer_size,
            aggregation_interval,
        }
    }

    /// Add an event to the telemetry stream
    ///
    /// Events are buffered and distributed to subscribers.
    pub fn push_event(&self, event: &TelemetryEvent) {
        {
            let Ok(mut buffer) = self.buffer.write() else {
                tracing::error!("Telemetry buffer lock poisoned");
                return;
            };
            buffer.push_back(event.clone());

            // Trim buffer if too large
            if buffer.len() > self.max_buffer_size {
                buffer.pop_front();
            }
        }

        // Notify subscribers
        {
            let Ok(mut subscribers) = self.subscribers.write() else {
                tracing::error!("Telemetry subscribers lock poisoned");
                return;
            };
            for subscriber in subscribers.iter_mut() {
                subscriber.on_event(event);
            }
        }

        self.update_metrics(event);
    }

    /// Add a subscriber to the telemetry stream
    pub fn add_subscriber(&self, subscriber: Box<dyn TelemetrySubscriber>) {
        let mut subscribers = self
            .subscribers
            .write()
            .expect("SAFETY: Telemetry subscribers lock poisoned - indicates panic in subscriber");
        subscribers.push(subscriber);
    }

    /// Get current metrics snapshot
    #[must_use]
    pub fn get_metrics(&self) -> TelemetryMetrics {
        self.metrics
            .read()
            .expect("SAFETY: Telemetry metrics lock poisoned - indicates panic in metrics update")
            .clone()
    }

    /// Get recent events from buffer
    #[must_use]
    pub fn get_recent_events(&self, count: usize) -> Vec<TelemetryEvent> {
        let Ok(buffer) = self.buffer.read() else {
            tracing::error!("Telemetry buffer lock poisoned");
            return Vec::new();
        };
        buffer.iter().rev().take(count).cloned().collect()
    }

    /// Update aggregated metrics based on event
    fn update_metrics(&self, event: &TelemetryEvent) {
        let Ok(mut metrics) = self.metrics.write() else {
            tracing::error!("Telemetry metrics lock poisoned");
            return;
        };

        match event {
            TelemetryEvent::PrimalDiscovered { primal_id, .. } => {
                metrics.total_primals += 1;
                metrics.active_primals += 1;
                metrics.primal_metrics.entry(primal_id.clone()).or_default();
            }

            TelemetryEvent::PrimalDisappeared { primal_id, .. } => {
                if metrics.active_primals > 0 {
                    metrics.active_primals -= 1;
                }
                // Keep metrics but mark as inactive
                if let Some(pm) = metrics.primal_metrics.get_mut(primal_id) {
                    let _ = pm; // Keep the entry
                }
            }

            TelemetryEvent::ApiCall {
                from,
                to,
                latency_ms,
                ..
            } => {
                metrics.total_api_calls += 1;

                // Update average latency
                let total = metrics.total_api_calls as f64;
                let prev_avg = metrics.avg_latency_ms;
                metrics.avg_latency_ms = (prev_avg * (total - 1.0) + latency_ms) / total;

                // Update per-primal metrics
                if let Some(pm) = metrics.primal_metrics.get_mut(from) {
                    pm.calls_made += 1;
                    let pm_total = pm.calls_made as f64;
                    pm.avg_latency_ms =
                        (pm.avg_latency_ms * (pm_total - 1.0) + latency_ms) / pm_total;
                }
                if let Some(pm) = metrics.primal_metrics.get_mut(to) {
                    pm.calls_received += 1;
                }
            }

            TelemetryEvent::DataTransfer {
                from, to, bytes, ..
            } => {
                metrics.total_bytes += bytes;

                if let Some(pm) = metrics.primal_metrics.get_mut(from) {
                    pm.bytes_sent += bytes;
                }
                if let Some(pm) = metrics.primal_metrics.get_mut(to) {
                    pm.bytes_received += bytes;
                }
            }

            TelemetryEvent::HealthUpdate { .. } => {
                // Health updates don't affect aggregate metrics
            }
        }
    }

    /// Clear all buffered events
    pub fn clear(&self) {
        let Ok(mut buffer) = self.buffer.write() else {
            tracing::error!("Telemetry buffer lock poisoned");
            return;
        };
        buffer.clear();
    }

    /// Get aggregation interval
    #[must_use]
    pub fn aggregation_interval(&self) -> Duration {
        self.aggregation_interval
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(10_000, Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSubscriber {
        events_received: Arc<RwLock<usize>>,
    }

    impl TelemetrySubscriber for TestSubscriber {
        fn on_event(&mut self, _event: &TelemetryEvent) {
            *self.events_received.write().expect("SAFETY: Lock poisoned") += 1;
        }
    }

    #[test]
    fn test_telemetry_collector_creation() {
        let collector = TelemetryCollector::new(1000, Duration::from_secs(1));
        assert_eq!(collector.get_metrics().total_primals, 0);
    }

    #[test]
    fn test_push_event() {
        let collector = TelemetryCollector::default();

        let event = TelemetryEvent::PrimalDiscovered {
            primal_id: "test-1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["cap1".to_string()],
            endpoint: "http://test:8080".to_string(),
            timestamp: SystemTime::now(),
        };

        collector.push_event(&event);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_primals, 1);
        assert_eq!(metrics.active_primals, 1);
    }

    #[test]
    fn test_subscriber_notification() {
        let collector = TelemetryCollector::default();
        let events_received = Arc::new(RwLock::new(0));

        let subscriber = Box::new(TestSubscriber {
            events_received: events_received.clone(),
        });

        collector.add_subscriber(subscriber);

        let event = TelemetryEvent::PrimalDiscovered {
            primal_id: "test-1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://test:8080".to_string(),
            timestamp: SystemTime::now(),
        };

        collector.push_event(&event);

        assert_eq!(*events_received.read().expect("SAFETY: Lock poisoned"), 1);
    }

    #[test]
    fn test_api_call_metrics() {
        let collector = TelemetryCollector::default();

        // Add two primals
        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "primal-a".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://a:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "primal-b".to_string(),
            primal_type: "storage".to_string(),
            capabilities: vec![],
            endpoint: "http://b:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        // Add API call
        collector.push_event(&TelemetryEvent::ApiCall {
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            capability: "store".to_string(),
            latency_ms: 10.0,
            status_code: 200,
            timestamp: SystemTime::now(),
        });

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_api_calls, 1);
        assert!((metrics.avg_latency_ms - 10.0).abs() < f64::EPSILON);

        let primal_a_metrics = metrics.primal_metrics.get("primal-a").unwrap();
        assert_eq!(primal_a_metrics.calls_made, 1);
        assert_eq!(primal_a_metrics.calls_received, 0);

        let primal_b_metrics = metrics.primal_metrics.get("primal-b").unwrap();
        assert_eq!(primal_b_metrics.calls_made, 0);
        assert_eq!(primal_b_metrics.calls_received, 1);
    }

    #[test]
    fn test_data_transfer_metrics() {
        let collector = TelemetryCollector::default();

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "primal-a".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://a:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        collector.push_event(&TelemetryEvent::DataTransfer {
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            bytes: 1024,
            data_type: "json".to_string(),
            timestamp: SystemTime::now(),
        });

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_bytes, 1024);

        let primal_a_metrics = metrics.primal_metrics.get("primal-a").unwrap();
        assert_eq!(primal_a_metrics.bytes_sent, 1024);
    }

    #[test]
    fn test_primal_disappeared() {
        let collector = TelemetryCollector::default();

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "test-1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://test:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        collector.push_event(&TelemetryEvent::PrimalDisappeared {
            primal_id: "test-1".to_string(),
            timestamp: SystemTime::now(),
        });

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_primals, 1);
        assert_eq!(metrics.active_primals, 0);
    }

    #[test]
    fn test_get_recent_events() {
        let collector = TelemetryCollector::default();

        for i in 0..5 {
            collector.push_event(&TelemetryEvent::PrimalDiscovered {
                primal_id: format!("primal-{i}"),
                primal_type: "compute".to_string(),
                capabilities: vec![],
                endpoint: format!("http://test:{}", 8080 + i),
                timestamp: SystemTime::now(),
            });
        }

        let recent = collector.get_recent_events(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_buffer_overflow() {
        let collector = TelemetryCollector::new(3, Duration::from_secs(1));

        for i in 0..5 {
            collector.push_event(&TelemetryEvent::PrimalDiscovered {
                primal_id: format!("primal-{i}"),
                primal_type: "compute".to_string(),
                capabilities: vec![],
                endpoint: format!("http://test:{}", 8080 + i),
                timestamp: SystemTime::now(),
            });
        }

        let recent = collector.get_recent_events(10);
        assert_eq!(recent.len(), 3, "Buffer should be limited to max size");
    }

    #[test]
    fn test_clear() {
        let collector = TelemetryCollector::default();

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "test-1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://test:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        collector.clear();

        let recent = collector.get_recent_events(10);
        assert_eq!(recent.len(), 0);
    }
}
