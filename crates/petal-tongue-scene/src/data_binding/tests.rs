// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::compiler::GrammarCompiler;
use petal_tongue_types::DataBinding;

#[test]
fn timeseries_compiles_to_line() {
    let binding = DataBinding::TimeSeries {
        id: "ts1".to_string(),
        label: "Glucose".to_string(),
        x_label: "Time".to_string(),
        y_label: "mg/dL".to_string(),
        unit: "mg/dL".to_string(),
        x_values: vec![0.0, 1.0, 2.0],
        y_values: vec![90.0, 95.0, 88.0],
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Line);
    assert_eq!(expr.variables.len(), 2);
    assert_eq!(data.len(), 3);
    assert_eq!(expr.data_source, "ts1");
    assert_eq!(expr.title.as_deref(), Some("Glucose"));
}

#[test]
fn distribution_compiles_to_histogram() {
    let values = vec![0.35, 0.38, 0.36, 0.40, 0.32, 0.39, 0.37];
    let binding = DataBinding::Distribution {
        id: "dist1".to_string(),
        label: "Risk".to_string(),
        unit: "risk".to_string(),
        values: values.clone(),
        mean: 0.36,
        std: 0.03,
        comparison_value: 0.43,
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Bar);
    assert_eq!(data.len(), 20);
    let (bins, counts) = utils::histogram_bins(&values, 20);
    assert_eq!(bins.len(), 20);
    assert_eq!(counts.len(), 20);
    let total: f64 = counts.iter().sum();
    assert!((total - 7.0).abs() < 1e-9);
}

