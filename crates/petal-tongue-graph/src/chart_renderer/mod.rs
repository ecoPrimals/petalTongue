// SPDX-License-Identifier: AGPL-3.0-or-later
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
mod game_scene_renderer;
mod soundscape_renderer;

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
        DataBinding::GameScene { label, scene, .. } => {
            game_scene_renderer::draw_game_scene(ui, label, scene);
        }
        DataBinding::Soundscape {
            label, definition, ..
        } => {
            soundscape_renderer::draw_soundscape(ui, label, definition);
        }
    }
}

#[cfg(test)]
mod tests;
