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

use crate::clinical_theme;
use egui::{RichText, Ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints, VLine};
use petal_tongue_core::DataBinding;

/// Draw a single data binding.
pub fn draw_channel(ui: &mut Ui, binding: &DataBinding) {
    match binding {
        DataBinding::TimeSeries {
            label,
            x_label,
            y_label,
            x_values,
            y_values,
            ..
        } => draw_timeseries(ui, label, x_label, y_label, x_values, y_values),
        DataBinding::Distribution {
            label,
            values,
            mean,
            std,
            comparison_value,
            ..
        } => draw_distribution(ui, label, values, *mean, *std, *comparison_value),
        DataBinding::Bar {
            label,
            categories,
            values,
            ..
        } => draw_bar_chart(ui, label, categories, values),
        DataBinding::Gauge {
            label,
            value,
            min,
            max,
            unit,
            normal_range,
            warning_range,
            ..
        } => draw_gauge(
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
            draw_heatmap(ui, label, x_labels, y_labels, values, unit);
        }
        DataBinding::Scatter3D {
            label,
            x,
            y,
            z,
            unit,
            ..
        } => {
            draw_scatter3d(ui, label, x, y, z, unit);
        }
        DataBinding::FieldMap {
            label,
            grid_x,
            grid_y,
            values,
            unit,
            ..
        } => {
            draw_fieldmap(ui, label, grid_x, grid_y, values, unit);
        }
        DataBinding::Spectrum {
            label,
            frequencies,
            amplitudes,
            unit,
            ..
        } => {
            draw_spectrum(ui, label, frequencies, amplitudes, unit);
        }
    }
}

fn draw_timeseries(
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

fn draw_distribution(
    ui: &mut Ui,
    label: &str,
    values: &[f64],
    mean: f64,
    std: f64,
    comparison_value: f64,
) {
    ui.label(
        RichText::new(label)
            .strong()
            .color(clinical_theme::TEXT_PRIMARY),
    );

    let n_bins = 30;
    let (lo, hi) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let bin_width = (hi - lo) / n_bins as f64;
    if bin_width <= 0.0 {
        ui.label("No spread in distribution");
        return;
    }

    let mut counts = vec![0u32; n_bins];
    for &v in values {
        let idx = ((v - lo) / bin_width).floor() as usize;
        let idx = idx.min(n_bins - 1);
        counts[idx] += 1;
    }

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

fn draw_bar_chart(ui: &mut Ui, label: &str, categories: &[String], values: &[f64]) {
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
fn draw_gauge(
    ui: &mut Ui,
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    normal_range: &[f64; 2],
    warning_range: &[f64; 2],
) {
    let color = if value >= normal_range[0] && value <= normal_range[1] {
        clinical_theme::HEALTHY
    } else if value >= warning_range[0] && value <= warning_range[1] {
        clinical_theme::WARNING
    } else {
        clinical_theme::CRITICAL
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
        draw_channel(ui, binding);
        ui.add_space(8.0);
    }
}

fn draw_heatmap(
    ui: &mut Ui,
    label: &str,
    x_labels: &[String],
    y_labels: &[String],
    values: &[f64],
    unit: &str,
) {
    ui.label(
        RichText::new(format!("{label} ({unit})"))
            .strong()
            .color(clinical_theme::TEXT_DIM),
    );

    let cols = x_labels.len();
    let rows = y_labels.len();
    if cols == 0 || rows == 0 || values.len() != cols * rows {
        ui.label(RichText::new("(invalid heatmap dimensions)").color(clinical_theme::WARNING));
        return;
    }

    let (vmin, vmax) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let range = (vmax - vmin).max(f64::EPSILON);
    let cell_w = (ui.available_width().min(320.0) / cols as f32).max(8.0);
    let cell_h = 14.0_f32;

    for (row, y_label) in y_labels.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(y_label)
                    .small()
                    .color(clinical_theme::TEXT_DIM),
            );
            for col in 0..cols {
                let t = ((values[row * cols + col] - vmin) / range) as f32;
                let color = clinical_theme::HEALTHY.linear_multiply(t.max(0.15));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
            }
        });
    }
}

fn draw_scatter3d(ui: &mut Ui, label: &str, x: &[f64], y: &[f64], z: &[f64], unit: &str) {
    ui.label(
        RichText::new(format!("{label} ({unit}) — {n} points", n = x.len()))
            .strong()
            .color(clinical_theme::TEXT_DIM),
    );

    if x.len() != y.len() || x.len() != z.len() || x.is_empty() {
        ui.label(RichText::new("(invalid scatter3d data)").color(clinical_theme::WARNING));
        return;
    }

    let points: PlotPoints = x.iter().zip(y.iter()).map(|(&xi, &yi)| [xi, yi]).collect();
    Plot::new(format!("{label}_scatter3d"))
        .height(160.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(points)
                    .name(label)
                    .style(egui_plot::LineStyle::dotted_dense()),
            );
        });
    ui.label(
        RichText::new("(z-axis projected; full 3D requires GPU renderer)")
            .small()
            .color(clinical_theme::TEXT_DIM),
    );
}

fn draw_fieldmap(
    ui: &mut Ui,
    label: &str,
    grid_x: &[f64],
    grid_y: &[f64],
    values: &[f64],
    unit: &str,
) {
    let cols = grid_x.len();
    let rows = grid_y.len();
    ui.label(
        RichText::new(format!("{label} ({unit}) — {rows}x{cols} grid"))
            .strong()
            .color(clinical_theme::TEXT_DIM),
    );

    if cols == 0 || rows == 0 || values.len() != cols * rows {
        ui.label(RichText::new("(invalid fieldmap dimensions)").color(clinical_theme::WARNING));
        return;
    }

    let (vmin, vmax) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let range = (vmax - vmin).max(f64::EPSILON);
    let cell_w = (ui.available_width().min(320.0) / cols as f32).max(4.0);
    let cell_h = (160.0_f32 / rows as f32).max(4.0);

    for row in 0..rows {
        ui.horizontal(|ui| {
            for col in 0..cols {
                let t = ((values[row * cols + col] - vmin) / range) as f32;
                let color = clinical_theme::INFO.linear_multiply(t.max(0.1));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, color);
            }
        });
    }
}

fn draw_spectrum(ui: &mut Ui, label: &str, frequencies: &[f64], amplitudes: &[f64], unit: &str) {
    ui.label(
        RichText::new(format!("{label} ({unit})"))
            .strong()
            .color(clinical_theme::TEXT_DIM),
    );

    if frequencies.len() != amplitudes.len() || frequencies.is_empty() {
        ui.label(RichText::new("(invalid spectrum data)").color(clinical_theme::WARNING));
        return;
    }

    let points: PlotPoints = frequencies
        .iter()
        .zip(amplitudes.iter())
        .map(|(&f, &a)| [f, a])
        .collect();
    Plot::new(format!("{label}_spectrum"))
        .height(120.0)
        .x_axis_label("Frequency")
        .y_axis_label("Amplitude")
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(points)
                    .name(label)
                    .fill(0.0)
                    .color(clinical_theme::INFO),
            );
        });
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
