// SPDX-License-Identifier: AGPL-3.0-only
//! Traffic view tests

use super::{
    ColorScheme, TrafficFlow, TrafficIntent, TrafficMetrics, TrafficView, bezier_control_points,
    calculate_flow_color, calculate_flow_width, prepare_flow_detail, primal_lane_layout,
};
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
        color: Color32::RED,
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
        color: Color32::GREEN,
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
        color: Color32::GREEN,
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
        color: Color32::RED,
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
    assert_eq!(c.r(), 255);
}

#[test]
fn test_calculate_flow_color_error_rate_clamped() {
    let m = TrafficMetrics {
        error_rate: 0.5,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::ErrorRate);
    assert!(c.r() > 200);
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
        color: Color32::GREEN,
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
        color: Color32::BLUE,
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
        color: Color32::GREEN,
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
        color: Color32::RED,
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
        color: Color32::GREEN,
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

// === Tests from view.rs (pure functions, intents, headless render) ===

mod proptest_impl {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn bezier_control_points_symmetry(
            from_x in -500.0f32..500.0f32,
            from_y in -500.0f32..500.0f32,
            to_x in -500.0f32..500.0f32,
            to_y in -500.0f32..500.0f32,
        ) {
            let (ctrl1, ctrl2) = bezier_control_points(from_x, from_y, to_x, to_y);
            let dx = to_x - from_x;
            let offset = dx.abs() * 0.3;
            let expect_ctrl1_x = from_x + offset * dx.signum();
            let expect_ctrl2_x = to_x - offset * dx.signum();
            assert!((ctrl1[0] - expect_ctrl1_x).abs() < 0.001, "ctrl1.x");
            assert!((ctrl1[1] - from_y).abs() < 0.001, "ctrl1.y");
            assert!((ctrl2[0] - expect_ctrl2_x).abs() < 0.001, "ctrl2.x");
            assert!((ctrl2[1] - to_y).abs() < 0.001, "ctrl2.y");
        }
    }
}

#[test]
fn calculate_flow_color_volume_zero() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Volume);
    assert_eq!(c.r(), 0);
    assert_eq!(c.g(), 255);
}

#[test]
fn calculate_flow_color_volume_max() {
    let m = TrafficMetrics {
        bytes_per_second: 200_000,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Volume);
    assert_eq!(c.r(), 255);
    assert_eq!(c.g(), 0);
}

#[test]
fn calculate_flow_color_latency_fast() {
    let m = TrafficMetrics {
        avg_latency_ms: 0.0,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Latency);
    assert_eq!(c.r(), 0);
}

#[test]
fn calculate_flow_color_error_high() {
    let m = TrafficMetrics {
        error_rate: 0.5,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::ErrorRate);
    assert!(c.r() > 200);
}

#[test]
fn calculate_flow_width_min() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 10000, 2.0, 40.0);
    assert!((w - 2.0).abs() < f32::EPSILON);
}

#[test]
fn calculate_flow_width_max() {
    let m = TrafficMetrics {
        bytes_per_second: 10000,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 10000, 2.0, 40.0);
    assert!((w - 40.0).abs() < f32::EPSILON);
}

#[test]
fn prepare_flow_detail_formats() {
    let flow = TrafficFlow {
        from: "alpha".to_string(),
        to: "beta".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 5000,
            requests_per_second: 42.5,
            avg_latency_ms: 12.34,
            error_rate: 0.05,
        },
        color: Color32::RED,
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.from, "alpha");
    assert_eq!(d.to, "beta");
    assert_eq!(d.volume_label, "5000 B/s");
    assert_eq!(d.requests_label, "42.5 req/s");
    assert_eq!(d.latency_label, "12.34 ms");
    assert_eq!(d.error_rate_label, "5.00%");
}

#[test]
fn apply_intents_set_color_scheme() {
    let mut view = TrafficView::new();
    view.apply_intents(&[TrafficIntent::SetColorScheme(ColorScheme::Latency)]);
    assert_eq!(view.color_scheme(), ColorScheme::Latency);
}

#[test]
fn apply_intents_select_and_close() {
    let mut view = TrafficView::new();
    view.apply_intents(&[TrafficIntent::SelectFlow {
        from: "a".to_string(),
        to: "b".to_string(),
    }]);
    assert!(view.selected_flow().is_some());
    view.apply_intents(&[TrafficIntent::CloseDetails]);
    assert!(view.selected_flow().is_none());
}

#[test]
fn apply_intents_toggle_metrics() {
    let mut view = TrafficView::new();
    assert!(view.show_metrics());
    view.apply_intents(&[TrafficIntent::ToggleMetrics]);
    assert!(!view.show_metrics());
}

#[test]
fn apply_intents_clear() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics::default(),
        color: Color32::GREEN,
    });
    view.apply_intents(&[TrafficIntent::Clear]);
    assert_eq!(view.flow_count(), 0);
}

