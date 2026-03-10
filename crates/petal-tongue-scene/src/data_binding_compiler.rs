// SPDX-License-Identifier: AGPL-3.0-only
//! DataBinding → GrammarExpr compiler.
//!
//! Converts healthSpring's typed data payloads into the Grammar of Graphics
//! pipeline for actual rendering.

use petal_tongue_core::{DataBinding, ThresholdRange};
use serde_json::Value;

use crate::grammar::{
    CoordinateSystem, GeometryType, GrammarExpr, ScaleType, VariableBinding, VariableRole,
};

/// Compiles DataBinding payloads into GrammarExpr and data rows for GrammarCompiler.
#[derive(Debug, Clone, Default)]
pub struct DataBindingCompiler;

impl DataBindingCompiler {
    /// Convert a DataBinding into a GrammarExpr and corresponding data rows
    /// suitable for GrammarCompiler::compile().
    #[must_use]
    #[expect(
        clippy::too_many_lines,
        reason = "single cohesive match over all DataBinding variants"
    )]
    pub fn compile(binding: &DataBinding, domain: Option<&str>) -> (GrammarExpr, Vec<Value>) {
        match binding {
            DataBinding::TimeSeries {
                id,
                label,
                x_values,
                y_values,
                ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Line)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = x_values
                    .iter()
                    .zip(y_values.iter())
                    .map(|(x, y)| serde_json::json!({"x": x, "y": y}))
                    .collect();
                (expr, data)
            }
            DataBinding::Distribution {
                id, label, values, ..
            } => {
                let (bins, counts) = histogram_bins(values, 20);
                let expr = GrammarExpr::new(id.clone(), GeometryType::Bar)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = bins
                    .iter()
                    .zip(counts.iter())
                    .map(|(x, y)| serde_json::json!({"x": x, "y": y}))
                    .collect();
                (expr, data)
            }
            DataBinding::Bar {
                id,
                label,
                categories,
                values,
                ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Bar)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Categorical)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = categories
                    .iter()
                    .enumerate()
                    .zip(values.iter())
                    .map(|((i, cat), v)| serde_json::json!({"x": i, "y": v, "label": cat}))
                    .collect();
                (expr, data)
            }
            DataBinding::Gauge {
                id,
                label,
                value,
                min,
                max,
                unit,
                ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Arc)
                    .with_x("x")
                    .with_y("y")
                    .with_title(format!("{label}: {value:.1} {unit}"));
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let normalized = if (max - min).abs() > f64::EPSILON {
                    (*value - min) / (max - min)
                } else {
                    0.5
                };
                let data = vec![serde_json::json!({"x": 0, "y": normalized, "label": label})];
                (expr, data)
            }
            DataBinding::Spectrum {
                id,
                label,
                frequencies,
                amplitudes,
                ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Area)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = frequencies
                    .iter()
                    .zip(amplitudes.iter())
                    .map(|(x, y)| serde_json::json!({"x": x, "y": y}))
                    .collect();
                (expr, data)
            }
            DataBinding::Heatmap {
                id,
                label,
                x_labels,
                y_labels,
                values,
                ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Tile)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Categorical)
                    .with_scale("y", ScaleType::Categorical);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let cols = x_labels.len();
                let data: Vec<Value> = y_labels
                    .iter()
                    .enumerate()
                    .flat_map(|(row, y_label)| {
                        x_labels.iter().enumerate().map(move |(col, x_label)| {
                            let val = values.get(row * cols + col).copied().unwrap_or(0.0);
                            serde_json::json!({"x": col, "y": row, "value": val, "x_label": x_label, "y_label": y_label})
                        })
                    })
                    .collect();
                (expr, data)
            }
            DataBinding::Scatter {
                id, label, x, y, ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Point)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = x
                    .iter()
                    .zip(y.iter())
                    .map(|(xi, yi)| serde_json::json!({"x": xi, "y": yi}))
                    .collect();
                (expr, data)
            }
            DataBinding::Scatter3D {
                id,
                label,
                x,
                y,
                z,
                point_labels,
                ..
            } => {
                let mut expr = GrammarExpr::new(id.clone(), GeometryType::Point)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone());
                expr.coordinate = CoordinateSystem::Perspective3D;
                expr.variables.push(VariableBinding {
                    name: "z".to_string(),
                    field: "z".to_string(),
                    role: VariableRole::Z,
                });
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = x
                    .iter()
                    .zip(y.iter())
                    .zip(z.iter())
                    .enumerate()
                    .map(|(i, ((xi, yi), zi))| {
                        let label = point_labels.get(i).map_or("", std::string::String::as_str);
                        serde_json::json!({"x": xi, "y": yi, "z": zi, "label": label})
                    })
                    .collect();
                (expr, data)
            }
            DataBinding::FieldMap {
                id,
                label,
                grid_x,
                grid_y,
                values,
                ..
            } => {
                let expr = GrammarExpr::new(id.clone(), GeometryType::Tile)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.clone())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let cols = grid_x.len();
                let data: Vec<Value> = grid_y
                    .iter()
                    .enumerate()
                    .flat_map(|(row, gy)| {
                        grid_x.iter().enumerate().map(move |(col, gx)| {
                            let val = values.get(row * cols + col).copied().unwrap_or(0.0);
                            serde_json::json!({"x": gx, "y": gy, "value": val})
                        })
                    })
                    .collect();
                (expr, data)
            }
        }
    }

    /// Compile with optional threshold ranges that color Heatmap/FieldMap cells
    /// by status (normal/warning/critical) instead of continuous intensity.
    ///
    /// Threshold ranges are matched against each cell's value. The highest-severity
    /// matching range wins. A `"status"` field is injected into data rows so the
    /// Tile geometry renderer can use domain palette status colors.
    #[must_use]
    pub fn compile_with_thresholds(
        binding: &DataBinding,
        domain: Option<&str>,
        thresholds: &[ThresholdRange],
    ) -> (GrammarExpr, Vec<Value>) {
        let (expr, data) = Self::compile(binding, domain);
        if thresholds.is_empty() {
            return (expr, data);
        }
        let needs_status = matches!(
            binding,
            DataBinding::Heatmap { .. } | DataBinding::FieldMap { .. }
        );
        if !needs_status {
            return (expr, data);
        }
        let data = data
            .into_iter()
            .map(|mut row| {
                if let Some(val) = row.get("value").and_then(Value::as_f64) {
                    let status = resolve_threshold_status(val, thresholds);
                    if let Value::Object(ref mut map) = row {
                        map.insert("status".to_string(), Value::String(status.to_string()));
                    }
                }
                row
            })
            .collect();
        (expr, data)
    }

    /// Compile a batch of DataBindings into a Vec of (GrammarExpr, data) pairs.
    #[must_use]
    pub fn compile_batch(
        bindings: &[DataBinding],
        domain: Option<&str>,
    ) -> Vec<(GrammarExpr, Vec<Value>)> {
        bindings.iter().map(|b| Self::compile(b, domain)).collect()
    }

    /// Compile a batch with threshold ranges applied to heatmap/fieldmap bindings.
    #[must_use]
    pub fn compile_batch_with_thresholds(
        bindings: &[DataBinding],
        domain: Option<&str>,
        thresholds: &[ThresholdRange],
    ) -> Vec<(GrammarExpr, Vec<Value>)> {
        bindings
            .iter()
            .map(|b| Self::compile_with_thresholds(b, domain, thresholds))
            .collect()
    }
}

