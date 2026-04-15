// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use super::test_fixtures::{MetricsUpdateSubscriber, TestSubscriber, event_primal_id};
use super::*;
use crate::types::TelemetryEvent;

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
