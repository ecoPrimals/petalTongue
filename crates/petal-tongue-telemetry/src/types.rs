// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryEvent {
    PrimalDiscovered {
        primal_id: String,
        primal_type: String,
        capabilities: Vec<String>,
        endpoint: String,
        timestamp: SystemTime,
    },

    PrimalDisappeared {
        primal_id: String,
        timestamp: SystemTime,
    },

    HealthUpdate {
        primal_id: String,
        health: String,
        timestamp: SystemTime,
    },

    ApiCall {
        from: String,
        to: String,
        capability: String,
        latency_ms: f64,
        status_code: u16,
        timestamp: SystemTime,
    },

    DataTransfer {
        from: String,
        to: String,
        bytes: u64,
        data_type: String,
        timestamp: SystemTime,
    },
}

#[derive(Debug, Clone, Default)]
pub struct TelemetryMetrics {
    pub total_primals: usize,
    pub active_primals: usize,
    pub total_api_calls: u64,
    pub avg_latency_ms: f64,
    pub total_bytes: u64,
    pub events_per_second: f64,
    pub primal_metrics: HashMap<String, PrimalMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct PrimalMetrics {
    pub calls_made: u64,
    pub calls_received: u64,
    pub avg_latency_ms: f64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

pub trait TelemetrySubscriber: Send + Sync {
    fn on_event(&mut self, event: &TelemetryEvent);

    fn on_metrics_update(&mut self, metrics: &TelemetryMetrics) {
        let _ = metrics;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_metrics_default() {
        let pm = PrimalMetrics::default();
        assert_eq!(pm.calls_made, 0);
        assert_eq!(pm.calls_received, 0);
        assert!((pm.avg_latency_ms - 0.0).abs() < f64::EPSILON);
        assert_eq!(pm.bytes_sent, 0);
        assert_eq!(pm.bytes_received, 0);
    }

    #[test]
    fn test_telemetry_metrics_default() {
        let m = TelemetryMetrics::default();
        assert_eq!(m.total_primals, 0);
        assert_eq!(m.active_primals, 0);
        assert_eq!(m.total_api_calls, 0);
        assert_eq!(m.total_bytes, 0);
    }

    #[test]
    fn test_telemetry_event_variants() {
        let _ = TelemetryEvent::PrimalDiscovered {
            primal_id: String::new(),
            primal_type: String::new(),
            capabilities: vec![],
            endpoint: String::new(),
            timestamp: SystemTime::now(),
        };
        let _ = TelemetryEvent::ApiCall {
            from: String::new(),
            to: String::new(),
            capability: String::new(),
            latency_ms: 0.0,
            status_code: 200,
            timestamp: SystemTime::now(),
        };
        let _ = TelemetryEvent::DataTransfer {
            from: String::new(),
            to: String::new(),
            bytes: 0,
            data_type: String::new(),
            timestamp: SystemTime::now(),
        };
    }

    #[test]
    fn test_events_per_second_default() {
        let m = TelemetryMetrics::default();
        assert!((m.events_per_second - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_event_primal_disappeared() {
        let _ = TelemetryEvent::PrimalDisappeared {
            primal_id: "p1".to_string(),
            timestamp: SystemTime::now(),
        };
    }

    #[test]
    fn test_telemetry_event_health_update() {
        let _ = TelemetryEvent::HealthUpdate {
            primal_id: "p1".to_string(),
            health: "healthy".to_string(),
            timestamp: SystemTime::now(),
        };
    }

    #[test]
    fn test_primal_metrics_construction() {
        let pm = PrimalMetrics {
            calls_made: 10,
            calls_received: 5,
            avg_latency_ms: 12.5,
            bytes_sent: 1000,
            bytes_received: 500,
        };
        assert_eq!(pm.calls_made, 10);
        assert_eq!(pm.calls_received, 5);
        assert!((pm.avg_latency_ms - 12.5).abs() < f64::EPSILON);
        assert_eq!(pm.bytes_sent, 1000);
        assert_eq!(pm.bytes_received, 500);
    }

    #[test]
    fn test_telemetry_metrics_construction() {
        let mut primal_metrics = HashMap::new();
        primal_metrics.insert(
            "p1".to_string(),
            PrimalMetrics {
                calls_made: 1,
                calls_received: 2,
                avg_latency_ms: 5.0,
                bytes_sent: 100,
                bytes_received: 200,
            },
        );
        let m = TelemetryMetrics {
            total_primals: 1,
            active_primals: 1,
            total_api_calls: 10,
            avg_latency_ms: 8.0,
            total_bytes: 1000,
            events_per_second: 2.5,
            primal_metrics,
        };
        assert_eq!(m.total_primals, 1);
        assert_eq!(m.active_primals, 1);
        assert_eq!(m.total_api_calls, 10);
        assert!((m.avg_latency_ms - 8.0).abs() < f64::EPSILON);
        assert_eq!(m.total_bytes, 1000);
        assert!((m.events_per_second - 2.5).abs() < f64::EPSILON);
        assert_eq!(m.primal_metrics.len(), 1);
    }

    #[test]
    fn test_telemetry_subscriber_on_metrics_update() {
        struct TestSubscriber;
        impl TelemetrySubscriber for TestSubscriber {
            fn on_event(&mut self, _event: &TelemetryEvent) {}
        }
        let mut sub = TestSubscriber;
        let metrics = TelemetryMetrics::default();
        sub.on_metrics_update(&metrics);
    }

    #[test]
    fn test_telemetry_event_debug() {
        let e = TelemetryEvent::PrimalDiscovered {
            primal_id: "p1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["cap1".to_string()],
            endpoint: "http://localhost".to_string(),
            timestamp: SystemTime::UNIX_EPOCH,
        };
        let s = format!("{e:?}");
        assert!(s.contains("PrimalDiscovered"));
        assert!(s.contains("p1"));
    }

    #[test]
    fn test_primal_metrics_clone() {
        let pm = PrimalMetrics {
            calls_made: 1,
            calls_received: 2,
            avg_latency_ms: 3.0,
            bytes_sent: 4,
            bytes_received: 5,
        };
        let cloned = pm.clone();
        assert_eq!(pm.calls_made, cloned.calls_made);
        assert_eq!(pm.bytes_received, cloned.bytes_received);
    }

    #[test]
    fn test_telemetry_metrics_clone() {
        let m = TelemetryMetrics {
            total_primals: 2,
            active_primals: 1,
            total_api_calls: 100,
            avg_latency_ms: 10.0,
            total_bytes: 5000,
            events_per_second: 5.0,
            primal_metrics: HashMap::new(),
        };
        let cloned = m.clone();
        assert_eq!(m.total_primals, cloned.total_primals);
        assert_eq!(m.events_per_second, cloned.events_per_second);
    }
}
