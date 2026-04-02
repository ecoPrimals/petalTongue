// SPDX-License-Identifier: AGPL-3.0-or-later
//! egui renderer for `DataBinding::Soundscape`.
//!
//! Visualizes soundscape layers as waveform previews with frequency,
//! amplitude, pan position, and duration indicators. Uses the synthesis
//! engine in `petal_tongue_scene::soundscape` to generate preview curves.

use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, pos2, vec2};
use petal_tongue_scene::soundscape::{SoundLayer, Soundscape, Waveform};

const WAVEFORM_PREVIEW_SAMPLES: usize = 128;
const LAYER_ROW_HEIGHT: f32 = 60.0;
const PAN_INDICATOR_RADIUS: f32 = 20.0;

/// Draw a soundscape definition parsed from a `DataBinding::Soundscape` JSON value.
///
/// Renders one row per sound layer with waveform preview, frequency/amplitude
/// indicators, and a stereo pan field.
pub fn draw_soundscape(ui: &mut Ui, label: &str, definition: &serde_json::Value) {
    let soundscape: Soundscape = match serde_json::from_value(definition.clone()) {
        Ok(s) => s,
        Err(e) => {
            ui.label(format!("🔊 {label} (invalid soundscape: {e})"));
            return;
        }
    };

    ui.group(|ui| {
        ui.label(
            egui::RichText::new(format!("🔊 {}", soundscape.name))
                .strong()
                .size(14.0),
        );
        ui.label(format!(
            "{:.1}s · {} layers · master {:.0}%",
            soundscape.duration_secs,
            soundscape.layers.len(),
            soundscape.master_amplitude * 100.0,
        ));
        ui.add_space(4.0);

        for layer in &soundscape.layers {
            draw_layer_row(ui, layer, soundscape.duration_secs);
            ui.add_space(2.0);
        }

        if soundscape.layers.len() > 1 {
            draw_stereo_field(ui, &soundscape.layers);
        }
    });
}

const fn waveform_label(w: Waveform) -> &'static str {
    match w {
        Waveform::Sine => "sine",
        Waveform::Square => "square",
        Waveform::Sawtooth => "saw",
        Waveform::Triangle => "tri",
        Waveform::WhiteNoise => "noise",
    }
}

const fn waveform_color(w: Waveform) -> Color32 {
    match w {
        Waveform::Sine => Color32::from_rgb(80, 180, 255),
        Waveform::Square => Color32::from_rgb(255, 140, 60),
        Waveform::Sawtooth => Color32::from_rgb(180, 80, 255),
        Waveform::Triangle => Color32::from_rgb(80, 220, 120),
        Waveform::WhiteNoise => Color32::from_rgb(180, 180, 180),
    }
}

fn draw_layer_row(ui: &mut Ui, layer: &SoundLayer, total_duration: f64) {
    let available_width = ui.available_width().max(200.0);
    let (response, painter) =
        ui.allocate_painter(vec2(available_width, LAYER_ROW_HEIGHT), Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, Rounding::same(3.0), Color32::from_gray(25));

    let info_width = 110.0_f32;
    let waveform_rect = Rect::from_min_max(
        rect.left_top() + vec2(info_width, 4.0),
        rect.right_bottom() - vec2(4.0, 4.0),
    );

    let color = waveform_color(layer.waveform);

    paint_waveform_preview(&painter, waveform_rect, layer, total_duration, color);

    painter.text(
        rect.left_top() + vec2(6.0, 4.0),
        Align2::LEFT_TOP,
        &layer.id,
        FontId::proportional(11.0),
        Color32::WHITE,
    );
    painter.text(
        rect.left_top() + vec2(6.0, 18.0),
        Align2::LEFT_TOP,
        waveform_label(layer.waveform),
        FontId::monospace(10.0),
        color,
    );
    painter.text(
        rect.left_top() + vec2(6.0, 31.0),
        Align2::LEFT_TOP,
        format!("{:.0} Hz", layer.frequency),
        FontId::monospace(10.0),
        Color32::LIGHT_GRAY,
    );

    let amp_bar_x = 6.0;
    let amp_bar_y = 44.0;
    let amp_bar_w = 80.0_f32;
    let amp_bar_h = 6.0_f32;
    let bg = Rect::from_min_size(
        rect.left_top() + vec2(amp_bar_x, amp_bar_y),
        vec2(amp_bar_w, amp_bar_h),
    );
    painter.rect_filled(bg, Rounding::same(2.0), Color32::from_gray(50));
    let fill = Rect::from_min_size(
        bg.left_top(),
        vec2(
            amp_bar_w * layer.amplitude.clamp(0.0, 1.0) as f32,
            amp_bar_h,
        ),
    );
    painter.rect_filled(fill, Rounding::same(2.0), color);

    let pan_x = 6.0 + amp_bar_w + 4.0;
    let pan_indicator = rect.left_top() + vec2(pan_x, amp_bar_y + 3.0);
    let pan_pos = pan_indicator + vec2(8.0 * layer.pan.clamp(-1.0, 1.0) as f32, 0.0);
    painter.text(
        pan_indicator - vec2(0.0, 1.0),
        Align2::LEFT_CENTER,
        "L",
        FontId::monospace(8.0),
        Color32::DARK_GRAY,
    );
    painter.circle_filled(Pos2::new(pan_pos.x, pan_pos.y), 3.0, color);
}

