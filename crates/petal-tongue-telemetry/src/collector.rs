// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::types::{TelemetryEvent, TelemetryMetrics, TelemetrySubscriber};

pub struct TelemetryCollector {
    buffer: Arc<RwLock<VecDeque<TelemetryEvent>>>,
    metrics: Arc<RwLock<TelemetryMetrics>>,
    subscribers: Arc<RwLock<Vec<Box<dyn TelemetrySubscriber>>>>,
    max_buffer_size: usize,
    aggregation_interval: Duration,
}

impl TelemetryCollector {
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

    pub fn push_event(&self, event: &TelemetryEvent) {
        {
            let Ok(mut buffer) = self.buffer.write() else {
                tracing::error!("Telemetry buffer lock poisoned");
                return;
            };
            buffer.push_back(event.clone());

            if buffer.len() > self.max_buffer_size {
                buffer.pop_front();
            }
        }

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

    /// Add a telemetry subscriber to receive future events.
    ///
    /// # Panics
    ///
    /// Panics if the subscribers lock is poisoned (e.g. from a panic in a subscriber callback).
    pub fn add_subscriber(&self, subscriber: Box<dyn TelemetrySubscriber>) {
        let mut subscribers = self
            .subscribers
            .write()
            .expect("SAFETY: Telemetry subscribers lock poisoned - indicates panic in subscriber");
        subscribers.push(subscriber);
    }

    /// Get a snapshot of current telemetry metrics.
    ///
    /// # Panics
    ///
    /// Panics if the metrics lock is poisoned (e.g. from a panic during metrics update).
    #[must_use]
    pub fn get_metrics(&self) -> TelemetryMetrics {
        self.metrics
            .read()
            .expect("SAFETY: Telemetry metrics lock poisoned - indicates panic in metrics update")
            .clone()
    }

    #[must_use]
    pub fn get_recent_events(&self, count: usize) -> Vec<TelemetryEvent> {
        let Ok(buffer) = self.buffer.read() else {
            tracing::error!("Telemetry buffer lock poisoned");
            return Vec::new();
        };
        buffer.iter().rev().take(count).cloned().collect()
    }

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
                if let Some(pm) = metrics.primal_metrics.get_mut(primal_id) {
                    let _ = pm;
                }
            }

            TelemetryEvent::ApiCall {
                from,
                to,
                latency_ms,
                ..
            } => {
                metrics.total_api_calls += 1;

                // f64 mantissa is 52 bits; precision loss acceptable for running-average math
                #[allow(clippy::cast_precision_loss)]
                let total = metrics.total_api_calls as f64;
                let prev_avg = metrics.avg_latency_ms;
                metrics.avg_latency_ms = (prev_avg * (total - 1.0) + latency_ms) / total;

                if let Some(pm) = metrics.primal_metrics.get_mut(from) {
                    pm.calls_made += 1;
                    // f64 mantissa is 52 bits; precision loss acceptable for per-primal averages
                    #[allow(clippy::cast_precision_loss)]
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

            TelemetryEvent::HealthUpdate { .. } => {}
        }
    }

    pub fn clear(&self) {
        let Ok(mut buffer) = self.buffer.write() else {
            tracing::error!("Telemetry buffer lock poisoned");
            return;
        };
        buffer.clear();
    }

    #[must_use]
    pub const fn aggregation_interval(&self) -> Duration {
        self.aggregation_interval
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(
            petal_tongue_core::constants::default_telemetry_buffer(),
            Duration::from_secs(1),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

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

        let metrics_a = metrics.primal_metrics.get("primal-a").unwrap();
        assert_eq!(metrics_a.calls_made, 1);
        assert_eq!(metrics_a.calls_received, 0);

        let metrics_b = metrics.primal_metrics.get("primal-b").unwrap();
        assert_eq!(metrics_b.calls_made, 0);
        assert_eq!(metrics_b.calls_received, 1);
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

        let metrics_a = metrics.primal_metrics.get("primal-a").unwrap();
        assert_eq!(metrics_a.bytes_sent, 1024);
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

    #[test]
    fn test_aggregation_interval() {
        let interval = Duration::from_secs(5);
        let collector = TelemetryCollector::new(100, interval);
        assert_eq!(collector.aggregation_interval(), interval);
    }

    #[test]
    fn test_default_collector_config() {
        let collector = TelemetryCollector::default();
        assert_eq!(collector.aggregation_interval(), Duration::from_secs(1));
        assert_eq!(collector.get_metrics().total_primals, 0);
    }

    #[test]
    fn test_health_update_does_not_affect_aggregates() {
        let collector = TelemetryCollector::default();
        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "p1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://p1:8080".to_string(),
            timestamp: SystemTime::now(),
        });
        let before = collector.get_metrics();

        collector.push_event(&TelemetryEvent::HealthUpdate {
            primal_id: "p1".to_string(),
            health: "degraded".to_string(),
            timestamp: SystemTime::now(),
        });
        let after = collector.get_metrics();

        assert_eq!(before.total_primals, after.total_primals);
        assert_eq!(before.active_primals, after.active_primals);
    }

