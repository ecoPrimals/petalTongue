// SPDX-License-Identifier: AGPL-3.0-only
//! Extended traffic view tests: pure functions, intents, headless render.

use super::{
    ColorScheme, TrafficFlow, TrafficIntent, TrafficMetrics, TrafficView, bezier_control_points,
    calculate_flow_color, calculate_flow_width, prepare_flow_detail, primal_lane_layout,
};

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
    assert_eq!(c[0], 0);
    assert_eq!(c[1], 255);
}

#[test]
fn calculate_flow_color_volume_max() {
    let m = TrafficMetrics {
        bytes_per_second: 200_000,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Volume);
    assert_eq!(c[0], 255);
    assert_eq!(c[1], 0);
}

#[test]
fn calculate_flow_color_latency_fast() {
    let m = TrafficMetrics {
        avg_latency_ms: 0.0,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::Latency);
    assert_eq!(c[0], 0);
}

#[test]
fn calculate_flow_color_error_high() {
    let m = TrafficMetrics {
        error_rate: 0.5,
        ..Default::default()
    };
    let c = calculate_flow_color(&m, ColorScheme::ErrorRate);
    assert!(c[0] > 200);
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
        color: [255, 0, 0, 255],
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
        color: [0, 255, 0, 255],
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
        color: [0, 255, 0, 255],
    });
    view.apply_intents(&[TrafficIntent::SetColorScheme(ColorScheme::Latency)]);
    assert_eq!(view.color_scheme(), ColorScheme::Latency);
    let flows = view.flows();
    assert_eq!(flows.len(), 1);
    assert!(
        flows[0].color != [0, 255, 0, 255],
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
        color: [255, 0, 0, 255],
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
        color: [0, 255, 0, 255],
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

#[test]
fn calculate_flow_color_volume_variants() {
    let zero = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert_eq!(zero[0], 0);
    assert_eq!(zero[1], 255);
    let mid = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 50_000,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert!(mid[0] > 0);
    assert!(mid[1] < 255);
    let high = calculate_flow_color(
        &TrafficMetrics {
            bytes_per_second: 150_000,
            ..Default::default()
        },
        ColorScheme::Volume,
    );
    assert_eq!(high[0], 255);
    assert_eq!(high[1], 0);
}

#[test]
fn calculate_flow_color_latency_variants() {
    let zero = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 0.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert_eq!(zero[0], 0);
    let mid = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 50.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert!(mid[0] > 0);
    assert!(mid[1] < 255);
    let high = calculate_flow_color(
        &TrafficMetrics {
            avg_latency_ms: 150.0,
            ..Default::default()
        },
        ColorScheme::Latency,
    );
    assert_eq!(high[0], 255);
}

#[test]
fn calculate_flow_color_error_rate_variants() {
    let zero = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.0,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert_eq!(zero[0], 0);
    let mid = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 0.1,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert!(mid[0] > 0);
    let high = calculate_flow_color(
        &TrafficMetrics {
            error_rate: 1.0,
            ..Default::default()
        },
        ColorScheme::ErrorRate,
    );
    assert_eq!(high[0], 255);
}

#[test]
fn update_from_topology_flow_construction_and_metric_mapping() {
    use petal_tongue_core::{PrimalId, TopologyEdge};

    let mut view = TrafficView::new();
    let edges = vec![
        TopologyEdge {
            from: PrimalId::from("a"),
            to: PrimalId::from("b"),
            edge_type: "conn".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("xy"),
            to: PrimalId::from("z"),
            edge_type: "conn".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
    ];
    view.update_from_topology(&edges);
    assert_eq!(view.flow_count(), 2);
    let flows = view.flows();
    assert_eq!(flows[0].from, "a");
    assert_eq!(flows[0].to, "b");
    assert_eq!(flows[0].metrics.bytes_per_second, 1100);
    assert_eq!(flows[1].from, "xy");
    assert_eq!(flows[1].to, "z");
    assert_eq!(flows[1].metrics.bytes_per_second, 1200);
}

#[test]
fn apply_intents_all_variants() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics::default(),
        color: [0, 255, 0, 255],
    });
    view.apply_intents(&[TrafficIntent::SetColorScheme(ColorScheme::Latency)]);
    assert_eq!(view.color_scheme(), ColorScheme::Latency);
    view.apply_intents(&[TrafficIntent::SelectFlow {
        from: "a".to_string(),
        to: "b".to_string(),
    }]);
    assert!(view.selected_flow().is_some());
    view.apply_intents(&[TrafficIntent::CloseDetails]);
    assert!(view.selected_flow().is_none());
    view.apply_intents(&[TrafficIntent::ToggleMetrics]);
    assert!(!view.show_metrics());
    view.apply_intents(&[TrafficIntent::Clear]);
    assert_eq!(view.flow_count(), 0);
}

#[test]
fn max_volume_no_flows() {
    let view = TrafficView::new();
    assert_eq!(view.max_volume(), 1);
}

