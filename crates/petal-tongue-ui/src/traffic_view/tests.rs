// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traffic view tests

use super::{
    ColorScheme, TrafficFlow, TrafficIntent, TrafficMetrics, TrafficView, bezier_control_points,
    calculate_flow_color, calculate_flow_width, prepare_flow_detail, primal_lane_layout,
};

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
        color: [0, 255, 0, 255],
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
            color: [0, 255, 0, 255],
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

    let volume_color = calculate_flow_color(&metrics, ColorScheme::Volume);
    let latency_color = calculate_flow_color(&metrics, ColorScheme::Latency);
    let error_color = calculate_flow_color(&metrics, ColorScheme::ErrorRate);

    assert_ne!(volume_color, latency_color);
    assert_ne!(volume_color, error_color);
    assert_ne!(latency_color, error_color);
}

#[test]
fn test_flow_color_volume_bounds() {
    let low = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    let high = calculate_flow_color(
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
    let low = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 0.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    let high = calculate_flow_color(
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
    let low = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.0,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    let high = calculate_flow_color(
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
        color: [0, 255, 0, 255],
    });

    view.add_flow(TrafficFlow {
        from: "b".to_string(),
        to: "c".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 10000,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });

    let max_vol = view
        .flows()
        .iter()
        .map(|f| f.metrics.bytes_per_second)
        .max()
        .unwrap_or(1);
    let width1 = calculate_flow_width(&view.flows()[0].metrics, max_vol, 2.0, 40.0);
    let width2 = calculate_flow_width(&view.flows()[1].metrics, max_vol, 2.0, 40.0);

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

#[test]
fn test_default_traffic_view() {
    let view = TrafficView::default();
    assert_eq!(view.flow_count(), 0);
    assert_eq!(view.primal_count(), 0);
}

#[test]
fn test_traffic_metrics_default() {
    let m = TrafficMetrics::default();
    assert_eq!(m.bytes_per_second, 0);
    assert!((m.requests_per_second - 0.0).abs() < f64::EPSILON);
    assert!((m.avg_latency_ms - 0.0).abs() < f64::EPSILON);
    assert!((m.error_rate - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_traffic_flow_clone() {
    let flow = TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 5000,
            requests_per_second: 25.0,
            avg_latency_ms: 12.5,
            error_rate: 0.02,
        },
        color: [255, 0, 0, 255],
    };
    let cloned = flow.clone();
    assert_eq!(cloned.from, "a");
    assert_eq!(cloned.to, "b");
    assert_eq!(cloned.metrics.bytes_per_second, 5000);
}

#[test]
fn test_traffic_flow_debug() {
    let flow = TrafficFlow {
        from: "src".to_string(),
        to: "dst".to_string(),
        metrics: TrafficMetrics::default(),
        color: [0, 255, 0, 255],
    };
    let debug = format!("{flow:?}");
    assert!(debug.contains("src"));
    assert!(debug.contains("dst"));
}

#[test]
fn test_color_scheme_variants() {
    assert_eq!(ColorScheme::Volume, ColorScheme::Volume);
    assert_ne!(ColorScheme::Volume, ColorScheme::Latency);
    assert_ne!(ColorScheme::Latency, ColorScheme::ErrorRate);
}

#[test]
fn test_flow_width_single_flow() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 5000,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });

    let max_vol = view
        .flows()
        .iter()
        .map(|f| f.metrics.bytes_per_second)
        .max()
        .unwrap_or(1);
    let width = calculate_flow_width(&view.flows()[0].metrics, max_vol, 2.0, 40.0);
    assert!(width >= 2.0);
    assert!(width <= 40.0);
}

#[test]
fn test_update_from_topology_clears_old() {
    use petal_tongue_core::{PrimalId, TopologyEdge};

    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "old".to_string(),
        to: "data".to_string(),
        metrics: TrafficMetrics::default(),
        color: [255, 0, 0, 255],
    });
    assert_eq!(view.flow_count(), 1);

    let edges = vec![TopologyEdge {
        from: PrimalId::from("new_a"),
        to: PrimalId::from("new_b"),
        edge_type: "connection".to_string(),
        label: None,
        capability: None,
        metrics: None,
    }];
    view.update_from_topology(&edges);
    assert_eq!(view.flow_count(), 1);
    assert_eq!(view.flows()[0].from, "new_a");
}

// === Additional coverage: constructor, edge cases, intents, data transformation ===

