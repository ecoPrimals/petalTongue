// SPDX-License-Identifier: AGPL-3.0-or-later
//! `DataBinding` → `GrammarExpr` compiler.
//!
//! Converts healthSpring's typed data payloads into the Grammar of Graphics
//! pipeline for actual rendering.

pub mod describe;
mod utils;

use petal_tongue_types::{DataBinding, ThresholdRange};
use serde_json::Value;

use crate::grammar::{
    CoordinateSystem, GeometryType, GrammarExpr, ScaleType, VariableBinding, VariableRole,
};

/// Compiles `DataBinding` payloads into `GrammarExpr` and data rows for `GrammarCompiler`.
#[derive(Debug, Clone, Default)]
pub struct DataBindingCompiler;

impl DataBindingCompiler {
    /// Convert a `DataBinding` into a `GrammarExpr` and corresponding data rows
    /// suitable for `GrammarCompiler::compile()`.
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
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Line)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                    .map(|(x, y)| serde_json::json!({"x": x, "y": y, "data_id": id}))
                    .collect();
                (expr, data)
            }
            DataBinding::Distribution {
                id, label, values, ..
            } => {
                let (bins, counts) = utils::histogram_bins(values, 20);
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Bar)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                    .enumerate()
                    .map(|(i, (x, y))| serde_json::json!({"x": x, "y": y, "data_id": format!("bin-{i}")}))
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
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Bar)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                    .map(|((i, cat), v)| serde_json::json!({"x": i, "y": v, "label": cat, "data_id": cat}))
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
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Arc)
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
                let data = vec![serde_json::json!({"x": 0, "y": normalized, "label": label, "data_id": id})];
                (expr, data)
            }
            DataBinding::Spectrum {
                id,
                label,
                frequencies,
                amplitudes,
                ..
            } => {
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Area)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                    .map(|(x, y)| serde_json::json!({"x": x, "y": y, "data_id": id}))
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
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Tile)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                            serde_json::json!({"x": col, "y": row, "value": val, "x_label": x_label, "y_label": y_label, "data_id": format!("{x_label}:{y_label}")})
                        })
                    })
                    .collect();
                (expr, data)
            }
            DataBinding::Scatter {
                id, label, x, y, point_labels, ..
            } => {
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Point)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                    .enumerate()
                    .map(|(i, (xi, yi))| {
                        let did = point_labels.get(i).map_or_else(
                            || format!("pt-{i}"),
                            |lbl| lbl.clone(),
                        );
                        serde_json::json!({"x": xi, "y": yi, "data_id": did})
                    })
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
                let mut expr = GrammarExpr::new(id.as_str(), GeometryType::Point)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str());
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
                        let did = if label.is_empty() { format!("pt-{i}") } else { label.to_string() };
                        serde_json::json!({"x": xi, "y": yi, "z": zi, "label": label, "data_id": did})
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
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Tile)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
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
                            serde_json::json!({"x": gx, "y": gy, "value": val, "data_id": format!("({gx},{gy})")})
                        })
                    })
                    .collect();
                (expr, data)
            }
            DataBinding::GameScene { id, label, scene } => {
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Tile)
                    .with_title(label.as_str())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Linear);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                (expr, vec![scene.clone()])
            }
            DataBinding::Soundscape {
                id,
                label,
                definition,
            } => {
                let expr =
                    GrammarExpr::new(id.as_str(), GeometryType::Area).with_title(label.as_str());
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                (expr, vec![definition.clone()])
            }
            DataBinding::GenomeTrack {
                id,
                label,
                tracks,
                segments,
                ..
            } => {
                let expr = GrammarExpr::new(id.as_str(), GeometryType::Tile)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str())
                    .with_scale("x", ScaleType::Linear)
                    .with_scale("y", ScaleType::Categorical);
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = segments
                    .iter()
                    .filter_map(|seg| {
                        let track = seg.get("track").and_then(Value::as_str).unwrap_or("");
                        let track_idx = tracks.iter().position(|t| t == track).unwrap_or(0);
                        let start = seg.get("start").and_then(Value::as_f64)?;
                        let end = seg.get("end").and_then(Value::as_f64)?;
                        let mid = (start + end) / 2.0;
                        let name = seg
                            .get("name")
                            .or_else(|| seg.get("label"))
                            .and_then(Value::as_str)
                            .unwrap_or(track);
                        Some(serde_json::json!({
                            "x": mid,
                            "y": track_idx,
                            "value": end - start,
                            "x_label": format!("{start:.0}–{end:.0}"),
                            "y_label": track,
                            "label": name,
                            "data_id": name,
                        }))
                    })
                    .collect();
                (expr, data)
            }
            DataBinding::CircularMap {
                id,
                label,
                rings,
                arcs,
                ..
            } => {
                let mut expr = GrammarExpr::new(id.as_str(), GeometryType::Arc)
                    .with_x("x")
                    .with_y("y")
                    .with_title(label.as_str());
                expr.coordinate = CoordinateSystem::Polar;
                let expr = if let Some(d) = domain {
                    expr.with_domain(d)
                } else {
                    expr
                };
                let data: Vec<Value> = arcs
                    .iter()
                    .filter_map(|arc| {
                        let ring_name = arc.get("ring").and_then(Value::as_str)
                            .or_else(|| arc.get("ring").and_then(Value::as_u64).map(|_| ""))
                            .unwrap_or("");
                        let ring_idx = if let Some(idx) = arc.get("ring").and_then(Value::as_u64) {
                            idx as usize
                        } else {
                            rings.iter().position(|r| r == ring_name).unwrap_or(0)
                        };
                        let start = arc.get("start_angle").and_then(Value::as_f64)?;
                        let end = arc.get("end_angle").and_then(Value::as_f64)?;
                        let name = arc.get("label").and_then(Value::as_str)
                            .or_else(|| arc.get("name").and_then(Value::as_str))
                            .unwrap_or("feature");
                        let category = arc.get("category").and_then(Value::as_str).unwrap_or("");
                        Some(serde_json::json!({
                            "x": (start + end) / 2.0,
                            "y": ring_idx,
                            "value": end - start,
                            "label": name,
                            "category": category,
                            "data_id": name,
                        }))
                    })
                    .collect();
                (expr, data)
            }
        }
    }

    /// Compile with optional threshold ranges that color Heatmap/FieldMap cells
    /// by status (normal/warning/critical) instead of continuous intensity.
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
                    let status = utils::resolve_threshold_status(val, thresholds);
                    if let Value::Object(ref mut map) = row {
                        map.insert("status".to_string(), Value::String(status.to_string()));
                    }
                }
                row
            })
            .collect();
        (expr, data)
    }

    /// Compile a batch of `DataBindings` into a Vec of (`GrammarExpr`, data) pairs.
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

#[cfg(test)]
mod tests;
