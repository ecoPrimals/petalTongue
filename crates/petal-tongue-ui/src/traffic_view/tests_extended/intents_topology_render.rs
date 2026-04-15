// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::{ColorScheme, TrafficFlow, TrafficIntent, TrafficMetrics, TrafficView};

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
