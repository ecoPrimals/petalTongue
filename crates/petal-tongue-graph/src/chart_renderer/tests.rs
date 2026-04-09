// SPDX-License-Identifier: AGPL-3.0-or-later

use super::basic_charts::{GaugeStatus, distribution_bins, gauge_status_for_value};
use super::*;
use petal_tongue_core::DataBinding;

fn run_draw_channel(binding: &DataBinding, domain: Option<&str>) {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            draw_channel(ui, binding, domain);
        });
    });
}

#[test]
fn test_node_detail_default() {
    let node = NodeDetail::default();
    assert!(node.name.is_empty());
    assert_eq!(node.health, 0);
    assert!(node.status.is_empty());
    assert!(node.capabilities.is_empty());
    assert!(node.data_bindings.is_empty());
}

#[test]
fn test_node_detail_with_data() {
    let node = NodeDetail {
        name: "Test Node".to_string(),
        health: 95,
        status: "Active".to_string(),
        capabilities: vec!["ui.render".to_string(), "ui.graph".to_string()],
        data_bindings: vec![],
    };
    assert_eq!(node.name, "Test Node");
    assert_eq!(node.health, 95);
    assert_eq!(node.status, "Active");
    assert_eq!(node.capabilities.len(), 2);
}

#[test]
fn test_gauge_status_normal() {
    let normal = [20.0, 80.0];
    let warning = [10.0, 90.0];
    assert_eq!(
        gauge_status_for_value(50.0, &normal, &warning),
        GaugeStatus::Normal
    );
    assert_eq!(
        gauge_status_for_value(20.0, &normal, &warning),
        GaugeStatus::Normal
    );
    assert_eq!(
        gauge_status_for_value(80.0, &normal, &warning),
        GaugeStatus::Normal
    );
}

#[test]
fn test_gauge_status_warning() {
    let normal = [20.0, 80.0];
    let warning = [10.0, 90.0];
    assert_eq!(
        gauge_status_for_value(15.0, &normal, &warning),
        GaugeStatus::Warning
    );
    assert_eq!(
        gauge_status_for_value(85.0, &normal, &warning),
        GaugeStatus::Warning
    );
    assert_eq!(
        gauge_status_for_value(10.0, &normal, &warning),
        GaugeStatus::Warning
    );
    assert_eq!(
        gauge_status_for_value(90.0, &normal, &warning),
        GaugeStatus::Warning
    );
}

#[test]
fn test_gauge_status_critical() {
    let normal = [20.0, 80.0];
    let warning = [10.0, 90.0];
    assert_eq!(
        gauge_status_for_value(5.0, &normal, &warning),
        GaugeStatus::Critical
    );
    assert_eq!(
        gauge_status_for_value(95.0, &normal, &warning),
        GaugeStatus::Critical
    );
}

#[test]
fn test_distribution_bins_empty_returns_none() {
    assert!(distribution_bins(&[], 10).is_none());
}

#[test]
fn test_distribution_bins_zero_bins_returns_none() {
    assert!(distribution_bins(&[1.0, 2.0, 3.0], 0).is_none());
}

#[test]
fn test_distribution_bins_single_value_returns_none() {
    // Single value gives bin_width=0, so returns None
    assert!(distribution_bins(&[42.0], 5).is_none());
}

#[test]
fn test_distribution_bins_two_values() {
    let result = distribution_bins(&[1.0, 2.0], 2);
    assert!(result.is_some());
    let (lo, hi, counts) = result.expect("two values should produce bins");
    assert!((lo - 1.0).abs() < f64::EPSILON);
    assert!((hi - 2.0).abs() < f64::EPSILON);
    assert_eq!(counts.len(), 2);
    assert_eq!(counts.iter().sum::<u32>(), 2);
}

#[test]
fn test_distribution_bins_spread() {
    let values: Vec<f64> = (0..100).map(f64::from).collect();
    let result = distribution_bins(&values, 10);
    assert!(result.is_some());
    let (lo, hi, counts) = result.expect("spread should produce bins");
    assert!((lo - 0.0).abs() < f64::EPSILON);
    assert!((hi - 99.0).abs() < f64::EPSILON);
    assert_eq!(counts.len(), 10);
    assert_eq!(counts.iter().sum::<u32>(), 100);
}

#[test]
fn test_distribution_bins_boundary_values() {
    let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let result = distribution_bins(&values, 3);
    assert!(result.is_some());
    let (lo, hi, counts) = result.expect("boundaries should produce bins");
    assert!((lo - 0.0).abs() < f64::EPSILON);
    assert!((hi - 9.0).abs() < f64::EPSILON);
    assert_eq!(counts.iter().sum::<u32>(), 10);
}

fn binding_label(binding: &DataBinding) -> &str {
    match binding {
        DataBinding::TimeSeries { label, .. }
        | DataBinding::Distribution { label, .. }
        | DataBinding::Bar { label, .. }
        | DataBinding::Gauge { label, .. }
        | DataBinding::Heatmap { label, .. }
        | DataBinding::Scatter { label, .. }
        | DataBinding::Scatter3D { label, .. }
        | DataBinding::FieldMap { label, .. }
        | DataBinding::Spectrum { label, .. }
        | DataBinding::GameScene { label, .. }
        | DataBinding::Soundscape { label, .. } => label,
    }
}

