// SPDX-License-Identifier: AGPL-3.0-only
//! Core chart renderers for standard `DataBinding` variants:
//! `TimeSeries`, `Distribution`, `Bar`, `Gauge`, plus the `NodeDetail` panel.

use crate::clinical_theme;
use egui::{RichText, Ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints, VLine};
use petal_tongue_core::DataBinding;

use super::draw_channel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GaugeStatus {
    Normal,
    Warning,
    Critical,
}

#[must_use]
pub(crate) fn gauge_status_for_value(
    value: f64,
    normal_range: &[f64; 2],
    warning_range: &[f64; 2],
) -> GaugeStatus {
    if value >= normal_range[0] && value <= normal_range[1] {
        GaugeStatus::Normal
    } else if value >= warning_range[0] && value <= warning_range[1] {
        GaugeStatus::Warning
    } else {
        GaugeStatus::Critical
    }
}

#[must_use]
pub(crate) fn distribution_bins(values: &[f64], n_bins: usize) -> Option<(f64, f64, Vec<u32>)> {
    if values.is_empty() || n_bins == 0 {
        return None;
    }
    let (lo, hi) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &v| {
            (l.min(v), h.max(v))
        });
    let bin_width = (hi - lo) / n_bins as f64;
    if !bin_width.is_finite() || bin_width <= 0.0 {
        return None;
    }
    let mut counts = vec![0u32; n_bins];
    for &v in values {
        let idx = ((v - lo) / bin_width).floor() as usize;
        let idx = idx.min(n_bins - 1);
        counts[idx] += 1;
    }
    Some((lo, hi, counts))
}

pub(crate) fn draw_timeseries(
    ui: &mut Ui,
    label: &str,
    x_label: &str,
    y_label: &str,
    x_values: &[f64],
    y_values: &[f64],
) {
    ui.label(
        RichText::new(label)
            .strong()
            .color(clinical_theme::TEXT_PRIMARY),
    );
    let points: PlotPoints = x_values
        .iter()
        .zip(y_values.iter())
        .map(|(&x, &y)| [x, y])
        .collect();
    let line = Line::new(points).color(clinical_theme::INFO).name(label);

    Plot::new(label)
        .height(160.0)
        .x_axis_label(x_label)
        .y_axis_label(y_label)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}

pub(crate) fn draw_distribution(
    ui: &mut Ui,
    label: &str,
    values: &[f64],
    mean: f64,
    std: f64,
    comparison_value: f64,
) {
    const N_BINS: usize = 30;

    ui.label(
        RichText::new(label)
            .strong()
            .color(clinical_theme::TEXT_PRIMARY),
    );

    let Some((lo, hi, counts)) = distribution_bins(values, N_BINS) else {
        ui.label("No spread in distribution");
        return;
    };
    let bin_width = (hi - lo) / N_BINS as f64;

    let bars: Vec<Bar> = counts
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let center = lo + (i as f64 + 0.5) * bin_width;
            Bar::new(center, f64::from(c))
                .width(bin_width * 0.9)
                .fill(clinical_theme::POPULATION.gamma_multiply(0.7))
        })
        .collect();

    let chart = BarChart::new(bars).name("Distribution");

    Plot::new(label)
        .height(160.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
            plot_ui.vline(
                VLine::new(mean)
                    .color(clinical_theme::INFO)
                    .name(format!("Mean: {mean:.4}")),
            );
            plot_ui.vline(
                VLine::new(mean + std)
                    .color(clinical_theme::TEXT_DIM)
                    .name("+1 SD"),
            );
            plot_ui.vline(
                VLine::new(mean - std)
                    .color(clinical_theme::TEXT_DIM)
                    .name("-1 SD"),
            );
            if comparison_value > 0.0 {
                plot_ui.vline(
                    VLine::new(lo + comparison_value * (hi - lo))
                        .color(clinical_theme::WARNING)
                        .name("Value"),
                );
            }
        });
}

