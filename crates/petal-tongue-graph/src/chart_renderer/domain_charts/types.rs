// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parameter structs for domain chart rendering.

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
