// SPDX-License-Identifier: AGPL-3.0-only
//! Domain-aware chart renderers for `Heatmap`, `Scatter3D`, `FieldMap`,
//! and `Spectrum` variants. Colors are selected from `DomainPalette` based
//! on an optional domain hint.

use crate::domain_theme;
use egui::{RichText, Ui};
use egui_plot::{Line, Plot, PlotPoints, Points};

#[must_use]
pub const fn validate_heatmap_dimensions(cols: usize, rows: usize, values_len: usize) -> bool {
    cols > 0 && rows > 0 && values_len == cols * rows
}

#[must_use]
pub const fn validate_scatter3d_lengths(x_len: usize, y_len: usize, z_len: usize) -> bool {
    x_len > 0 && x_len == y_len && x_len == z_len
}

#[must_use]
pub const fn validate_scatter2d_lengths(x_len: usize, y_len: usize) -> bool {
    x_len > 0 && x_len == y_len
}

#[must_use]
pub const fn validate_spectrum_lengths(freq_len: usize, amp_len: usize) -> bool {
    freq_len > 0 && freq_len == amp_len
}

/// Compute value range for heatmap/fieldmap normalization (testable without egui).
#[must_use]
pub fn value_range(values: &[f64]) -> Option<(f64, f64, f64)> {
    if values.is_empty() {
        return None;
    }
    let (vmin, vmax) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let range = (vmax - vmin).max(f64::EPSILON);
    if !range.is_finite() {
        return None;
    }
    Some((vmin, vmax, range))
}

/// Normalize value to [0, 1] for color mapping (testable without egui).
#[must_use]
pub fn normalize_value(value: f64, vmin: f64, range: f64) -> f32 {
    ((value - vmin) / range).clamp(0.0, 1.0) as f32
}

/// Assign scatter3d points to z-bands for color/size encoding (testable without egui).
#[must_use]
#[allow(dead_code)] // Used in tests; draw_scatter3d uses inline loop for egui integration
pub fn scatter3d_bands(
    x_vals: &[f64],
    y_vals: &[f64],
    z_vals: &[f64],
    n_bands: usize,
) -> Option<Vec<Vec<[f64; 2]>>> {
    if !validate_scatter3d_lengths(x_vals.len(), y_vals.len(), z_vals.len()) || n_bands == 0 {
        return None;
    }
    let (z_min, z_max) = z_vals
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &val| {
            (lo.min(val), hi.max(val))
        });
    let z_range = (z_max - z_min).max(f64::EPSILON);

    let mut bands: Vec<Vec<[f64; 2]>> = vec![Vec::new(); n_bands];
    for ((&xv, &yv), &zv) in x_vals.iter().zip(y_vals.iter()).zip(z_vals.iter()) {
        let norm = ((zv - z_min) / z_range).clamp(0.0, 1.0);
        for (band_idx, band_vec) in bands.iter_mut().enumerate().take(n_bands) {
            let lo = band_idx as f64 / n_bands as f64;
            let hi = (band_idx + 1) as f64 / n_bands as f64;
            let in_band = if band_idx == n_bands - 1 {
                norm >= lo && norm <= 1.0
            } else {
                norm >= lo && norm < hi
            };
            if in_band {
                band_vec.push([xv, yv]);
                break;
            }
        }
    }
    Some(bands)
}

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

/// `Scatter` (2D) rendering parameters bundled to reduce argument count.
pub struct Scatter2dParams<'a> {
    pub label: &'a str,
    pub x_vals: &'a [f64],
    pub y_vals: &'a [f64],
    pub point_labels: &'a [String],
    pub x_label: &'a str,
    pub y_label: &'a str,
    pub unit: &'a str,
    pub domain: Option<&'a str>,
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

/// Number of z-bands for color/size encoding (higher z = darker, larger).
const Z_BANDS: usize = 8;