#[test]
fn test_calculate_flow_width_max_volume_zero() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 0, 2.0, 40.0);
    assert!((w - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_calculate_flow_width_max_volume_one() {
    let m = TrafficMetrics {
        bytes_per_second: 1,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 1, 2.0, 40.0);
    assert!((w - 40.0).abs() < f32::EPSILON);
}

#[test]
fn test_calculate_flow_color_latency_clamped() {
    let m = TrafficMetrics {
        avg_latency_ms: 200.0,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Latency);
    assert_eq!(c[0], 255);
}

#[test]
fn test_calculate_flow_color_error_rate_clamped() {
    let m = TrafficMetrics {
        error_rate: 0.5,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::ErrorRate);
    assert!(c[0] > 200);
}

#[test]
fn test_apply_intents_set_color_scheme_updates_flow_colors() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 5000,
            avg_latency_ms: 50.0,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    let initial_color = view.flows()[0].color;
    view.apply_intents(&[TrafficIntent::SetColorScheme(ColorScheme::Latency)]);
    assert_eq!(view.color_scheme(), ColorScheme::Latency);
    assert_ne!(view.flows()[0].color, initial_color);
}

#[test]
fn test_selected_flow_accessor() {
    let mut view = TrafficView::new();
    assert!(view.selected_flow().is_none());
    view.apply_intents(&[TrafficIntent::SelectFlow {
        from: "src".to_string(),
        to: "dst".to_string(),
    }]);
    let sel = view.selected_flow();
    assert!(sel.is_some());
    let (from, to) = sel.unwrap();
    assert_eq!(from, "src");
    assert_eq!(to, "dst");
}

#[test]
fn test_traffic_intent_equality() {
    assert_eq!(
        TrafficIntent::SetColorScheme(ColorScheme::Volume),
        TrafficIntent::SetColorScheme(ColorScheme::Volume)
    );
    assert_ne!(
        TrafficIntent::SetColorScheme(ColorScheme::Volume),
        TrafficIntent::SetColorScheme(ColorScheme::Latency)
    );
    assert_eq!(
        TrafficIntent::SelectFlow {
            from: "a".to_string(),
            to: "b".to_string(),
        },
        TrafficIntent::SelectFlow {
            from: "a".to_string(),
            to: "b".to_string(),
        }
    );
}

#[test]
fn test_prepare_flow_detail() {
    let flow = TrafficFlow {
        from: "x".to_string(),
        to: "y".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 12345,
            requests_per_second: 99.9,
            avg_latency_ms: 7.77,
            error_rate: 0.123,
        },
        color: [0, 0, 255, 255],
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.from, "x");
    assert_eq!(d.to, "y");
    assert_eq!(d.volume_label, "12345 B/s");
    assert_eq!(d.requests_label, "99.9 req/s");
    assert_eq!(d.latency_label, "7.77 ms");
    assert_eq!(d.error_rate_label, "12.30%");
}

#[test]
fn test_set_primals_clears_old() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo};

    let mut view = TrafficView::new();
    view.set_primals(vec![PrimalInfo {
        id: PrimalId::from("old1"),
        name: "Old 1".to_string(),
        primal_type: "Test".to_string(),
        endpoint: "http://old".to_string(),
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
    }]);
    assert_eq!(view.primal_count(), 1);

    view.set_primals(vec![PrimalInfo {
        id: PrimalId::from("new1"),
        name: "New 1".to_string(),
        primal_type: "Test".to_string(),
        endpoint: "http://new".to_string(),
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
    }]);
    assert_eq!(view.primal_count(), 1);
}

#[test]
fn test_apply_intents_combined() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics::default(),
        color: [0, 255, 0, 255],
    });
    view.apply_intents(&[
        TrafficIntent::SelectFlow {
            from: "a".to_string(),
            to: "b".to_string(),
        },
        TrafficIntent::SetColorScheme(ColorScheme::ErrorRate),
        TrafficIntent::ToggleMetrics,
    ]);
    assert!(view.selected_flow().is_some());
    assert_eq!(view.color_scheme(), ColorScheme::ErrorRate);
    assert!(!view.show_metrics());
}

#[test]
fn test_update_from_topology_empty_edges() {
    use petal_tongue_core::TopologyEdge;

    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "x".to_string(),
        to: "y".to_string(),
        metrics: TrafficMetrics::default(),
        color: [255, 0, 0, 255],
    });
    view.update_from_topology(&[] as &[TopologyEdge]);
    assert_eq!(view.flow_count(), 0);
}

#[test]
fn test_clear_also_clears_selection() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics::default(),
        color: [0, 255, 0, 255],
    });
    view.apply_intents(&[TrafficIntent::SelectFlow {
        from: "a".to_string(),
        to: "b".to_string(),
    }]);
    assert!(view.selected_flow().is_some());
    view.clear();
    assert_eq!(view.flow_count(), 0);
    assert!(view.selected_flow().is_none());
}

#[test]
fn test_primal_lane_layout_many() {
    let layout = primal_lane_layout(10, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 10);
    let node_height = (600.0 - 40.0) / 10.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 20.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 80.0).abs() < f32::EPSILON);
        assert!((right_x - 720.0).abs() < f32::EPSILON);
    }
}

#[test]
fn test_bezier_control_points_vertical() {
    let (ctrl1, ctrl2) = bezier_control_points(100.0, 0.0, 100.0, 200.0);
    let dx = 0.0_f32;
    let _offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - 100.0).abs() < f32::EPSILON);
    assert!((ctrl1[1] - 0.0).abs() < f32::EPSILON);
    assert!((ctrl2[0] - 100.0).abs() < f32::EPSILON);
    assert!((ctrl2[1] - 200.0).abs() < f32::EPSILON);
}
