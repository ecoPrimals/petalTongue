// SPDX-License-Identifier: AGPL-3.0-only
//! Domain-aware chart renderers for `Heatmap`, `Scatter3D`, `FieldMap`,
//! and `Spectrum` variants. Colors are selected from `DomainPalette` based
//! on an optional domain hint.

use crate::domain_theme;
use egui::{RichText, Ui};
use egui_plot::{Line, Plot, PlotPoints, Points};

pub(crate) fn draw_heatmap(
    ui: &mut Ui,
    label: &str,
    x_labels: &[String],
    y_labels: &[String],
    values: &[f64],
    unit: &str,
    domain: Option<&str>,
) {
    let palette = domain_theme::palette_for_domain(domain.unwrap_or("health"));
    ui.label(
        RichText::new(format!("{label} ({unit})"))
            .strong()
            .color(palette.text_dim),
    );

    let cols = x_labels.len();
    let rows = y_labels.len();
    if cols == 0 || rows == 0 || values.len() != cols * rows {
        ui.label(RichText::new("(invalid heatmap dimensions)").color(palette.caution));
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
            ui.label(RichText::new(y_label).small().color(palette.text_dim));
            for col in 0..cols {
                let t = ((values[row * cols + col] - vmin) / range) as f32;
                let color = palette.positive.linear_multiply(t.max(0.15));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
            }
        });
    }
}

/// Number of z-bands for color/size encoding (higher z = darker, larger).
const Z_BANDS: usize = 8;

/// `Scatter3D` rendering parameters bundled to reduce argument count.
pub(crate) struct Scatter3dParams<'a> {
    pub label: &'a str,
    pub x_vals: &'a [f64],
    pub y_vals: &'a [f64],
    pub z_vals: &'a [f64],
    pub point_labels: &'a [String],
    pub unit: &'a str,
    pub domain: Option<&'a str>,
}

pub(crate) fn draw_scatter3d(ui: &mut Ui, params: &Scatter3dParams<'_>) {
    let palette = domain_theme::palette_for_domain(params.domain.unwrap_or("health"));
    let x_vals = params.x_vals;
    let y_vals = params.y_vals;
    let z_vals = params.z_vals;
    let point_labels = params.point_labels;
    let label = params.label;
    let unit = params.unit;
    let count = x_vals.len();

    if x_vals.len() != y_vals.len() || x_vals.len() != z_vals.len() || x_vals.is_empty() {
        ui.label(RichText::new("(invalid scatter3d data)").color(palette.caution));
        return;
    }

    let (z_min, z_max) = z_vals
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &val| {
            (lo.min(val), hi.max(val))
        });
    let z_range = (z_max - z_min).max(f64::EPSILON);

    let has_labels = point_labels.len() == count;

    ui.label(
        RichText::new(format!(
            "{label} ({unit}) — {count} points · z ∈ [{z_min:.2}, {z_max:.2}]"
        ))
        .strong()
        .color(palette.text_dim),
    );

    let base_color = palette.info;

    let plot_id = format!("{label}_scatter3d");
    let mut plot = Plot::new(plot_id.clone())
        .height(160.0)
        .show_axes(true)
        .x_axis_label("x")
        .y_axis_label("y");

    if has_labels {
        plot = plot.label_formatter(move |_name, value| {
            let (cursor_x, cursor_y) = (value.x, value.y);
            let mut best_idx = 0usize;
            let mut best_dist = f64::INFINITY;
            for (idx, (&xi, &yi)) in x_vals.iter().zip(y_vals.iter()).enumerate() {
                let dist = (xi - cursor_x).powi(2) + (yi - cursor_y).powi(2);
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = idx;
                }
            }
            if best_idx < point_labels.len() {
                let pt_label = &point_labels[best_idx];
                let zi = z_vals.get(best_idx).copied().unwrap_or(0.0);
                format!("{pt_label}\nx: {cursor_x:.2}  y: {cursor_y:.2}  z: {zi:.2}")
            } else {
                format!("x: {cursor_x:.2}  y: {cursor_y:.2}")
            }
        });
    }

    plot.show(ui, |plot_ui| {
        for band in 0..Z_BANDS {
            let band_lo = band as f64 / Z_BANDS as f64;
            let band_hi = (band + 1) as f64 / Z_BANDS as f64;
            let band_center = f64::midpoint(band_lo, band_hi);
            let band_vec: Vec<[f64; 2]> = x_vals
                .iter()
                .zip(y_vals.iter())
                .zip(z_vals.iter())
                .filter_map(|((&xv, &yv), &zv)| {
                    let norm = ((zv - z_min) / z_range).clamp(0.0, 1.0);
                    if norm >= band_lo && norm < band_hi {
                        Some([xv, yv])
                    } else {
                        None
                    }
                })
                .collect();

            if band_vec.is_empty() {
                continue;
            }

            let band_points: PlotPoints = band_vec.into();
            let color = base_color.gamma_multiply(0.3 + 0.7 * (1.0 - band_center as f32));
            let radius = 2.0 + 2.0 * band_center as f32;

            plot_ui.points(
                Points::new(band_points)
                    .color(color)
                    .radius(radius)
                    .name(if band == 0 { label } else { "" }),
            );
        }
    });

    ui.label(
        RichText::new("(z encoded as color & size; hover for labels)")
            .small()
            .color(palette.text_dim),
    );
}

pub(crate) fn draw_fieldmap(
    ui: &mut Ui,
    label: &str,
    grid_x: &[f64],
    grid_y: &[f64],
    values: &[f64],
    unit: &str,
    domain: Option<&str>,
) {
    let palette = domain_theme::palette_for_domain(domain.unwrap_or("health"));
    let cols = grid_x.len();
    let rows = grid_y.len();
    ui.label(
        RichText::new(format!("{label} ({unit}) — {rows}x{cols} grid"))
            .strong()
            .color(palette.text_dim),
    );

    if cols == 0 || rows == 0 || values.len() != cols * rows {
        ui.label(RichText::new("(invalid fieldmap dimensions)").color(palette.caution));
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
                let color = palette.info.linear_multiply(t.max(0.1));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, color);
            }
        });
    }
}

pub(crate) fn draw_spectrum(
    ui: &mut Ui,
    label: &str,
    frequencies: &[f64],
    amplitudes: &[f64],
    unit: &str,
    domain: Option<&str>,
) {
    let palette = domain_theme::palette_for_domain(domain.unwrap_or("health"));
    ui.label(
        RichText::new(format!("{label} ({unit})"))
            .strong()
            .color(palette.text_dim),
    );

    if frequencies.len() != amplitudes.len() || frequencies.is_empty() {
        ui.label(RichText::new("(invalid spectrum data)").color(palette.caution));
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
            plot_ui.line(Line::new(points).name(label).fill(0.0).color(palette.info));
        });
}
