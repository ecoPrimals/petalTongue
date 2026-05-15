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

#[expect(dead_code, reason = "pre-SceneGraph legacy renderer — SceneGraph pipeline handles heatmap via grammar compilation")]
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

    let Some((vmin, vmax, range)) = value_range(values) else {
        ui.label(RichText::new("(no value range)").color(palette.caution));
        return;
    };
    let cell_w = (ui.available_width().min(320.0) / cols as f32).max(8.0);
    let cell_h = 14.0_f32;

    // Column labels (x-axis)
    if cols > 1 {
        ui.horizontal(|ui| {
            ui.label(RichText::new("").small()); // spacer for y-label column
            for x_label in x_labels {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                let truncated = if x_label.len() > 6 {
                    format!("{}…", &x_label[..5])
                } else {
                    x_label.clone()
                };
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    truncated,
                    egui::FontId::proportional(8.0),
                    palette.text_dim,
                );
            }
        });
    }

    for (row, y_label) in y_labels.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(y_label).small().color(palette.text_dim));
            for col in 0..cols {
                let t = normalize_value(values[row * cols + col], vmin, range);
                let color = palette.positive.linear_multiply(t.max(0.15_f32));
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(cell_w, cell_h), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
                if resp.hovered() {
                    let val = values[row * cols + col];
                    let x_lbl = x_labels.get(col).map_or("?", String::as_str);
                    resp.on_hover_text(format!("{y_label} × {x_lbl}: {val:.3}"));
                }
            }
        });
    }

    // Colorbar
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{vmin:.2}")).small().color(palette.text_dim));
        let bar_w = ui.available_width().min(200.0);
        let n_stops = 20;
        let stop_w = bar_w / n_stops as f32;
        for i in 0..n_stops {
            let t = (i as f32 + 0.5) / n_stops as f32;
            let color = palette.positive.linear_multiply(t.max(0.15));
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(stop_w, 10.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, color);
        }
        ui.label(RichText::new(format!("{vmax:.2}")).small().color(palette.text_dim));
    });
}

/// Draw a linear genome track with stacked horizontal bars.
///
/// Expects JSON segments with `start`, `end`, `track` (category name),
/// optional `strand` (+/-), optional `label`.
#[expect(dead_code, reason = "pre-SceneGraph legacy renderer — SceneGraph pipeline handles genome_track via grammar compilation")]
pub fn draw_genome_track(
    ui: &mut Ui,
    label: &str,
    sequence_length: f64,
    tracks: &[String],
    segments: &[serde_json::Value],
    unit: &str,
    domain: Option<&str>,
) {
    let palette = domain_theme::palette_for_domain(domain.unwrap_or("health"));
    ui.label(
        RichText::new(format!("{label} ({unit}) — {:.0} bp", sequence_length))
            .strong()
            .color(palette.text_dim),
    );

    let avail_w = ui.available_width().min(400.0);
    let track_height = 16.0_f32;
    let track_spacing = 2.0_f32;

    // Genomic axis
    let axis_ticks = format_genomic_axis(sequence_length);
    ui.horizontal(|ui| {
        for (pos, tick_label) in &axis_ticks {
            let frac = (*pos / sequence_length) as f32;
            let x_off = frac * avail_w;
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(0.0, 0.0), egui::Sense::hover());
            ui.painter().text(
                egui::pos2(rect.left() + x_off, rect.top()),
                egui::Align2::CENTER_TOP,
                tick_label,
                egui::FontId::proportional(8.0),
                palette.text_dim,
            );
        }
    });

    // One row per track category
    let category_colors: Vec<egui::Color32> = tracks
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let hues = [
                palette.info,
                palette.positive,
                palette.caution,
                palette.text_dim,
            ];
            hues[i % hues.len()]
        })
        .collect();

    for (track_idx, track_name) in tracks.iter().enumerate() {
        let base_color = category_colors[track_idx];
        ui.horizontal(|ui| {
            ui.label(RichText::new(track_name).small().color(palette.text_dim));
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(avail_w, track_height + track_spacing),
                egui::Sense::hover(),
            );
            let painter = ui.painter();

            // Background track line
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.top() + track_height * 0.4),
                    egui::vec2(avail_w, 2.0),
                ),
                0.0,
                palette.text_dim.gamma_multiply(0.2),
            );

            // Draw segments belonging to this track
            for seg in segments {
                let obj = match seg.as_object() {
                    Some(o) => o,
                    None => continue,
                };
                let seg_track = obj
                    .get("track")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if seg_track != track_name {
                    continue;
                }
                let start = obj.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let end = obj.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let strand = obj.get("strand").and_then(|v| v.as_str()).unwrap_or("+");
                let seg_label = obj.get("label").and_then(|v| v.as_str()).unwrap_or("");

                let x0 = (start / sequence_length) as f32 * avail_w + rect.left();
                let x1 = (end / sequence_length) as f32 * avail_w + rect.left();
                let w = (x1 - x0).max(2.0);
                let y_top = rect.top();

                if strand == "+" || strand == "-" {
                    // Arrow-shaped segment
                    let arrow_w = w.min(6.0);
                    let mid_y = y_top + track_height / 2.0;
                    let points = if strand == "+" {
                        vec![
                            egui::pos2(x0, y_top),
                            egui::pos2(x0 + w - arrow_w, y_top),
                            egui::pos2(x0 + w, mid_y),
                            egui::pos2(x0 + w - arrow_w, y_top + track_height),
                            egui::pos2(x0, y_top + track_height),
                        ]
                    } else {
                        vec![
                            egui::pos2(x0 + arrow_w, y_top),
                            egui::pos2(x0 + w, y_top),
                            egui::pos2(x0 + w, y_top + track_height),
                            egui::pos2(x0 + arrow_w, y_top + track_height),
                            egui::pos2(x0, mid_y),
                        ]
                    };
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        base_color.gamma_multiply(0.7),
                        egui::Stroke::new(0.5, base_color),
                    ));
                } else {
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(x0, y_top),
                            egui::vec2(w, track_height),
                        ),
                        2.0,
                        base_color.gamma_multiply(0.7),
                    );
                }

                if !seg_label.is_empty() && w > 20.0 {
                    painter.text(
                        egui::pos2(x0 + w / 2.0, y_top + track_height / 2.0),
                        egui::Align2::CENTER_CENTER,
                        seg_label,
                        egui::FontId::proportional(7.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        });
    }
}