#[test]
fn bezier_control_points_left_to_right() {
    let (ctrl1, ctrl2) = bezier_control_points(100.0, 50.0, 400.0, 100.0);
    let dx = 400.0_f32 - 100.0;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - (100.0 + offset)).abs() < f32::EPSILON);
    assert!((ctrl1[1] - 50.0).abs() < f32::EPSILON);
    assert!((ctrl2[0] - (400.0 - offset)).abs() < f32::EPSILON);
    assert!((ctrl2[1] - 100.0).abs() < f32::EPSILON);
}

#[test]
fn bezier_control_points_right_to_left() {
    let (ctrl1, ctrl2) = bezier_control_points(400.0, 50.0, 100.0, 100.0);
    let dx: f32 = 100.0 - 400.0;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - dx.signum().mul_add(offset, 400.0)).abs() < f32::EPSILON);
    assert!((ctrl2[0] - dx.signum().mul_add(-offset, 100.0)).abs() < f32::EPSILON);
}

#[test]
fn primal_lane_layout_empty() {
    let layout = primal_lane_layout(0, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert!(layout.is_empty());
}

#[test]
fn primal_lane_layout_single() {
    let layout = primal_lane_layout(1, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 1);
    let (y, left_x, right_x) = layout[0];
    assert!((y - 300.0).abs() < f32::EPSILON);
    assert!((left_x - 80.0).abs() < f32::EPSILON);
    assert!((right_x - 720.0).abs() < f32::EPSILON);
}

#[test]
fn primal_lane_layout_three() {
    let layout = primal_lane_layout(3, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 3);
    let node_height = (600.0 - 40.0) / 3.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 20.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 80.0).abs() < f32::EPSILON);
        assert!((right_x - 720.0).abs() < f32::EPSILON);
    }
}

#[test]
fn calculate_flow_width_max_volume_zero_no_panic() {
    let m = TrafficMetrics {
        bytes_per_second: 0,
        ..Default::default()
    };
    let w = calculate_flow_width(&m, 0, 2.0, 40.0);
    assert!(
        w >= 2.0 && w <= 40.0,
        "result should be in valid range: {w}"
    );
    assert!(
        (w - 2.0).abs() < 0.01,
        "zero volume with max_vol guard should give min_width: {w}"
    );
}

#[test]
fn apply_intents_set_color_scheme_updates_flow_colors() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 50000,
            ..Default::default()
        },
        color: Color32::GREEN,
    });
    view.apply_intents(&[TrafficIntent::SetColorScheme(ColorScheme::Latency)]);
    assert_eq!(view.color_scheme(), ColorScheme::Latency);
    let flows = view.flows();
    assert_eq!(flows.len(), 1);
    assert!(
        flows[0].color != Color32::GREEN,
        "color should be recalculated"
    );
}

#[test]
fn update_from_topology_empty() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "x".to_string(),
        to: "y".to_string(),
        metrics: TrafficMetrics::default(),
        color: Color32::RED,
    });
    view.update_from_topology(&[]);
    assert_eq!(view.flow_count(), 0);
}

#[test]
fn update_from_topology_with_edges() {
    use petal_tongue_core::{PrimalId, TopologyEdge};

    let mut view = TrafficView::new();
    let edges = vec![
        TopologyEdge {
            from: PrimalId::from("alpha"),
            to: PrimalId::from("beta"),
            edge_type: "api_call".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("beta"),
            to: PrimalId::from("gamma"),
            edge_type: "capability".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
    ];
    view.update_from_topology(&edges);
    assert_eq!(view.flow_count(), 2);
    let flows = view.flows();
    assert_eq!(flows[0].from, "alpha");
    assert_eq!(flows[0].to, "beta");
    assert_eq!(flows[1].from, "beta");
    assert_eq!(flows[1].to, "gamma");
}

#[test]
fn traffic_intent_variants_eq() {
    assert_eq!(
        TrafficIntent::SetColorScheme(ColorScheme::Volume),
        TrafficIntent::SetColorScheme(ColorScheme::Volume)
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
    assert_eq!(TrafficIntent::CloseDetails, TrafficIntent::CloseDetails);
    assert_eq!(TrafficIntent::ToggleMetrics, TrafficIntent::ToggleMetrics);
    assert_eq!(TrafficIntent::Clear, TrafficIntent::Clear);
}

#[test]
fn traffic_view_render_headless() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo};

    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "alpha".to_string(),
        to: "beta".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 5000,
            requests_per_second: 10.0,
            avg_latency_ms: 5.0,
            error_rate: 0.01,
        },
        color: Color32::GREEN,
    });
    view.set_primals(vec![
        PrimalInfo::new(
            PrimalId::from("alpha"),
            "Alpha",
            "Compute",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            PrimalId::from("beta"),
            "Beta",
            "Storage",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
    ]);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            view.apply_intents(&intents);
        });
    });
}

#[test]
fn traffic_view_render_empty_headless() {
    let mut view = TrafficView::new();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = view.render(ui);
            assert!(intents.is_empty());
        });
    });
}
