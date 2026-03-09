// SPDX-License-Identifier: AGPL-3.0-only
//! Traffic view tests

use super::TrafficView;
use super::types::{ColorScheme, TrafficFlow, TrafficMetrics};
use egui::Color32;

#[test]
fn test_traffic_view_creation() {
    let view = TrafficView::new();
    assert_eq!(view.flow_count(), 0);
    assert!(view.show_metrics());
}

#[test]
fn test_add_flow() {
    let mut view = TrafficView::new();

    let flow = TrafficFlow {
        from: "primal1".to_string(),
        to: "primal2".to_string(),
        metrics: TrafficMetrics::default(),
        color: Color32::GREEN,
    };

    view.add_flow(flow);
    assert_eq!(view.flow_count(), 1);
}

#[test]
fn test_clear_flows() {
    let mut view = TrafficView::new();

    for i in 0..5 {
        view.add_flow(TrafficFlow {
            from: format!("primal{i}"),
            to: format!("primal{}", i + 1),
            metrics: TrafficMetrics::default(),
            color: Color32::GREEN,
        });
    }

    assert_eq!(view.flow_count(), 5);

    view.clear();
    assert_eq!(view.flow_count(), 0);
    assert!(view.flows().is_empty());
}

#[test]
fn test_color_schemes() {
    let metrics = TrafficMetrics {
        bytes_per_second: 1000,
        requests_per_second: 10.0,
        avg_latency_ms: 50.0,
        error_rate: 0.05,
    };

    let volume_color = TrafficView::calculate_flow_color(&metrics, ColorScheme::Volume);
    let latency_color = TrafficView::calculate_flow_color(&metrics, ColorScheme::Latency);
    let error_color = TrafficView::calculate_flow_color(&metrics, ColorScheme::ErrorRate);

    assert_ne!(volume_color, latency_color);
    assert_ne!(volume_color, error_color);
    assert_ne!(latency_color, error_color);
}

#[test]
fn test_flow_color_volume_bounds() {
    let low = TrafficView::calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    let high = TrafficView::calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 200_000,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert_ne!(low, high);
}

#[test]
fn test_flow_color_latency_bounds() {
    let low = TrafficView::calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 0.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    let high = TrafficView::calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 150.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert_ne!(low, high);
}

#[test]
fn test_flow_color_error_bounds() {
    let low = TrafficView::calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.0,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    let high = TrafficView::calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.5,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert_ne!(low, high);
}

#[test]
fn test_flow_width_calculation() {
    let mut view = TrafficView::new();

    // Add flows with different volumes
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 1000,
            ..Default::default()
        },
        color: Color32::GREEN,
    });

    view.add_flow(TrafficFlow {
        from: "b".to_string(),
        to: "c".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 10000,
            ..Default::default()
        },
        color: Color32::GREEN,
    });

    let width1 = view.calculate_flow_width(&view.flows()[0].metrics);
    let width2 = view.calculate_flow_width(&view.flows()[1].metrics);

    // Higher volume should have wider flow
    assert!(width2 > width1);
}

#[test]
fn test_update_from_topology() {
    use petal_tongue_core::{PrimalId, TopologyEdge};

    let mut view = TrafficView::new();
    let edges = vec![
        TopologyEdge {
            from: PrimalId::from("a"),
            to: PrimalId::from("b"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("b"),
            to: PrimalId::from("c"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
    ];
    view.update_from_topology(&edges);
    assert_eq!(view.flow_count(), 2);
}

#[test]
fn test_set_primals() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo};

    let mut view = TrafficView::new();
    let primals = vec![
        PrimalInfo {
            id: PrimalId::from("p1"),
            name: "Primal 1".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://test".to_string(),
            capabilities: vec![],
            health: PrimalHealthStatus::Healthy,
            last_seen: 0,
            endpoints: None,
            metadata: None,
            properties: Default::default(),
            #[expect(deprecated)]
            trust_level: None,
            #[expect(deprecated)]
            family_id: None,
        },
        PrimalInfo {
            id: PrimalId::from("p2"),
            name: "Primal 2".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://test".to_string(),
            capabilities: vec![],
            health: PrimalHealthStatus::Healthy,
            last_seen: 0,
            endpoints: None,
            metadata: None,
            properties: Default::default(),
            #[expect(deprecated)]
            trust_level: None,
            #[expect(deprecated)]
            family_id: None,
        },
    ];
    view.set_primals(primals);
    assert_eq!(view.primal_count(), 2);
}
