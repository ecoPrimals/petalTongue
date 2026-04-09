// SPDX-License-Identifier: AGPL-3.0-or-later
//! Core chart renderers for standard `DataBinding` variants:
//! `TimeSeries`, `Distribution`, `Bar`, `Gauge`, plus the `NodeDetail` panel.

use crate::clinical_theme;
use egui::{RichText, Ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints, VLine};
use petal_tongue_core::DataBinding;

use super::draw_channel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeStatus {
    Normal,
    Warning,
    Critical,
}

#[must_use]
pub fn gauge_status_for_value(
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
pub fn distribution_bins(values: &[f64], n_bins: usize) -> Option<(f64, f64, Vec<u32>)> {
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

pub fn draw_timeseries(
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

pub fn draw_distribution(
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
            let center = (i as f64 + 0.5).mul_add(bin_width, lo);
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
                    VLine::new(comparison_value.mul_add(hi - lo, lo))
                        .color(clinical_theme::WARNING)
                        .name("Value"),
                );
            }
        });
}

pub fn draw_bar_chart(ui: &mut Ui, label: &str, categories: &[String], values: &[f64]) {
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
pub fn draw_gauge(
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
        egui::pos2(
            rect.width().mul_add(normal_left as f32, rect.left()),
            rect.top(),
        ),
        egui::pos2(
            rect.width().mul_add(normal_right as f32, rect.left()),
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
mod tests;
