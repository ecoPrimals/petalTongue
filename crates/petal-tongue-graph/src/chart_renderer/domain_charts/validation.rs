// SPDX-License-Identifier: AGPL-3.0-only
//! Validation and normalization helpers for domain charts (testable without egui).

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
