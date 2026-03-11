// SPDX-License-Identifier: AGPL-3.0-only
//! Human entropy window UI rendering

#![allow(unexpected_cfgs)]

use super::state::HumanEntropyWindow;
use super::types::{CaptureWindowState, EntropyModality};
use eframe::egui;

/// Map quality (0.0–1.0) to RGB color: green (good), yellow (medium), red (poor).
#[must_use]
pub fn quality_color_rgb(quality: f32) -> [u8; 3] {
    if quality > 0.7 {
        [0, 255, 0] // green
    } else if quality > 0.4 {
        [255, 255, 0] // yellow
    } else {
        [255, 0, 0] // red
    }
}

/// Format recording duration in seconds as human-readable string.
#[must_use]
pub fn format_recording_duration(elapsed_secs: f64) -> String {
    format!("{elapsed_secs:.1}s")
}

impl HumanEntropyWindow {
    /// Render the window
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;

        egui::Window::new("🌸 Human Entropy Capture")
            .default_width(600.0)
            .default_height(500.0)
            .open(&mut open)
            .show(ctx, |ui| {
                self.render_ui(ui);
            });

        open
    }

    pub(super) fn render_ui(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.heading("Capture Human Entropy for Sovereign Keys");
        ui.add_space(10.0);

        // Privacy notice
        ui.horizontal(|ui| {
            ui.label("🔒");
            ui.label("Privacy: Stream-only (never persisted), encrypted transmission, secure zeroization");
        });
        ui.add_space(10.0);

        // Modality selector
        self.render_modality_selector(ui);
        ui.add_space(10.0);

        // Capture area
        match self.state {
            CaptureWindowState::Idle => {
                self.render_idle_state(ui);
            }
            CaptureWindowState::Recording => {
                self.render_recording_state(ui);
            }
            CaptureWindowState::Stopped => {
                self.render_stopped_state(ui);
            }
            CaptureWindowState::Processing => {
                self.render_processing_state(ui);
            }
        }

        ui.add_space(10.0);

        // Status bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.label(&self.status_message);
        });
    }

    fn render_modality_selector(&mut self, ui: &mut egui::Ui) {
        ui.label("Choose Modality:");

        ui.horizontal(|ui| {
            for modality in [
                EntropyModality::Audio,
                EntropyModality::Visual,
                EntropyModality::Narrative,
                EntropyModality::Gesture,
                EntropyModality::Video,
            ] {
                let enabled = modality.is_available() && self.state == CaptureWindowState::Idle;

                ui.add_enabled_ui(enabled, |ui| {
                    if ui
                        .selectable_label(self.modality == modality, modality.name())
                        .clicked()
                    {
                        self.modality = modality;
                        self.status_message = format!("Selected: {}", modality.name());
                    }
                });

                if !modality.is_available() && ui.is_enabled() {
                    ui.label("(unavailable)");
                }
            }
        });

        ui.add_space(5.0);
        ui.label(self.modality.description());
    }

    fn render_idle_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            if ui.button("🎙️ Start Capture").clicked() {
                self.start_capture();
            }

            ui.add_space(10.0);
            ui.label("Click to begin capturing entropy");
        });
    }

    fn render_recording_state(&mut self, ui: &mut egui::Ui) {
        // Show duration
        if let Some(start) = self.capture_start {
            let duration = std::time::Instant::now().duration_since(start);
            ui.horizontal(|ui| {
                ui.label("Recording:");
                ui.label(format_recording_duration(duration.as_secs_f64()));
            });
        }

        ui.add_space(10.0);

        // Show real-time quality (if available)
        if let Some(quality) = self.current_quality {
            ui.horizontal(|ui| {
                ui.label("Quality:");

                let [r, g, b] = quality_color_rgb(quality as f32);
                let color = egui::Color32::from_rgb(r, g, b);

                ui.colored_label(color, format!("{:.1}%", quality * 100.0));
            });

            // Quality bar
            let progress_bar = egui::ProgressBar::new(quality as f32)
                .desired_width(400.0)
                .show_percentage();
            ui.add(progress_bar);
        }

        ui.add_space(10.0);

        ui.add_space(20.0);

        // Stop button
        ui.vertical_centered(|ui| {
            if ui.button("⏹️ Stop Capture").clicked() {
                self.stop_capture();
            }
        });
    }

    fn render_stopped_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            if let Some(quality) = self.current_quality {
                ui.label(format!(
                    "Capture complete! Quality: {:.1}%",
                    quality * 100.0
                ));
            } else {
                ui.label("Capture complete!");
            }

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("✅ Send to Entropy Source").clicked() {
                    self.finalize_and_stream();
                }

                if ui.button("🗑️ Discard").clicked() {
                    self.discard();
                }
            });
        });
    }

    fn render_processing_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.spinner();
            ui.label("Processing and streaming entropy...");
            ui.add_space(10.0);
            ui.label("🔒 Encrypted transmission to entropy source");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_color_rgb_good() {
        let rgb = quality_color_rgb(0.8f32);
        assert_eq!(rgb, [0, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_medium() {
        let rgb = quality_color_rgb(0.5f32);
        assert_eq!(rgb, [255, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_poor() {
        let rgb = quality_color_rgb(0.3f32);
        assert_eq!(rgb, [255, 0, 0]);
    }

    #[test]
    fn quality_color_rgb_boundary_07() {
        let rgb = quality_color_rgb(0.71f32);
        assert_eq!(rgb, [0, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_boundary_04() {
        let rgb = quality_color_rgb(0.41f32);
        assert_eq!(rgb, [255, 255, 0]);
    }

    #[test]
    fn test_format_recording_duration() {
        assert_eq!(super::format_recording_duration(0.0), "0.0s");
        assert_eq!(super::format_recording_duration(42.5), "42.5s");
        assert_eq!(super::format_recording_duration(123.456), "123.5s");
    }
}