fn paint_waveform_preview(
    painter: &egui::Painter,
    rect: Rect,
    layer: &SoundLayer,
    total_duration: f64,
    color: Color32,
) {
    let width = rect.width();
    let height = rect.height();
    let mid_y = rect.center().y;

    painter.rect_stroke(
        rect,
        Rounding::same(2.0),
        Stroke::new(0.5, Color32::from_gray(60)),
    );
    painter.line_segment(
        [pos2(rect.left(), mid_y), pos2(rect.right(), mid_y)],
        Stroke::new(0.5, Color32::from_gray(40)),
    );

    if total_duration > 0.0 {
        let start_frac = (layer.offset_secs / total_duration).clamp(0.0, 1.0) as f32;
        let end_frac =
            ((layer.offset_secs + layer.duration_secs) / total_duration).clamp(0.0, 1.0) as f32;
        let x_start = rect.left() + width * start_frac;
        let x_end = rect.left() + width * end_frac;
        let active_rect = Rect::from_min_max(pos2(x_start, rect.top()), pos2(x_end, rect.bottom()));
        painter.rect_filled(active_rect, Rounding::ZERO, color.gamma_multiply(0.08));
    }

    let cycles = 3.0_f64;
    let points: Vec<Pos2> = (0..WAVEFORM_PREVIEW_SAMPLES)
        .map(|i| {
            let frac = i as f64 / WAVEFORM_PREVIEW_SAMPLES as f64;
            let phase = frac * cycles;
            let sample = layer.waveform.sample(phase, i as u64);
            let amplitude = layer.amplitude.clamp(0.0, 1.0);
            let py = mid_y - (sample * amplitude * f64::from(height) * 0.4) as f32;
            let px = rect.left() + width * frac as f32;
            pos2(px, py)
        })
        .collect();

    for pair in points.windows(2) {
        painter.line_segment([pair[0], pair[1]], Stroke::new(1.5, color));
    }
}

fn draw_stereo_field(ui: &mut Ui, layers: &[SoundLayer]) {
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Stereo Field")
            .size(11.0)
            .color(Color32::GRAY),
    );
    let size = vec2(PAN_INDICATOR_RADIUS * 6.0, PAN_INDICATOR_RADIUS * 3.0);
    let (response, painter) = ui.allocate_painter(size, Sense::hover());
    let rect = response.rect;
    let center = rect.center();

    painter.rect_filled(rect, Rounding::same(4.0), Color32::from_gray(20));
    painter.line_segment(
        [
            pos2(center.x, rect.top() + 4.0),
            pos2(center.x, rect.bottom() - 4.0),
        ],
        Stroke::new(0.5, Color32::from_gray(60)),
    );
    painter.text(
        pos2(rect.left() + 4.0, center.y),
        Align2::LEFT_CENTER,
        "L",
        FontId::monospace(10.0),
        Color32::from_gray(80),
    );
    painter.text(
        pos2(rect.right() - 4.0, center.y),
        Align2::RIGHT_CENTER,
        "R",
        FontId::monospace(10.0),
        Color32::from_gray(80),
    );

    let field_w = rect.width() - 30.0;
    for layer in layers {
        let pan_x = center.x + (layer.pan.clamp(-1.0, 1.0) as f32) * field_w * 0.5;
        let amp_r = 3.0 + layer.amplitude.clamp(0.0, 1.0) as f32 * 6.0;
        let color = waveform_color(layer.waveform);
        painter.circle_filled(pos2(pan_x, center.y), amp_r, color.gamma_multiply(0.7));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_in_egui(mut f: impl FnMut(&mut Ui)) {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| f(ui));
        });
    }

    #[test]
    fn draw_empty_soundscape() {
        let def = serde_json::json!({
            "name": "Silence",
            "duration_secs": 5.0,
            "layers": []
        });
        run_in_egui(|ui| draw_soundscape(ui, "test", &def));
    }

    #[test]
    fn draw_multi_layer_soundscape() {
        let def = serde_json::json!({
            "name": "Forest Ambience",
            "duration_secs": 30.0,
            "layers": [
                {
                    "id": "wind",
                    "waveform": "white_noise",
                    "frequency": 200.0,
                    "amplitude": 0.3,
                    "duration_secs": 30.0,
                    "pan": -0.5,
                    "fade_in_secs": 2.0
                },
                {
                    "id": "birdsong",
                    "waveform": "sine",
                    "frequency": 800.0,
                    "amplitude": 0.6,
                    "duration_secs": 10.0,
                    "pan": 0.7,
                    "offset_secs": 5.0
                },
                {
                    "id": "creek",
                    "waveform": "triangle",
                    "frequency": 400.0,
                    "amplitude": 0.4,
                    "duration_secs": 30.0,
                    "pan": 0.0
                }
            ],
            "master_amplitude": 0.8
        });
        run_in_egui(|ui| draw_soundscape(ui, "Forest", &def));
    }

    #[test]
    fn draw_invalid_soundscape_degrades() {
        run_in_egui(|ui| {
            draw_soundscape(ui, "Bad", &serde_json::json!("not a soundscape"));
        });
    }

    #[test]
    fn waveform_colors_are_distinct() {
        let colors: Vec<Color32> = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
            Waveform::WhiteNoise,
        ]
        .iter()
        .map(|w| waveform_color(*w))
        .collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }
}
