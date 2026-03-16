// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-aware chart renderers for `Heatmap`, `Scatter3D`, `FieldMap`,
//! and `Spectrum` variants. Colors are selected from `DomainPalette` based
//! on an optional domain hint.

mod types;
mod validation;

use crate::domain_theme;
use egui::{RichText, Ui};
use egui_plot::{Line, Plot, PlotPoints, Points};

#[cfg(test)]
mod tests;

pub use types::{Scatter2dParams, Scatter3dParams};
pub use validation::{
    normalize_value, scatter3d_bands, validate_heatmap_dimensions, validate_scatter2d_lengths,
    validate_scatter3d_lengths, validate_spectrum_lengths, value_range,
};

/// Number of z-bands for color/size encoding (higher z = darker, larger).
pub const Z_BANDS: usize = 8;

pub fn draw_heatmap(
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
    if !validate_heatmap_dimensions(cols, rows, values.len()) {
        ui.label(RichText::new("(invalid heatmap dimensions)").color(palette.caution));
        return;
    }

    let Some((vmin, _vmax, range)) = value_range(values) else {
        ui.label(RichText::new("(no value range)").color(palette.caution));
        return;
    };
    let cell_w = (ui.available_width().min(320.0) / cols as f32).max(8.0);
    let cell_h = 14.0_f32;

    for (row, y_label) in y_labels.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(y_label).small().color(palette.text_dim));
            for col in 0..cols {
                let t = normalize_value(values[row * cols + col], vmin, range);
                let color = palette.positive.linear_multiply(t.max(0.15_f32));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
            }
        });
    }
}

/// Draw 2D scatter plot (e.g., `PCoA` ordination, UMAP embedding).
pub fn draw_scatter(ui: &mut Ui, params: &Scatter2dParams<'_>) {
    let palette = domain_theme::palette_for_domain(params.domain.unwrap_or("health"));
    let x_vals = params.x_vals;
    let y_vals = params.y_vals;
    let point_labels = params.point_labels;
    let label = params.label;
    let x_label = params.x_label;
    let y_label = params.y_label;
    let unit = params.unit;

    if !validate_scatter2d_lengths(x_vals.len(), y_vals.len()) {
        ui.label(RichText::new("(invalid scatter data)").color(palette.caution));
        return;
    }

    let count = x_vals.len();
    let has_labels = point_labels.len() == count;

    ui.label(
        RichText::new(format!("{label} ({unit}) — {count} points"))
            .strong()
            .color(palette.text_dim),
    );

    let plot_id = format!("{label}_scatter");
    let mut plot = Plot::new(plot_id)
        .height(160.0)
        .show_axes(true)
        .x_axis_label(x_label)
        .y_axis_label(y_label);

    if has_labels {
        plot = plot.label_formatter(move |_name, value| {
            let (cursor_x, cursor_y) = (value.x, value.y);
            let mut best_idx = 0usize;
            let mut best_dist = f64::INFINITY;
            for (idx, (&xi, &yi)) in x_vals.iter().zip(y_vals.iter()).enumerate() {
                let dist = (yi - cursor_y).mul_add(yi - cursor_y, (xi - cursor_x).powi(2));
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = idx;
                }
            }
            if best_idx < point_labels.len() {
                let pt_label = &point_labels[best_idx];
                format!("{pt_label}\nx: {cursor_x:.2}  y: {cursor_y:.2}")
            } else {
                format!("x: {cursor_x:.2}  y: {cursor_y:.2}")
            }
        });
    }

    let points: PlotPoints = x_vals
        .iter()
        .zip(y_vals.iter())
        .map(|(&x, &y)| [x, y])
        .collect();

    plot.show(ui, |plot_ui| {
        plot_ui.points(
            Points::new(points)
                .color(palette.info)
                .radius(3.0)
                .name(label),
        );
    });
}

pub fn draw_scatter3d(ui: &mut Ui, params: &Scatter3dParams<'_>) {
    let palette = domain_theme::palette_for_domain(params.domain.unwrap_or("health"));
    let x_vals = params.x_vals;
    let y_vals = params.y_vals;
    let z_vals = params.z_vals;
    let point_labels = params.point_labels;
    let label = params.label;
    let unit = params.unit;
    let count = x_vals.len();

    if !validate_scatter3d_lengths(x_vals.len(), y_vals.len(), z_vals.len()) {
        ui.label(RichText::new("(invalid scatter3d data)").color(palette.caution));
        return;
    }

    let (z_min, z_max) = z_vals
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &val| {
            (lo.min(val), hi.max(val))
        });

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
    let mut plot = Plot::new(plot_id)
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
                let dist = (yi - cursor_y).mul_add(yi - cursor_y, (xi - cursor_x).powi(2));
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

    let Some(bands) = scatter3d_bands(x_vals, y_vals, z_vals, Z_BANDS) else {
        return;
    };

    plot.show(ui, |plot_ui| {
        for (band, band_vec) in bands.into_iter().enumerate() {
            if band_vec.is_empty() {
                continue;
            }

            let band_lo = band as f64 / Z_BANDS as f64;
            let band_hi = (band + 1) as f64 / Z_BANDS as f64;
            let band_center = f64::midpoint(band_lo, band_hi);

            let band_points: PlotPoints = band_vec.into();
            let color = base_color.gamma_multiply(0.7f32.mul_add(1.0 - band_center as f32, 0.3));
            let radius = 2.0f32.mul_add(band_center as f32, 2.0);

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

pub fn draw_fieldmap(
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

    if !validate_heatmap_dimensions(cols, rows, values.len()) {
        ui.label(RichText::new("(invalid fieldmap dimensions)").color(palette.caution));
        return;
    }

    let Some((vmin, _vmax, range)) = value_range(values) else {
        ui.label(RichText::new("(no value range)").color(palette.caution));
        return;
    };
    let cell_w = (ui.available_width().min(320.0) / cols as f32).max(4.0);
    let cell_h = (160.0_f32 / rows as f32).max(4.0);

    for row in 0..rows {
        ui.horizontal(|ui| {
            for col in 0..cols {
                let t = normalize_value(values[row * cols + col], vmin, range);
                let color = palette.info.linear_multiply(t.max(0.1));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, color);
            }
        });
    }
}

pub fn draw_spectrum(
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

    if !validate_spectrum_lengths(frequencies.len(), amplitudes.len()) {
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