#[test]
fn test_draw_channel_dispatch_timeseries() {
    let b = DataBinding::TimeSeries {
        id: "t1".to_string(),
        label: "TS".to_string(),
        x_label: "X".to_string(),
        y_label: "Y".to_string(),
        unit: "u".to_string(),
        x_values: vec![0.0, 1.0],
        y_values: vec![1.0, 2.0],
    };
    assert_eq!(binding_label(&b), "TS");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_distribution() {
    let b = DataBinding::Distribution {
        id: "d1".to_string(),
        label: "Dist".to_string(),
        unit: "u".to_string(),
        values: vec![1.0, 2.0],
        mean: 1.5,
        std: 0.5,
        comparison_value: 1.8,
    };
    assert_eq!(binding_label(&b), "Dist");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_bar() {
    let b = DataBinding::Bar {
        id: "b1".to_string(),
        label: "Bar".to_string(),
        categories: vec!["A".to_string()],
        values: vec![1.0],
        unit: "u".to_string(),
    };
    assert_eq!(binding_label(&b), "Bar");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_gauge() {
    let b = DataBinding::Gauge {
        id: "g1".to_string(),
        label: "Gauge".to_string(),
        value: 50.0,
        min: 0.0,
        max: 100.0,
        unit: "u".to_string(),
        normal_range: [20.0, 80.0],
        warning_range: [10.0, 90.0],
    };
    assert_eq!(binding_label(&b), "Gauge");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_heatmap() {
    let b = DataBinding::Heatmap {
        id: "h1".to_string(),
        label: "Heatmap".to_string(),
        x_labels: vec!["A".to_string()],
        y_labels: vec!["B".to_string()],
        values: vec![1.0],
        unit: "u".to_string(),
    };
    assert_eq!(binding_label(&b), "Heatmap");
    run_draw_channel(&b, Some("health"));
}

#[test]
fn test_draw_channel_dispatch_scatter() {
    let b = DataBinding::Scatter {
        id: "s1".to_string(),
        label: "Scatter".to_string(),
        x: vec![1.0],
        y: vec![2.0],
        point_labels: vec![],
        x_label: String::new(),
        y_label: String::new(),
        unit: "u".to_string(),
    };
    assert_eq!(binding_label(&b), "Scatter");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_scatter3d() {
    let b = DataBinding::Scatter3D {
        id: "s3".to_string(),
        label: "Scatter3D".to_string(),
        x: vec![1.0],
        y: vec![2.0],
        z: vec![3.0],
        point_labels: vec![],
        x_label: String::new(),
        y_label: String::new(),
        z_label: String::new(),
        unit: "u".to_string(),
    };
    assert_eq!(binding_label(&b), "Scatter3D");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_fieldmap() {
    let b = DataBinding::FieldMap {
        id: "f1".to_string(),
        label: "FieldMap".to_string(),
        grid_x: vec![0.0, 1.0],
        grid_y: vec![0.0],
        values: vec![1.0, 2.0],
        unit: "u".to_string(),
    };
    assert_eq!(binding_label(&b), "FieldMap");
    run_draw_channel(&b, None);
}

#[test]
fn test_draw_channel_dispatch_spectrum() {
    let b = DataBinding::Spectrum {
        id: "sp1".to_string(),
        label: "Spectrum".to_string(),
        frequencies: vec![1.0],
        amplitudes: vec![0.5],
        unit: "u".to_string(),
    };
    assert_eq!(binding_label(&b), "Spectrum");
    run_draw_channel(&b, None);
}

#[expect(
    clippy::too_many_lines,
    reason = "data-driven test fixture: 9 enum variants with field initialization"
)]
fn all_binding_variants() -> Vec<(&'static str, DataBinding)> {
    vec![
        (
            "TS",
            DataBinding::TimeSeries {
                id: String::new(),
                label: "TS".to_string(),
                x_label: String::new(),
                y_label: String::new(),
                unit: String::new(),
                x_values: vec![],
                y_values: vec![],
            },
        ),
        (
            "Dist",
            DataBinding::Distribution {
                id: String::new(),
                label: "Dist".to_string(),
                unit: String::new(),
                values: vec![],
                mean: 0.0,
                std: 0.0,
                comparison_value: 0.0,
            },
        ),
        (
            "Bar",
            DataBinding::Bar {
                id: String::new(),
                label: "Bar".to_string(),
                categories: vec![],
                values: vec![],
                unit: String::new(),
            },
        ),
        (
            "G",
            DataBinding::Gauge {
                id: String::new(),
                label: "G".to_string(),
                value: 0.0,
                min: 0.0,
                max: 100.0,
                unit: String::new(),
                normal_range: [0.0, 100.0],
                warning_range: [0.0, 100.0],
            },
        ),
        (
            "H",
            DataBinding::Heatmap {
                id: String::new(),
                label: "H".to_string(),
                x_labels: vec![],
                y_labels: vec![],
                values: vec![],
                unit: String::new(),
            },
        ),
        (
            "S",
            DataBinding::Scatter {
                id: String::new(),
                label: "S".to_string(),
                x: vec![],
                y: vec![],
                point_labels: vec![],
                x_label: String::new(),
                y_label: String::new(),
                unit: String::new(),
            },
        ),
        (
            "S3",
            DataBinding::Scatter3D {
                id: String::new(),
                label: "S3".to_string(),
                x: vec![],
                y: vec![],
                z: vec![],
                point_labels: vec![],
                x_label: String::new(),
                y_label: String::new(),
                z_label: String::new(),
                unit: String::new(),
            },
        ),
        (
            "F",
            DataBinding::FieldMap {
                id: String::new(),
                label: "F".to_string(),
                grid_x: vec![],
                grid_y: vec![],
                values: vec![],
                unit: String::new(),
            },
        ),
        (
            "Sp",
            DataBinding::Spectrum {
                id: String::new(),
                label: "Sp".to_string(),
                frequencies: vec![],
                amplitudes: vec![],
                unit: String::new(),
            },
        ),
    ]
}

#[test]
fn test_binding_label_all_variants() {
    for (expected, binding) in all_binding_variants() {
        assert_eq!(binding_label(&binding), expected);
    }
}