/// Draw a circular plasmid/genome map with concentric feature arcs.
#[expect(dead_code, reason = "pre-SceneGraph legacy renderer — SceneGraph pipeline handles circular_map via grammar compilation")]
pub fn draw_circular_map(
    ui: &mut Ui,
    label: &str,
    sequence_length: f64,
    _rings: &[String],
    arcs: &[serde_json::Value],
    unit: &str,
    domain: Option<&str>,
) {
    let palette = domain_theme::palette_for_domain(domain.unwrap_or("health"));
    ui.label(
        RichText::new(format!("{label} ({unit})"))
            .strong()
            .color(palette.text_dim),
    );

    let size = ui.available_width().min(300.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let base_radius: f32 = size * 0.25;
    let ring_spacing: f32 = size * 0.04;

    // Backbone circle
    painter.circle_stroke(
        center,
        base_radius,
        egui::Stroke::new(1.5, palette.info),
    );

    let category_colors = [
        palette.info,
        palette.positive,
        palette.caution,
        egui::Color32::from_rgb(180, 120, 200),
        egui::Color32::from_rgb(200, 160, 80),
    ];

    for arc in arcs {
        let obj = match arc.as_object() {
            Some(o) => o,
            None => continue,
        };
        let start_angle = obj
            .get("start_angle")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let end_angle = obj
            .get("end_angle")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let ring_idx = obj
            .get("ring")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let arc_label = obj
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let r = base_radius + (ring_idx as f32 + 1.0) * ring_spacing;
        let color = category_colors[ring_idx % category_colors.len()];

        // Convert degrees to radians, rotate -90 so 0 is at top
        let start_rad = (start_angle as f32).to_radians() - std::f32::consts::FRAC_PI_2;
        let end_rad = (end_angle as f32).to_radians() - std::f32::consts::FRAC_PI_2;
        let n_samples = ((end_rad - start_rad).abs() * 16.0).ceil().max(4.0) as usize;

        let arc_thickness = ring_spacing * 0.7;
        let r_inner = r - arc_thickness / 2.0;
        let r_outer = r + arc_thickness / 2.0;

        let mut poly = Vec::with_capacity(n_samples * 2 + 2);
        for j in 0..=n_samples {
            let t = j as f32 / n_samples as f32;
            let angle = start_rad + t * (end_rad - start_rad);
            poly.push(egui::pos2(
                center.x + r_outer * angle.cos(),
                center.y + r_outer * angle.sin(),
            ));
        }
        for j in (0..=n_samples).rev() {
            let t = j as f32 / n_samples as f32;
            let angle = start_rad + t * (end_rad - start_rad);
            poly.push(egui::pos2(
                center.x + r_inner * angle.cos(),
                center.y + r_inner * angle.sin(),
            ));
        }

        painter.add(egui::Shape::convex_polygon(
            poly,
            color.gamma_multiply(0.7),
            egui::Stroke::new(0.5, color),
        ));

        // Feature label at midpoint
        if !arc_label.is_empty() && (end_angle - start_angle).abs() > 15.0 {
            let mid_rad = (start_rad + end_rad) / 2.0;
            let label_r = r_outer + 8.0;
            painter.text(
                egui::pos2(
                    center.x + label_r * mid_rad.cos(),
                    center.y + label_r * mid_rad.sin(),
                ),
                egui::Align2::CENTER_CENTER,
                arc_label,
                egui::FontId::proportional(7.0),
                palette.text_dim,
            );
        }
    }

    // Center label
    let bp_label = format_bp(sequence_length);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        format!("{label}\n{bp_label}"),
        egui::FontId::proportional(10.0),
        palette.text_dim,
    );
}

/// Format genomic coordinate as bp / kbp / Mbp.
fn format_bp(bp: f64) -> String {
    if bp >= 1_000_000.0 {
        format!("{:.2} Mbp", bp / 1_000_000.0)
    } else if bp >= 1_000.0 {
        format!("{:.1} kbp", bp / 1_000.0)
    } else {
        format!("{:.0} bp", bp)
    }
}

/// Generate tick positions and labels for a genomic axis.
fn format_genomic_axis(seq_len: f64) -> Vec<(f64, String)> {
    let step = if seq_len >= 1_000_000.0 {
        500_000.0
    } else if seq_len >= 100_000.0 {
        50_000.0
    } else if seq_len >= 10_000.0 {
        5_000.0
    } else {
        1_000.0
    };

    let mut ticks = Vec::new();
    let mut pos = 0.0;
    while pos <= seq_len {
        ticks.push((pos, format_bp(pos)));
        pos += step;
    }
    ticks
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

#[expect(dead_code, reason = "pre-SceneGraph legacy renderer — SceneGraph pipeline handles fieldmap via grammar compilation")]
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

#[expect(dead_code, reason = "pre-SceneGraph legacy renderer — SceneGraph pipeline handles spectrum via grammar compilation")]
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