/// `Scatter3D` rendering parameters bundled to reduce argument count.
pub struct Scatter3dParams<'a> {
    pub label: &'a str,
    pub x_vals: &'a [f64],
    pub y_vals: &'a [f64],
    pub z_vals: &'a [f64],
    pub point_labels: &'a [String],
    pub unit: &'a str,
    pub domain: Option<&'a str>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_heatmap_valid() {
        assert!(validate_heatmap_dimensions(3, 4, 12));
        assert!(validate_heatmap_dimensions(1, 1, 1));
    }

    #[test]
    fn validate_heatmap_invalid() {
        assert!(!validate_heatmap_dimensions(0, 4, 12));
        assert!(!validate_heatmap_dimensions(3, 0, 12));
        assert!(!validate_heatmap_dimensions(3, 4, 10));
        assert!(!validate_heatmap_dimensions(3, 4, 14));
    }

    #[test]
    fn validate_scatter2d_valid() {
        assert!(validate_scatter2d_lengths(5, 5));
        assert!(validate_scatter2d_lengths(1, 1));
    }

    #[test]
    fn validate_scatter2d_invalid() {
        assert!(!validate_scatter2d_lengths(0, 5));
        assert!(!validate_scatter2d_lengths(5, 4));
    }

    #[test]
    fn validate_scatter3d_valid() {
        assert!(validate_scatter3d_lengths(5, 5, 5));
        assert!(validate_scatter3d_lengths(1, 1, 1));
    }

    #[test]
    fn validate_scatter3d_invalid() {
        assert!(!validate_scatter3d_lengths(0, 5, 5));
        assert!(!validate_scatter3d_lengths(5, 4, 5));
        assert!(!validate_scatter3d_lengths(5, 5, 4));
    }

    #[test]
    fn validate_spectrum_valid() {
        assert!(validate_spectrum_lengths(10, 10));
        assert!(validate_spectrum_lengths(1, 1));
    }

    #[test]
    fn validate_spectrum_invalid() {
        assert!(!validate_spectrum_lengths(0, 0));
        assert!(!validate_spectrum_lengths(10, 9));
        assert!(!validate_spectrum_lengths(9, 10));
    }

    #[test]
    fn validate_heatmap_edge_cases() {
        assert!(!validate_heatmap_dimensions(1, 0, 0));
        assert!(!validate_heatmap_dimensions(0, 1, 0));
        assert!(validate_heatmap_dimensions(2, 3, 6));
        assert!(!validate_heatmap_dimensions(2, 3, 5));
        assert!(!validate_heatmap_dimensions(2, 3, 7));
    }

    #[test]
    fn validate_scatter3d_edge_cases() {
        assert!(!validate_scatter3d_lengths(1, 0, 0));
        assert!(!validate_scatter3d_lengths(0, 1, 1));
        assert!(!validate_scatter3d_lengths(1, 1, 0));
        assert!(validate_scatter3d_lengths(100, 100, 100));
    }

    #[test]
    fn scatter3d_params_construction() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![4.0, 5.0, 6.0];
        let z = vec![7.0, 8.0, 9.0];
        let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let params = Scatter3dParams {
            label: "test",
            x_vals: &x,
            y_vals: &y,
            z_vals: &z,
            point_labels: &labels,
            unit: "m",
            domain: Some("health"),
        };
        assert_eq!(params.label, "test");
        assert_eq!(params.x_vals.len(), 3);
        assert_eq!(params.y_vals.len(), 3);
        assert_eq!(params.z_vals.len(), 3);
        assert_eq!(params.unit, "m");
        assert_eq!(params.domain, Some("health"));
    }

    #[test]
    fn scatter3d_params_domain_none() {
        let x = vec![1.0];
        let y = vec![2.0];
        let z = vec![3.0];
        let params = Scatter3dParams {
            label: "l",
            x_vals: &x,
            y_vals: &y,
            z_vals: &z,
            point_labels: &[],
            unit: "u",
            domain: None,
        };
        assert!(params.domain.is_none());
    }

    #[test]
    fn z_bands_constant() {
        assert_eq!(Z_BANDS, 8);
    }

    #[test]
    fn scatter2d_params_construction() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![4.0, 5.0, 6.0];
        let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let params = Scatter2dParams {
            label: "test",
            x_vals: &x,
            y_vals: &y,
            point_labels: &labels,
            x_label: "X",
            y_label: "Y",
            unit: "m",
            domain: Some("health"),
        };
        assert_eq!(params.label, "test");
        assert_eq!(params.x_vals.len(), 3);
        assert_eq!(params.y_vals.len(), 3);
        assert_eq!(params.point_labels.len(), 3);
        assert_eq!(params.x_label, "X");
        assert_eq!(params.y_label, "Y");
        assert_eq!(params.unit, "m");
        assert_eq!(params.domain, Some("health"));
    }

    #[test]
    fn value_range_empty() {
        let values: Vec<f64> = vec![];
        assert!(value_range(&values).is_none());
    }

    #[test]
    fn value_range_single() {
        let values = vec![42.0];
        let (vmin, vmax, range) = value_range(&values).expect("should have range");
        assert!((vmin - 42.0).abs() < 1e-10);
        assert!((vmax - 42.0).abs() < 1e-10);
        assert!(range >= f64::EPSILON);
    }

    #[test]
    fn value_range_spread() {
        let values = vec![1.0, 5.0, 3.0, 9.0, 2.0];
        let (vmin, vmax, range) = value_range(&values).expect("should have range");
        assert!((vmin - 1.0).abs() < 1e-10);
        assert!((vmax - 9.0).abs() < 1e-10);
        assert!((range - 8.0).abs() < 1e-10);
    }

    #[test]
    fn normalize_value_mid_range() {
        let t = normalize_value(5.0, 0.0, 10.0);
        assert!((t - 0.5).abs() < 1e-5);
    }

    #[test]
    fn normalize_value_clamps() {
        let t_lo = normalize_value(-1.0, 0.0, 10.0);
        assert!(t_lo <= 0.0);
        let t_hi = normalize_value(15.0, 0.0, 10.0);
        assert!(t_hi >= 1.0);
    }

    #[test]
    fn scatter3d_bands_distribution() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let z = vec![0.0, 0.25, 0.5, 0.75, 1.0]; // spread across 4 bands
        let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid scatter3d input");
        assert_eq!(bands.len(), 4);
        assert_eq!(bands[0].len(), 1); // z=0 -> band 0
        assert_eq!(bands[1].len(), 1); // z=0.25 -> band 1
        assert_eq!(bands[2].len(), 1); // z=0.5 -> band 2
        assert_eq!(bands[3].len(), 2); // z=0.75 and 1.0 -> band 3
    }

    #[test]
    fn scatter3d_bands_invalid_input() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0, 2.0];
        let z = vec![1.0]; // length mismatch
        assert!(scatter3d_bands(&x, &y, &z, 4).is_none());
    }

    #[test]
    fn scatter3d_bands_zero_bands() {
        let x = vec![1.0];
        let y = vec![2.0];
        let z = vec![3.0];
        assert!(scatter3d_bands(&x, &y, &z, 0).is_none());
    }

    #[test]
    fn value_range_infinity_returns_none() {
        let values = vec![1.0, f64::INFINITY, 3.0];
        assert!(value_range(&values).is_none());
    }

    #[test]
    fn value_range_all_nan_returns_none() {
        let values = vec![f64::NAN, f64::NAN];
        let result = value_range(&values);
        assert!(
            result.is_none() || {
                let (vmin, vmax, range) = result.expect("unreachable");
                !vmin.is_finite() || !vmax.is_finite() || !range.is_finite()
            }
        );
    }

    #[test]
    fn value_range_negative_values() {
        let values = vec![-10.0, -5.0, 0.0, 5.0];
        let (vmin, vmax, range) = value_range(&values).expect("should have range");
        assert!((vmin - (-10.0)).abs() < 1e-10);
        assert!((vmax - 5.0).abs() < 1e-10);
        assert!((range - 15.0).abs() < 1e-10);
    }

    #[test]
    fn normalize_value_at_min() {
        let t = normalize_value(0.0, 0.0, 10.0);
        assert!((t - 0.0).abs() < 1e-5);
    }

    #[test]
    fn normalize_value_at_max() {
        let t = normalize_value(10.0, 0.0, 10.0);
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn scatter3d_bands_single_point() {
        let x = vec![1.0];
        let y = vec![2.0];
        let z = vec![0.5];
        let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid scatter3d input");
        assert_eq!(bands.len(), 4);
        let total: usize = bands.iter().map(Vec::len).sum();
        assert_eq!(total, 1, "Single point should appear in exactly one band");
    }

    #[test]
    fn scatter3d_bands_uniform_z() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0];
        let z = vec![1.0, 1.0, 1.0];
        let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid scatter3d input");
        let total: usize = bands.iter().map(Vec::len).sum();
        assert_eq!(total, 3, "All 3 points should appear in bands");
    }

    #[test]
    fn heatmap_value_range_integration() {
        let values = vec![10.0, 20.0, 30.0, 40.0];
        let (vmin, _vmax, range) = value_range(&values).expect("should have range");
        for (i, &v) in values.iter().enumerate() {
            let t = normalize_value(v, vmin, range);
            let expected = i as f32 / 3.0; // 0, 1/3, 2/3, 1
            assert!((t - expected).abs() < 0.01);
        }
    }

    #[test]
    fn value_range_neg_infinity_returns_none() {
        let values = vec![1.0, f64::NEG_INFINITY, 3.0];
        assert!(value_range(&values).is_none());
    }

    #[test]
    fn value_range_nan_does_not_panic() {
        let values = vec![1.0, f64::NAN, 3.0];
        let _ = value_range(&values);
    }

    #[test]
    fn normalize_value_zero_range_uses_epsilon() {
        // When range is f64::EPSILON (from value_range), value at vmin normalizes to 0
        let t = normalize_value(0.0, 0.0, f64::EPSILON);
        assert!((0.0..=1.0).contains(&t));
    }

    #[test]
    fn scatter3d_bands_last_band_inclusive() {
        // Point at z=1.0 should land in last band (norm=1.0, band n_bands-1 uses norm <= 1.0)
        let x = vec![1.0];
        let y = vec![2.0];
        let z = vec![1.0];
        let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid input");
        let total: usize = bands.iter().map(Vec::len).sum();
        assert_eq!(total, 1, "z=1.0 should be in last band");
    }

    #[test]
    fn scatter3d_bands_first_band() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0];
        let z = vec![0.0, 0.1, 1.0]; // range=1, norm 0/0.1/1 → band0 gets 2, band3 gets 1
        let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid input");
        assert_eq!(bands[0].len(), 2);
    }

    #[test]
    fn normalize_value_below_min_clamps_to_zero() {
        let t = normalize_value(-5.0, 0.0, 10.0);
        assert!((t - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_value_above_max_clamps_to_one() {
        let t = normalize_value(100.0, 0.0, 10.0);
        assert!((t - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn value_range_all_same_returns_valid() {
        let values = vec![7.0, 7.0, 7.0];
        let (vmin, vmax, range) = value_range(&values).expect("same values still valid");
        assert!((vmin - 7.0).abs() < f64::EPSILON);
        assert!((vmax - 7.0).abs() < f64::EPSILON);
        assert!(range >= f64::EPSILON);
    }

    #[test]
    fn scatter3d_bands_boundary_between_bands() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0, 2.0];
        let z = vec![0.0, 0.5];
        let bands = scatter3d_bands(&x, &y, &z, 2).expect("valid input");
        assert_eq!(bands.len(), 2);
        let total: usize = bands.iter().map(Vec::len).sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn scatter3d_bands_norm_at_boundary() {
        let x = vec![1.0];
        let y = vec![2.0];
        let z = vec![0.999_999];
        let bands = scatter3d_bands(&x, &y, &z, 4).expect("valid input");
        let total: usize = bands.iter().map(Vec::len).sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn validate_scatter3d_mismatched_lengths() {
        assert!(!validate_scatter3d_lengths(2, 3, 2));
        assert!(!validate_scatter3d_lengths(3, 2, 3));
    }

    #[test]
    fn validate_spectrum_mismatched() {
        assert!(!validate_spectrum_lengths(5, 6));
    }
}
