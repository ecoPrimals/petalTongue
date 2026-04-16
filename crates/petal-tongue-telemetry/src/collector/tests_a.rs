// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use super::*;
use crate::TelemetrySubscriberImpl;
use crate::subscriber_impl::TestSubscriber;
use crate::types::TelemetryEvent;

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

    collector.add_subscriber(TelemetrySubscriberImpl::Test(TestSubscriber {
        events_received: events_received.clone(),
    }));

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
