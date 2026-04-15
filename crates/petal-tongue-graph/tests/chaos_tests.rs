// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Chaos and fault injection tests for petal-tongue-graph.

use std::sync::{Arc, RwLock};
use std::thread;

use petal_tongue_core::test_fixtures::primals;
use petal_tongue_core::{DataBinding, GraphEngine, PrimalHealthStatus};
use petal_tongue_graph::{AudioSonificationRenderer, draw_channel};

fn run_draw_channel(binding: &DataBinding, domain: Option<&str>) {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| draw_channel(ui, binding, domain));
    });
}

#[rustfmt::skip]
fn chaos_binding_variants() -> Vec<DataBinding> {
    vec![
        DataBinding::TimeSeries { id: "c-ts".into(), label: "TS".into(), x_label: "x".into(), y_label: "y".into(), unit: "u".into(), x_values: vec![], y_values: vec![] },
        DataBinding::Distribution { id: "c-d".into(), label: "D".into(), unit: "u".into(), values: vec![], mean: 0.0, std: 1.0, comparison_value: 0.0 },
        DataBinding::Bar { id: "c-b".into(), label: "B".into(), categories: vec![], values: vec![], unit: "u".into() },
        DataBinding::Gauge { id: "c-g".into(), label: "G".into(), value: 0.0, min: 0.0, max: 1.0, unit: "u".into(), normal_range: [0.0, 1.0], warning_range: [0.0, 1.0] },
        DataBinding::Heatmap { id: "c-h".into(), label: "H".into(), x_labels: vec![], y_labels: vec![], values: vec![], unit: "u".into() },
        DataBinding::Scatter { id: "c-s2".into(), label: "S2".into(), x: vec![], y: vec![], point_labels: vec![], x_label: String::new(), y_label: String::new(), unit: "u".into() },
        DataBinding::Scatter3D { id: "c-s3".into(), label: "S3".into(), x: vec![], y: vec![], z: vec![], point_labels: vec![], x_label: String::new(), y_label: String::new(), z_label: String::new(), unit: "u".into() },
        DataBinding::FieldMap { id: "c-f".into(), label: "F".into(), grid_x: vec![], grid_y: vec![], values: vec![], unit: "u".into() },
        DataBinding::Spectrum { id: "c-sp".into(), label: "Sp".into(), frequencies: vec![], amplitudes: vec![], unit: "u".into() },
    ]
}

#[test]
fn chaos_concurrent_graph_insert_remove() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let threads: Vec<_> = (0..8)
        .map(|t| {
            let g = Arc::clone(&graph);
            thread::spawn(move || {
                for i in 0..200 {
                    let id = format!("chaos-node-{t}-{i}");
                    let p = primals::test_primal_with_type(&id, "Compute");
                    if let Ok(mut w) = g.write() {
                        w.add_node(p);
                    }
                    if let Ok(mut w) = g.write() {
                        let _ = w.remove_node(&id);
                    }
                }
            })
        })
        .collect();
    for h in threads {
        h.join().unwrap();
    }
    assert_eq!(graph.read().unwrap().stats().node_count, 0);
}

#[test]
fn chaos_render_empty_datasets() {
    for b in chaos_binding_variants() {
        run_draw_channel(&b, None);
        run_draw_channel(&b, Some("health"));
    }
}

#[test]
fn chaos_timeseries_extreme_point_count() {
    let n = 100_000usize;
    let x_values: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y_values: Vec<f64> = x_values.iter().map(|x| x.sin()).collect();
    run_draw_channel(
        &DataBinding::TimeSeries {
            id: "stress".into(),
            label: "Stress".into(),
            x_label: "i".into(),
            y_label: "sin(i)".into(),
            unit: "u".into(),
            x_values,
            y_values,
        },
        Some("physics"),
    );
}

#[rustfmt::skip]
#[test]
fn chaos_nonfinite_values_in_charts() {
    let cases = [
        DataBinding::Distribution { id: "nan-d".into(), label: "d".into(), unit: "u".into(), values: vec![1.0, f64::NAN, 3.0], mean: 2.0, std: 1.0, comparison_value: 2.5 },
        DataBinding::Gauge { id: "nan-g".into(), label: "g".into(), value: f64::NAN, min: 0.0, max: 100.0, unit: "u".into(), normal_range: [20.0, 80.0], warning_range: [10.0, 90.0] },
        DataBinding::Heatmap { id: "nan-h".into(), label: "h".into(), x_labels: vec!["a".into()], y_labels: vec!["b".into()], values: vec![f64::NAN], unit: "u".into() },
        DataBinding::Gauge { id: "inf-g".into(), label: "gi".into(), value: f64::INFINITY, min: 0.0, max: 100.0, unit: "u".into(), normal_range: [20.0, 80.0], warning_range: [10.0, 90.0] },
    ];
    for b in cases {
        run_draw_channel(&b, None);
    }
}

#[test]
fn chaos_rapid_chart_type_switching() {
    let variants = chaos_binding_variants();
    for _ in 0..50 {
        for b in &variants {
            run_draw_channel(b, Some("health"));
        }
    }
}

#[test]
fn chaos_audio_sonification_edge_case_data() {
    let empty = Arc::new(RwLock::new(GraphEngine::new()));
    let r0 = AudioSonificationRenderer::new(Arc::clone(&empty));
    assert!(r0.generate_audio_attributes().is_empty());
    assert!(r0.describe_soundscape().contains("silent"));
    let single = Arc::new(RwLock::new(GraphEngine::new()));
    single
        .write()
        .unwrap()
        .add_node(primals::test_primal_with_type("only", "AI"));
    let r1 = AudioSonificationRenderer::new(single);
    assert_eq!(r1.generate_audio_attributes().len(), 1);
    assert!(r1.describe_soundscape().contains("1 primals"));
    let zeros = Arc::new(RwLock::new(GraphEngine::new()));
    let mut z = primals::test_primal_with_type("z0", "Storage");
    z.capabilities = vec![];
    z.health = PrimalHealthStatus::Unknown;
    zeros.write().unwrap().add_node(z);
    let rz = AudioSonificationRenderer::new(zeros);
    let attrs = rz.generate_audio_attributes();
    assert_eq!(attrs.len(), 1);
    assert!((attrs[0].1.pitch - 0.5).abs() < f32::EPSILON);
}