pub(crate) fn draw_bar_chart(ui: &mut Ui, label: &str, categories: &[String], values: &[f64]) {
    ui.label(
        RichText::new(label)
            .strong()
            .color(clinical_theme::TEXT_PRIMARY),
    );

    let bars: Vec<Bar> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let name = categories.get(i).map_or("?", String::as_str);
            Bar::new(i as f64, v)
                .width(0.7)
                .name(name)
                .fill(clinical_theme::INFO.gamma_multiply(0.8))
        })
        .collect();

    let chart = BarChart::new(bars).name(label);

    Plot::new(label)
        .height(120.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });
}

#[expect(
    clippy::too_many_arguments,
    reason = "matches DataBinding::Gauge fields"
)]
pub(crate) fn draw_gauge(
    ui: &mut Ui,
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    normal_range: &[f64; 2],
    warning_range: &[f64; 2],
) {
    let color = match gauge_status_for_value(value, normal_range, warning_range) {
        GaugeStatus::Normal => clinical_theme::HEALTHY,
        GaugeStatus::Warning => clinical_theme::WARNING,
        GaugeStatus::Critical => clinical_theme::CRITICAL,
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(clinical_theme::TEXT_DIM));
        ui.label(
            RichText::new(format!("{value:.1} {unit}"))
                .strong()
                .color(color),
        );
    });

    let frac = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().min(300.0), 14.0),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, clinical_theme::BG_CARD);

    let normal_left = ((normal_range[0] - min) / (max - min)).clamp(0.0, 1.0);
    let normal_right = ((normal_range[1] - min) / (max - min)).clamp(0.0, 1.0);
    let nr = egui::Rect::from_min_max(
        egui::pos2(rect.left() + rect.width() * normal_left as f32, rect.top()),
        egui::pos2(
            rect.left() + rect.width() * normal_right as f32,
            rect.bottom(),
        ),
    );
    painter.rect_filled(nr, 2.0, clinical_theme::HEALTHY.gamma_multiply(0.2));

    let bar_width = rect.width() * frac as f32;
    let bar_rect = egui::Rect::from_min_size(rect.min, egui::vec2(bar_width, rect.height()));
    painter.rect_filled(bar_rect, 4.0, color.gamma_multiply(0.8));
}

/// Node data for rendering a detail panel.
///
/// Used by `draw_node_detail` to render a full node with all its data bindings.
/// Adapters can convert from `PrimalInfo` (with `data_bindings` in properties)
/// or from scenario formats.
#[derive(Debug, Clone, Default)]
pub struct NodeDetail {
    /// Display name
    pub name: String,
    /// Health score 0..100
    pub health: u8,
    /// Status text (e.g. "Active", "Degraded")
    pub status: String,
    /// Capability identifiers
    pub capabilities: Vec<String>,
    /// Data bindings to render
    pub data_bindings: Vec<DataBinding>,
}