/// Resolve the threshold status for a value against a set of threshold ranges.
///
/// Returns "critical" > "warning" > "normal" > "unknown" (highest severity wins).
fn resolve_threshold_status(value: f64, thresholds: &[ThresholdRange]) -> &'static str {
    let mut best: Option<&'static str> = None;
    for t in thresholds {
        if value >= t.min && value <= t.max {
            let status = match t.status.as_str() {
                "critical" => Some("critical"),
                "warning" => Some("warning"),
                "normal" => Some("normal"),
                _ => continue,
            };
            best = match (best, status) {
                (Some("critical"), _) | (_, Some("critical")) => Some("critical"),
                (Some("warning"), _) | (_, Some("warning")) => Some("warning"),
                (_, s) => s.or(best),
            };
        }
    }
    best.unwrap_or("unknown")
}

/// Bin continuous values into a histogram.
fn histogram_bins(values: &[f64], n_bins: usize) -> (Vec<f64>, Vec<f64>) {
    if values.is_empty() || n_bins == 0 {
        return (vec![], vec![]);
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        return (vec![min], vec![values.len() as f64]);
    }
    let bin_width = (max - min) / n_bins as f64;
    let mut counts = vec![0.0_f64; n_bins];
    let mut centers = Vec::with_capacity(n_bins);
    for i in 0..n_bins {
        centers.push(min + (i as f64 + 0.5) * bin_width);
    }
    for v in values {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "histogram bin index bounded by n_bins"
        )]
        let bin = ((*v - min) / bin_width).floor() as usize;
        let bin = bin.min(n_bins - 1);
        counts[bin] += 1.0;
    }
    (centers, counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::GrammarCompiler;
    use petal_tongue_core::DataBinding;

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
        let (bins, counts) = histogram_bins(&values, 20);
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
        // Value is normalized to 0..1 for arc (72 in [40,140] -> (72-40)/100 = 0.32)
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
            petal_tongue_core::ThresholdRange {
                label: "Low".to_string(),
                min: 0.0,
                max: 0.2,
                status: "normal".to_string(),
            },
            petal_tongue_core::ThresholdRange {
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
        let thresholds = vec![petal_tongue_core::ThresholdRange {
            label: "T".to_string(),
            min: 0.0,
            max: 2.0,
            status: "normal".to_string(),
        }];
        let (_expr, data) =
            DataBindingCompiler::compile_with_thresholds(&binding, None, &thresholds);
        assert!(data[0].get("status").is_none());
    }

    #[test]
    fn threshold_critical_wins_over_warning() {
        assert_eq!(
            super::resolve_threshold_status(
                0.5,
                &[
                    petal_tongue_core::ThresholdRange {
                        label: "W".to_string(),
                        min: 0.0,
                        max: 1.0,
                        status: "warning".to_string(),
                    },
                    petal_tongue_core::ThresholdRange {
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
        let (bins, counts) = histogram_bins(&[], 10);
        assert!(bins.is_empty());
        assert!(counts.is_empty());
    }

    #[test]
    fn histogram_bins_single_value() {
        let (bins, counts) = histogram_bins(&[42.0], 5);
        assert_eq!(bins.len(), 1);
        assert_eq!(counts.len(), 1);
        assert!((counts[0] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn histogram_bins_uniform() {
        let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let (bins, counts) = histogram_bins(&values, 10);
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
}