#[test]
fn max_volume_all_zero() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    view.add_flow(TrafficFlow {
        from: "b".to_string(),
        to: "c".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 0,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    assert_eq!(view.max_volume(), 0);
}

#[test]
fn max_volume_very_large() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "a".to_string(),
        to: "b".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 1,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    view.add_flow(TrafficFlow {
        from: "b".to_string(),
        to: "c".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: u64::MAX,
            ..Default::default()
        },
        color: [0, 255, 0, 255],
    });
    assert_eq!(view.max_volume(), u64::MAX);
}

#[test]
fn primal_lane_layout_one_primal() {
    let layout = primal_lane_layout(1, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 1);
    let (y, left_x, right_x) = layout[0];
    assert!((y - 300.0).abs() < f32::EPSILON);
    assert!((left_x - 80.0).abs() < f32::EPSILON);
    assert!((right_x - 720.0).abs() < f32::EPSILON);
}

#[test]
fn primal_lane_layout_twenty_primals() {
    let layout = primal_lane_layout(20, 0.0, 0.0, 800.0, 600.0, 20.0, 120.0);
    assert_eq!(layout.len(), 20);
    let node_height = (600.0 - 40.0) / 20.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 20.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 80.0).abs() < f32::EPSILON);
        assert!((right_x - 720.0).abs() < f32::EPSILON);
    }
}

#[test]
fn primal_lane_layout_very_narrow_rect() {
    let layout = primal_lane_layout(3, 0.0, 0.0, 100.0, 50.0, 5.0, 30.0);
    assert_eq!(layout.len(), 3);
    let node_height = (50.0 - 10.0) / 3.0;
    for (i, &(y, left_x, right_x)) in layout.iter().enumerate() {
        let expected_y = 5.0 + node_height * (i as f32 + 0.5);
        assert!((y - expected_y).abs() < f32::EPSILON);
        assert!((left_x - 20.0).abs() < f32::EPSILON);
        assert!((right_x - 80.0).abs() < f32::EPSILON);
    }
}

#[test]
fn bezier_control_points_diagonal() {
    let (ctrl1, ctrl2) = bezier_control_points(0.0, 0.0, 100.0, 100.0);
    let dx = 100.0_f32;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - offset).abs() < f32::EPSILON);
    assert!((ctrl1[1] - 0.0).abs() < f32::EPSILON);
    assert!((ctrl2[0] - (100.0 - offset)).abs() < f32::EPSILON);
    assert!((ctrl2[1] - 100.0).abs() < f32::EPSILON);
}

#[test]
fn bezier_control_points_reversed() {
    let (ctrl1, ctrl2) = bezier_control_points(200.0, 100.0, 50.0, 50.0);
    let dx = 50.0_f32 - 200.0;
    let offset = dx.abs() * 0.3;
    assert!((ctrl1[0] - (200.0 + offset * dx.signum())).abs() < f32::EPSILON);
    assert!((ctrl2[0] - (50.0 - offset * dx.signum())).abs() < f32::EPSILON);
}

#[test]
fn prepare_flow_detail_zero_metrics() {
    let flow = TrafficFlow {
        from: "src".to_string(),
        to: "dst".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 0,
            requests_per_second: 0.0,
            avg_latency_ms: 0.0,
            error_rate: 0.0,
        },
        color: [0, 255, 0, 255],
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.volume_label, "0 B/s");
    assert_eq!(d.requests_label, "0.0 req/s");
    assert_eq!(d.latency_label, "0.00 ms");
    assert_eq!(d.error_rate_label, "0.00%");
}

#[test]
fn prepare_flow_detail_high_metrics() {
    let flow = TrafficFlow {
        from: "src".to_string(),
        to: "dst".to_string(),
        metrics: TrafficMetrics {
            bytes_per_second: 999_999_999,
            requests_per_second: 12345.67,
            avg_latency_ms: 999.99,
            error_rate: 0.999,
        },
        color: [0, 255, 0, 255],
    };
    let d = prepare_flow_detail(&flow);
    assert_eq!(d.volume_label, "999999999 B/s");
    assert_eq!(d.requests_label, "12345.7 req/s");
    assert_eq!(d.latency_label, "999.99 ms");
    assert_eq!(d.error_rate_label, "99.90%");
}

#[test]
fn selected_flow_none() {
    let view = TrafficView::new();
    assert!(view.selected_flow().is_none());
}

#[test]
fn selected_flow_one() {
    let mut view = TrafficView::new();
    view.add_flow(TrafficFlow {
        from: "alpha".to_string(),
        to: "beta".to_string(),
        metrics: TrafficMetrics::default(),
        color: [0, 255, 0, 255],
    });
    view.apply_intents(&[TrafficIntent::SelectFlow {
        from: "alpha".to_string(),
        to: "beta".to_string(),
    }]);
    let sel = view.selected_flow();
    assert!(sel.is_some());
    let (from, to) = sel.unwrap();
    assert_eq!(from, "alpha");
    assert_eq!(to, "beta");
}
