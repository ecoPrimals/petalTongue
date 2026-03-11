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
#[expect(
    clippy::too_many_lines,
    reason = "match on DataBinding variants; splitting would reduce clarity"
)]
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
    use super::basic_charts::{GaugeStatus, distribution_bins, gauge_status_for_value};
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
}