#[test]
fn bar_compiles_with_categorical_scale() {
    let binding = DataBinding::Bar {
        id: "bar1".to_string(),
        label: "Abundances".to_string(),
        categories: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        values: vec![0.3, 0.5, 0.2],
        unit: "rel".to_string(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Bar);
    assert!(
        expr.scales
            .iter()
            .any(|s| s.variable == "x" && s.scale_type == ScaleType::Categorical)
    );
    assert_eq!(data.len(), 3);
    assert_eq!(data[0]["label"], "A");
}

#[test]
fn gauge_compiles_with_value_in_title() {
    let binding = DataBinding::Gauge {
        id: "g1".to_string(),
        label: "Heart Rate".to_string(),
        value: 72.0,
        min: 40.0,
        max: 140.0,
        unit: "bpm".to_string(),
        normal_range: [60.0, 100.0],
        warning_range: [40.0, 60.0],
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Arc);
    assert!(
        expr.title
            .as_ref()
            .is_some_and(|t| t.contains("72.0") && t.contains("bpm"))
    );
    assert_eq!(data.len(), 1);
    assert!((data[0]["y"].as_f64().unwrap() - 0.32).abs() < 0.01);
}

#[test]
fn spectrum_compiles_to_area() {
    let binding = DataBinding::Spectrum {
        id: "spec1".to_string(),
        label: "Power".to_string(),
        frequencies: vec![0.0, 1.0, 2.0, 3.0],
        amplitudes: vec![0.1, 0.5, 0.3, 0.05],
        unit: "dB".to_string(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Area);
    assert_eq!(data.len(), 4);
}

#[test]
fn heatmap_compiles_to_tile_row_major() {
    let binding = DataBinding::Heatmap {
        id: "hm1".to_string(),
        label: "Matrix".to_string(),
        x_labels: vec!["A".to_string(), "B".to_string()],
        y_labels: vec!["X".to_string(), "Y".to_string()],
        values: vec![1.0, 2.0, 3.0, 4.0],
        unit: String::new(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Tile);
    assert_eq!(data.len(), 4);
    assert_eq!(data[0]["x"], 0);
    assert_eq!(data[0]["y"], 0);
    assert_eq!(data[0]["value"], 1.0);
}

#[test]
fn scatter_compiles_to_point() {
    let binding = DataBinding::Scatter {
        id: "pcoa".to_string(),
        label: "PCoA".to_string(),
        x: vec![1.0, 2.0, 3.0],
        y: vec![4.0, 5.0, 6.0],
        point_labels: vec![],
        x_label: "PC1".to_string(),
        y_label: "PC2".to_string(),
        unit: String::new(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Point);
    assert_eq!(data.len(), 3);
}

#[test]
fn scatter3d_compiles_to_point_perspective3d() {
    let binding = DataBinding::Scatter3D {
        id: "s3d".to_string(),
        label: "Embedding".to_string(),
        x: vec![1.0, 2.0],
        y: vec![3.0, 4.0],
        z: vec![5.0, 6.0],
        point_labels: vec!["p1".to_string(), "p2".to_string()],
        x_label: String::new(),
        y_label: String::new(),
        z_label: String::new(),
        unit: String::new(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Point);
    assert_eq!(expr.coordinate, CoordinateSystem::Perspective3D);
    assert_eq!(expr.variables.len(), 3);
    assert!(expr.variables.iter().any(|v| v.role == VariableRole::Z));
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["label"], "p1");
}

#[test]
fn fieldmap_compiles_to_tile_numeric_grid() {
    let binding = DataBinding::FieldMap {
        id: "fm1".to_string(),
        label: "Field".to_string(),
        grid_x: vec![0.0, 1.0],
        grid_y: vec![0.0, 1.0],
        values: vec![10.0, 20.0, 30.0, 40.0],
        unit: String::new(),
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(expr.geometry, GeometryType::Tile);
    assert_eq!(data.len(), 4);
    assert_eq!(data[0]["x"], 0.0);
    assert_eq!(data[0]["y"], 0.0);
    assert_eq!(data[0]["value"], 10.0);
}

#[test]
fn compile_batch_multiple_bindings() {
    let bindings = vec![
        DataBinding::TimeSeries {
            id: "a".to_string(),
            label: "A".to_string(),
            x_label: String::new(),
            y_label: String::new(),
            unit: String::new(),
            x_values: vec![0.0],
            y_values: vec![1.0],
        },
        DataBinding::Gauge {
            id: "b".to_string(),
            label: "B".to_string(),
            value: 50.0,
            min: 0.0,
            max: 100.0,
            unit: String::new(),
            normal_range: [0.0, 100.0],
            warning_range: [0.0, 0.0],
        },
    ];
    let results = DataBindingCompiler::compile_batch(&bindings, Some("test"));
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0.data_source, "a");
    assert_eq!(results[1].0.data_source, "b");
}

#[test]
fn compile_with_thresholds_injects_status() {
    let binding = DataBinding::Heatmap {
        id: "hm".to_string(),
        label: "Matrix".to_string(),
        x_labels: vec!["A".to_string(), "B".to_string()],
        y_labels: vec!["X".to_string()],
        values: vec![0.1, 0.4],
        unit: String::new(),
    };
    let thresholds = vec![
        petal_tongue_types::ThresholdRange {
            label: "Low".to_string(),
            min: 0.0,
            max: 0.2,
            status: "normal".to_string(),
        },
        petal_tongue_types::ThresholdRange {
            label: "High".to_string(),
            min: 0.2,
            max: 1.0,
            status: "warning".to_string(),
        },
    ];
    let (_expr, data) =
        DataBindingCompiler::compile_with_thresholds(&binding, Some("health"), &thresholds);
    assert_eq!(data[0]["status"], "normal");
    assert_eq!(data[1]["status"], "warning");
}

#[test]
fn compile_with_thresholds_no_thresholds_passthrough() {
    let binding = DataBinding::Heatmap {
        id: "hm".to_string(),
        label: "M".to_string(),
        x_labels: vec!["A".to_string()],
        y_labels: vec!["X".to_string()],
        values: vec![1.0],
        unit: String::new(),
    };
    let (_expr, data) = DataBindingCompiler::compile_with_thresholds(&binding, None, &[]);
    assert!(data[0].get("status").is_none());
}

#[test]
fn compile_with_thresholds_non_heatmap_unchanged() {
    let binding = DataBinding::Bar {
        id: "b".to_string(),
        label: "B".to_string(),
        categories: vec!["A".to_string()],
        values: vec![1.0],
        unit: String::new(),
    };
    let thresholds = vec![petal_tongue_types::ThresholdRange {
        label: "T".to_string(),
        min: 0.0,
        max: 2.0,
        status: "normal".to_string(),
    }];
    let (_expr, data) = DataBindingCompiler::compile_with_thresholds(&binding, None, &thresholds);
    assert!(data[0].get("status").is_none());
}

#[test]
fn threshold_critical_wins_over_warning() {
    assert_eq!(
        utils::resolve_threshold_status(
            0.5,
            &[
                petal_tongue_types::ThresholdRange {
                    label: "W".to_string(),
                    min: 0.0,
                    max: 1.0,
                    status: "warning".to_string(),
                },
                petal_tongue_types::ThresholdRange {
                    label: "C".to_string(),
                    min: 0.3,
                    max: 0.7,
                    status: "critical".to_string(),
                },
            ]
        ),
        "critical"
    );
}

#[test]
fn histogram_bins_empty() {
    let (bins, counts) = utils::histogram_bins(&[], 10);
    assert!(bins.is_empty());
    assert!(counts.is_empty());
}

#[test]
fn histogram_bins_single_value() {
    let (bins, counts) = utils::histogram_bins(&[42.0], 5);
    assert_eq!(bins.len(), 1);
    assert_eq!(counts.len(), 1);
    assert!((counts[0] - 1.0).abs() < f64::EPSILON);
}

#[test]
fn histogram_bins_uniform() {
    let values: Vec<f64> = (0..100).map(f64::from).collect();
    let (bins, counts) = utils::histogram_bins(&values, 10);
    assert_eq!(bins.len(), 10);
    assert_eq!(counts.len(), 10);
    let total: f64 = counts.iter().sum();
    assert!((total - 100.0).abs() < 1e-9);
}

#[test]
fn integration_compile_then_grammar_compiler() {
    let binding = DataBinding::TimeSeries {
        id: "int".to_string(),
        label: "Test".to_string(),
        x_label: String::new(),
        y_label: String::new(),
        unit: String::new(),
        x_values: vec![0.0, 1.0, 2.0],
        y_values: vec![1.0, 2.0, 3.0],
    };
    let (expr, data) = DataBindingCompiler::compile(&binding, None);
    let compiler = GrammarCompiler::new();
    let scene = compiler.compile(&expr, &data);
    assert!(scene.node_count() > 0);
    assert!(scene.total_primitives() > 0);
}

#[test]
fn gauge_min_equals_max_uses_half() {
    let binding = DataBinding::Gauge {
        id: "g".to_string(),
        label: "G".to_string(),
        value: 50.0,
        min: 100.0,
        max: 100.0,
        unit: String::new(),
        normal_range: [100.0, 100.0],
        warning_range: [0.0, 0.0],
    };
    let (_expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(data.len(), 1);
    assert!((data[0]["y"].as_f64().expect("y") - 0.5).abs() < f64::EPSILON);
}

#[test]
fn heatmap_sparse_values_uses_zero_fallback() {
    let binding = DataBinding::Heatmap {
        id: "hm".to_string(),
        label: "M".to_string(),
        x_labels: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        y_labels: vec!["X".to_string(), "Y".to_string()],
        values: vec![1.0, 2.0],
        unit: String::new(),
    };
    let (_expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(data.len(), 6);
    assert!((data[0]["value"].as_f64().expect("v") - 1.0).abs() < f64::EPSILON);
    assert!((data[2]["value"].as_f64().expect("v") - 0.0).abs() < f64::EPSILON);
}

#[test]
fn scatter3d_empty_point_labels_uses_empty_string() {
    let binding = DataBinding::Scatter3D {
        id: "s3d".to_string(),
        label: "S".to_string(),
        x: vec![1.0],
        y: vec![2.0],
        z: vec![3.0],
        point_labels: vec![],
        x_label: String::new(),
        y_label: String::new(),
        z_label: String::new(),
        unit: String::new(),
    };
    let (_expr, data) = DataBindingCompiler::compile(&binding, None);
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["label"].as_str(), Some(""));
}

#[test]
fn compile_batch_with_thresholds_fieldmap() {
    let binding = DataBinding::FieldMap {
        id: "fm".to_string(),
        label: "F".to_string(),
        grid_x: vec![0.0, 1.0],
        grid_y: vec![0.0],
        values: vec![0.1, 0.9],
        unit: String::new(),
    };
    let thresholds = vec![petal_tongue_types::ThresholdRange {
        label: "T".to_string(),
        min: 0.0,
        max: 0.5,
        status: "normal".to_string(),
    }];
    let results =
        DataBindingCompiler::compile_batch_with_thresholds(&[binding], Some("health"), &thresholds);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1[0]["status"], "normal");
    assert_eq!(results[0].1[1]["status"], "unknown");
}
