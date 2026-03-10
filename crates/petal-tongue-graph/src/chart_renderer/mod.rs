// SPDX-License-Identifier: AGPL-3.0-only
//! Rendering functions for data bindings using egui.
//!
//! Each function takes an `egui::Ui` and a `DataBinding` and draws the
//! appropriate visualization.
//!
//! Rendering code necessarily converts between f64 domain values and f32
//! pixel coordinates; these truncations are intentional and harmless.
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "rendering: domain f64 → pixel f32 truncation is intentional"
)]

mod basic_charts;
mod domain_charts;

use egui::Ui;
use petal_tongue_core::DataBinding;

pub use basic_charts::{NodeDetail, draw_node_detail};

/// Draw a single data binding.
///
/// If `domain` is provided, uses a domain-aware color palette (e.g. "health",
/// "physics", "ecology"). If `None`, defaults to the health palette for
/// backward compatibility.
#[allow(clippy::too_many_lines)]
pub fn draw_channel(ui: &mut Ui, binding: &DataBinding, domain: Option<&str>) {
    match binding {
        DataBinding::TimeSeries {
            label,
            x_label,
            y_label,
            x_values,
            y_values,
            ..
        } => basic_charts::draw_timeseries(ui, label, x_label, y_label, x_values, y_values),
        DataBinding::Distribution {
            label,
            values,
            mean,
            std,
            comparison_value,
            ..
        } => basic_charts::draw_distribution(ui, label, values, *mean, *std, *comparison_value),
        DataBinding::Bar {
            label,
            categories,
            values,
            ..
        } => basic_charts::draw_bar_chart(ui, label, categories, values),
        DataBinding::Gauge {
            label,
            value,
            min,
            max,
            unit,
            normal_range,
            warning_range,
            ..
        } => basic_charts::draw_gauge(
            ui,
            label,
            *value,
            *min,
            *max,
            unit,
            normal_range,
            warning_range,
        ),
        DataBinding::Heatmap {
            label,
            x_labels,
            y_labels,
            values,
            unit,
            ..
        } => {
            domain_charts::draw_heatmap(ui, label, x_labels, y_labels, values, unit, domain);
        }
        DataBinding::Scatter {
            label,
            x,
            y,
            point_labels,
            x_label,
            y_label,
            unit,
            ..
        } => {
            domain_charts::draw_scatter(
                ui,
                &domain_charts::Scatter2dParams {
                    label,
                    x_vals: x,
                    y_vals: y,
                    point_labels,
                    x_label,
                    y_label,
                    unit,
                    domain,
                },
            );
        }
        DataBinding::Scatter3D {
            label,
            x,
            y,
            z,
            point_labels,
            unit,
            ..
        } => {
            domain_charts::draw_scatter3d(
                ui,
                &domain_charts::Scatter3dParams {
                    label,
                    x_vals: x,
                    y_vals: y,
                    z_vals: z,
                    point_labels,
                    unit,
                    domain,
                },
            );
        }
        DataBinding::FieldMap {
            label,
            grid_x,
            grid_y,
            values,
            unit,
            ..
        } => {
            domain_charts::draw_fieldmap(ui, label, grid_x, grid_y, values, unit, domain);
        }
        DataBinding::Spectrum {
            label,
            frequencies,
            amplitudes,
            unit,
            ..
        } => {
            domain_charts::draw_spectrum(ui, label, frequencies, amplitudes, unit, domain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
