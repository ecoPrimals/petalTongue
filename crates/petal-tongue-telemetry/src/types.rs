// SPDX-License-Identifier: AGPL-3.0-only

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
}