/// Draw a node detail panel with all its data bindings.
pub fn draw_node_detail(ui: &mut Ui, node: &NodeDetail) {
    let color = clinical_theme::health_color(node.health);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&node.name).heading().color(color));
        ui.label(
            RichText::new(format!("{}%", node.health))
                .strong()
                .color(color),
        );
        ui.label(
            RichText::new(&node.status)
                .color(clinical_theme::TEXT_DIM)
                .italics(),
        );
    });

    if !node.capabilities.is_empty() {
        ui.horizontal_wrapped(|ui| {
            for cap in &node.capabilities {
                let short = cap.rsplit('.').next().unwrap_or(cap);
                ui.label(
                    RichText::new(short)
                        .small()
                        .background_color(clinical_theme::BG_CARD)
                        .color(clinical_theme::TEXT_DIM),
                );
            }
        });
    }

    ui.separator();

    for binding in &node.data_bindings {
        draw_channel(ui, binding, None);
        ui.add_space(8.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_status_normal() {
        assert_eq!(
            gauge_status_for_value(50.0, &[0.0, 100.0], &[0.0, 100.0]),
            GaugeStatus::Normal
        );
        assert_eq!(
            gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
            GaugeStatus::Normal
        );
        assert_eq!(
            gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
            GaugeStatus::Normal
        );
    }

    #[test]
    fn gauge_status_warning() {
        assert_eq!(
            gauge_status_for_value(105.0, &[0.0, 100.0], &[100.0, 120.0]),
            GaugeStatus::Warning
        );
        assert_eq!(
            gauge_status_for_value(-5.0, &[0.0, 100.0], &[-10.0, 0.0]),
            GaugeStatus::Warning
        );
    }

    #[test]
    fn gauge_status_critical() {
        assert_eq!(
            gauge_status_for_value(150.0, &[0.0, 100.0], &[100.0, 120.0]),
            GaugeStatus::Critical
        );
        assert_eq!(
            gauge_status_for_value(-20.0, &[0.0, 100.0], &[-10.0, 0.0]),
            GaugeStatus::Critical
        );
    }

    #[test]
    fn distribution_bins_spread() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 2.5, 3.5];
        let result = distribution_bins(&values, 5);
        let (lo, hi, counts) = result.unwrap();
        assert!((lo - 1.0).abs() < 1e-10);
        assert!((hi - 5.0).abs() < 1e-10);
        assert_eq!(counts.len(), 5);
        assert_eq!(counts.iter().sum::<u32>(), 7);
    }

    #[test]
    fn distribution_bins_no_spread() {
        let values = vec![42.0, 42.0, 42.0];
        let result = distribution_bins(&values, 10);
        assert!(result.is_none());
    }

    #[test]
    fn distribution_bins_empty() {
        let values: Vec<f64> = vec![];
        let result = distribution_bins(&values, 10);
        assert!(result.is_none());
    }

    #[test]
    fn distribution_bins_zero_bins() {
        let values = vec![1.0, 2.0, 3.0];
        let result = distribution_bins(&values, 0);
        assert!(result.is_none());
    }

    #[test]
    fn distribution_bins_single_bin() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = distribution_bins(&values, 1);
        let (lo, hi, counts) = result.unwrap();
        assert!((lo - 1.0).abs() < 1e-10);
        assert!((hi - 5.0).abs() < 1e-10);
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0], 5);
    }

    #[test]
    fn distribution_bins_boundary_values() {
        let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]; // 5.0 at boundary
        let result = distribution_bins(&values, 5);
        let (_lo, _hi, counts) = result.unwrap();
        assert_eq!(counts.iter().sum::<u32>(), 7);
        assert!(counts[counts.len() - 1] >= 1); // last bin gets 5.0 values
    }

    #[test]
    fn gauge_status_boundary_normal() {
        assert_eq!(
            gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
            GaugeStatus::Normal
        );
        assert_eq!(
            gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
            GaugeStatus::Normal
        );
    }

    #[test]
    fn gauge_status_boundary_warning() {
        assert_eq!(
            gauge_status_for_value(100.0, &[0.0, 99.0], &[99.0, 120.0]),
            GaugeStatus::Warning
        );
        assert_eq!(
            gauge_status_for_value(0.0, &[1.0, 100.0], &[-10.0, 1.0]),
            GaugeStatus::Warning
        );
    }

    #[test]
    fn gauge_status_enum_variants() {
        assert_eq!(GaugeStatus::Normal, GaugeStatus::Normal);
        assert_ne!(GaugeStatus::Normal, GaugeStatus::Warning);
        assert_ne!(GaugeStatus::Warning, GaugeStatus::Critical);
        assert_ne!(GaugeStatus::Normal, GaugeStatus::Critical);
    }

    #[test]
    fn node_detail_fields() {
        let node = NodeDetail {
            name: "Test".to_string(),
            health: 80,
            status: "Active".to_string(),
            capabilities: vec!["a.b.c".to_string(), "x".to_string()],
            data_bindings: vec![],
        };
        assert_eq!(node.name, "Test");
        assert_eq!(node.health, 80);
        assert_eq!(node.status, "Active");
        assert_eq!(node.capabilities.len(), 2);
        assert_eq!(node.capabilities[0], "a.b.c");
        assert_eq!(node.capabilities[1], "x");
    }

    #[test]
    fn node_detail_capability_short_name_logic() {
        // Test the rsplit('.').next().unwrap_or(cap) logic used in draw_node_detail
        fn short(cap: &str) -> String {
            cap.rsplit('.').next().unwrap_or(cap).to_string()
        }
        assert_eq!(short("ui.render"), "render");
        assert_eq!(short("ui.graph"), "graph");
        assert_eq!(short("a.b.c"), "c");
        assert_eq!(short("no-dots"), "no-dots");
        assert_eq!(short(""), "");
    }

    #[test]
    fn bar_chart_category_fallback() {
        // Test categories.get(i).map_or("?", String::as_str) logic
        let categories = ["A".to_string(), "B".to_string()];
        let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
        assert_eq!(name_for(0), "A");
        assert_eq!(name_for(1), "B");
        assert_eq!(name_for(2), "?");
    }

    #[test]
    fn bar_chart_empty_categories() {
        let categories: Vec<String> = vec![];
        let name_for = |i: usize| categories.get(i).map_or("?", String::as_str);
        assert_eq!(name_for(0), "?");
    }

    #[test]
    fn distribution_bins_infinity_returns_none() {
        let values = vec![1.0, f64::INFINITY, 3.0];
        let result = distribution_bins(&values, 5);
        assert!(result.is_none());
    }

    #[test]
    fn distribution_bins_value_exceeds_bin_index() {
        // Values that could produce idx >= n_bins before min()
        let values = vec![0.0, 0.1, 99.0, 100.0, 1000.0];
        let result = distribution_bins(&values, 5);
        let (lo, hi, counts) = result.expect("should have bins");
        assert!(lo.is_finite());
        assert!(hi.is_finite());
        assert_eq!(counts.len(), 5);
        assert_eq!(counts.iter().sum::<u32>(), 5);
    }

    #[test]
    fn gauge_status_value_at_normal_boundary_low() {
        assert_eq!(
            gauge_status_for_value(0.0, &[0.0, 100.0], &[-10.0, 110.0]),
            GaugeStatus::Normal
        );
    }

    #[test]
    fn gauge_status_value_at_normal_boundary_high() {
        assert_eq!(
            gauge_status_for_value(100.0, &[0.0, 100.0], &[-10.0, 110.0]),
            GaugeStatus::Normal
        );
    }

    #[test]
    fn node_detail_default() {
        let node = NodeDetail::default();
        assert!(node.name.is_empty());
        assert_eq!(node.health, 0);
        assert!(node.status.is_empty());
        assert!(node.capabilities.is_empty());
        assert!(node.data_bindings.is_empty());
    }

    #[test]
    fn node_detail_capability_rsplit_dot() {
        fn short(cap: &str) -> &str {
            cap.rsplit('.').next().unwrap_or(cap)
        }
        assert_eq!(short("a.b.c.d"), "d");
        assert_eq!(short("single"), "single");
    }

    #[test]
    fn distribution_bins_negative_values() {
        let values = vec![-10.0, -5.0, 0.0, 5.0, 10.0];
        let result = distribution_bins(&values, 5);
        let (lo, hi, counts) = result.expect("negative range valid");
        assert!((lo - (-10.0)).abs() < f64::EPSILON);
        assert!((hi - 10.0).abs() < f64::EPSILON);
        assert_eq!(counts.iter().sum::<u32>(), 5);
    }

    #[test]
    fn gauge_status_value_at_warning_boundary() {
        assert_eq!(
            gauge_status_for_value(120.0, &[0.0, 100.0], &[100.0, 120.0]),
            GaugeStatus::Warning
        );
        assert_eq!(
            gauge_status_for_value(-10.0, &[0.0, 100.0], &[-10.0, 0.0]),
            GaugeStatus::Warning
        );
    }

    #[test]
    fn gauge_status_value_at_critical_boundary() {
        assert_eq!(
            gauge_status_for_value(121.0, &[0.0, 100.0], &[100.0, 120.0]),
            GaugeStatus::Critical
        );
    }
}