    #[test]
    fn test_metrics_export_format() {
        let collector = TelemetryCollector::default();
        collector.push_event(&TelemetryEvent::ApiCall {
            from: "a".to_string(),
            to: "b".to_string(),
            capability: "call".to_string(),
            latency_ms: 42.0,
            status_code: 200,
            timestamp: SystemTime::now(),
        });
        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_api_calls, 1);
        assert!((metrics.avg_latency_ms - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_multiple_subscribers() {
        let collector = TelemetryCollector::default();
        let count1 = Arc::new(RwLock::new(0usize));
        let count2 = Arc::new(RwLock::new(0usize));

        collector.add_subscriber(Box::new(TestSubscriber {
            events_received: count1.clone(),
        }));
        collector.add_subscriber(Box::new(TestSubscriber {
            events_received: count2.clone(),
        }));

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "test".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://test:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        assert_eq!(*count1.read().expect("lock"), 1);
        assert_eq!(*count2.read().expect("lock"), 1);
    }

    fn event_primal_id(event: &TelemetryEvent) -> Option<&str> {
        match event {
            TelemetryEvent::PrimalDiscovered { primal_id, .. }
            | TelemetryEvent::PrimalDisappeared { primal_id, .. }
            | TelemetryEvent::HealthUpdate { primal_id, .. } => Some(primal_id),
            _ => None,
        }
    }

    #[test]
    fn test_api_call_latency_averaging() {
        let collector = TelemetryCollector::default();

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "a".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://a:8080".to_string(),
            timestamp: SystemTime::now(),
        });
        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "b".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://b:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        collector.push_event(&TelemetryEvent::ApiCall {
            from: "a".to_string(),
            to: "b".to_string(),
            capability: "call".to_string(),
            latency_ms: 10.0,
            status_code: 200,
            timestamp: SystemTime::now(),
        });
        collector.push_event(&TelemetryEvent::ApiCall {
            from: "a".to_string(),
            to: "b".to_string(),
            capability: "call".to_string(),
            latency_ms: 30.0,
            status_code: 200,
            timestamp: SystemTime::now(),
        });

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_api_calls, 2);
        let avg = metrics.avg_latency_ms;
        assert!(
            (10.0..=30.0).contains(&avg),
            "avg should be between 10 and 30, got {avg}"
        );
    }

    #[test]
    fn test_data_transfer_both_directions() {
        let collector = TelemetryCollector::default();

        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "a".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://a:8080".to_string(),
            timestamp: SystemTime::now(),
        });
        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "b".to_string(),
            primal_type: "storage".to_string(),
            capabilities: vec![],
            endpoint: "http://b:8080".to_string(),
            timestamp: SystemTime::now(),
        });

        collector.push_event(&TelemetryEvent::DataTransfer {
            from: "a".to_string(),
            to: "b".to_string(),
            bytes: 100,
            data_type: "json".to_string(),
            timestamp: SystemTime::now(),
        });
        collector.push_event(&TelemetryEvent::DataTransfer {
            from: "b".to_string(),
            to: "a".to_string(),
            bytes: 200,
            data_type: "json".to_string(),
            timestamp: SystemTime::now(),
        });

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_bytes, 300);
        let a_metrics = metrics.primal_metrics.get("a").expect("a should exist");
        assert_eq!(a_metrics.bytes_sent, 100);
        assert_eq!(a_metrics.bytes_received, 200);
        let b_metrics = metrics.primal_metrics.get("b").expect("b should exist");
        assert_eq!(b_metrics.bytes_sent, 200);
        assert_eq!(b_metrics.bytes_received, 100);
    }

    #[test]
    fn test_get_recent_events_empty() {
        let collector = TelemetryCollector::default();
        let recent = collector.get_recent_events(5);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_get_recent_events_request_more_than_buffer() {
        let collector = TelemetryCollector::default();
        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "p1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://p1:8080".to_string(),
            timestamp: SystemTime::now(),
        });
        let recent = collector.get_recent_events(100);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_recent_events_order() {
        let collector = TelemetryCollector::default();
        for i in 0..5 {
            collector.push_event(&TelemetryEvent::PrimalDiscovered {
                primal_id: format!("p{i}"),
                primal_type: "compute".to_string(),
                capabilities: vec![],
                endpoint: format!("http://p{i}:8080"),
                timestamp: SystemTime::now(),
            });
        }
        let recent = collector.get_recent_events(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(event_primal_id(&recent[0]), Some("p4"));
        assert_eq!(event_primal_id(&recent[2]), Some("p2"));
    }

    struct MetricsUpdateSubscriber {
        updates: Arc<RwLock<usize>>,
    }

    impl TelemetrySubscriber for MetricsUpdateSubscriber {
        fn on_event(&mut self, _event: &TelemetryEvent) {}

        fn on_metrics_update(&mut self, metrics: &TelemetryMetrics) {
            *self.updates.write().expect("lock") += 1;
            let _ = metrics;
        }
    }

    #[test]
    fn test_subscriber_on_metrics_update_override() {
        let collector = TelemetryCollector::default();
        let updates = Arc::new(RwLock::new(0usize));
        collector.add_subscriber(Box::new(MetricsUpdateSubscriber {
            updates: updates.clone(),
        }));
        collector.push_event(&TelemetryEvent::PrimalDiscovered {
            primal_id: "p1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![],
            endpoint: "http://p1:8080".to_string(),
            timestamp: SystemTime::now(),
        });
        assert_eq!(
            *updates.read().expect("lock"),
            0,
            "on_metrics_update not called on push"
        );
    }
}
