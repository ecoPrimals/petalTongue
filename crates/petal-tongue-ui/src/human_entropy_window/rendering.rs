// SPDX-License-Identifier: AGPL-3.0-or-later
//! Human entropy window UI rendering

use super::rendering_helpers::{
    capture_state_display, format_recording_duration, modality_selector_enabled, quality_color_rgb,
    quality_to_percent_display, stopped_state_message,
};
use super::state::HumanEntropyWindow;
use super::types::{CaptureWindowState, EntropyModality};
use eframe::egui;

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
        let display = capture_state_display(&self.state, self.current_quality, None);
        ui.label("Choose Modality:");

        ui.horizontal(|ui| {
            for modality in [
                EntropyModality::Audio,
                EntropyModality::Visual,
                EntropyModality::Narrative,
                EntropyModality::Gesture,
                EntropyModality::Video,
            ] {
                let enabled = modality_selector_enabled(modality.is_available(), display.can_start);

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
        let display = capture_state_display(&self.state, self.current_quality, None);
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            ui.add_enabled_ui(display.can_start, |ui| {
                if ui.button(format!("🎙️ {}", display.action_label)).clicked() {
                    self.start_capture();
                }
            });

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

                ui.colored_label(color, quality_to_percent_display(quality));
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
        let display = capture_state_display(&self.state, self.current_quality, None);
        ui.vertical_centered(|ui| {
            ui.add_enabled_ui(display.can_stop, |ui| {
                if ui.button(format!("⏹️ {}", display.action_label)).clicked() {
                    self.stop_capture();
                }
            });
        });
    }

    fn render_stopped_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            ui.label(stopped_state_message(self.current_quality));

            ui.add_space(20.0);

            let display = capture_state_display(&self.state, self.current_quality, None);
            ui.horizontal(|ui| {
                if ui.button(format!("✅ {}", display.action_label)).clicked() {
                    self.finalize_and_stream();
                }

                ui.add_enabled_ui(display.can_discard, |ui| {
                    if ui.button("🗑️ Discard").clicked() {
                        self.discard();
                    }
                });
            });
        });
    }

    fn render_processing_state(&self, ui: &mut egui::Ui) {
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
    use super::super::rendering_helpers::{
        capture_state_display, format_recording_duration, modality_selector_enabled,
        quality_color_rgb, quality_to_percent_display, stopped_state_message,
    };
    use super::super::state::HumanEntropyWindow;
    use super::super::types::{CaptureWindowState, EntropyModality};

    #[test]
    fn format_recording_duration_values() {
        assert_eq!(format_recording_duration(0.0), "0.0s");
        assert_eq!(format_recording_duration(1.0), "1.0s");
        assert_eq!(format_recording_duration(65.5), "65.5s");
    }

    #[test]
    fn quality_color_rgb_green() {
        assert_eq!(quality_color_rgb(0.8), [0, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_yellow() {
        assert_eq!(quality_color_rgb(0.5), [255, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_red() {
        assert_eq!(quality_color_rgb(0.2), [255, 0, 0]);
    }

    #[test]
    fn capture_state_display_idle() {
        let d = capture_state_display(&CaptureWindowState::Idle, None, None);
        assert_eq!(d.state_label, "Idle");
        assert!(d.can_start);
        assert!(!d.can_stop);
    }

    #[test]
    fn capture_state_display_recording() {
        let d = capture_state_display(&CaptureWindowState::Recording, Some(0.9), Some(10.0));
        assert_eq!(d.state_label, "Recording");
        assert!(!d.can_start);
        assert!(d.can_stop);
        assert_eq!(d.progress_percent, Some(90.0));
    }

    #[test]
    fn capture_state_display_stopped() {
        let d = capture_state_display(&CaptureWindowState::Stopped, Some(0.6), None);
        assert_eq!(d.state_label, "Stopped");
        assert!(d.can_discard);
    }

    #[test]
    fn capture_state_display_processing() {
        let d = capture_state_display(&CaptureWindowState::Processing, None, None);
        assert_eq!(d.state_label, "Processing");
        assert!(!d.can_start);
        assert!(!d.can_discard);
    }

    #[test]
    fn modality_selector_enabled_both_true() {
        assert!(modality_selector_enabled(true, true));
    }

    #[test]
    fn modality_selector_enabled_unavailable() {
        assert!(!modality_selector_enabled(false, true));
    }

    #[test]
    fn modality_selector_enabled_cannot_start() {
        assert!(!modality_selector_enabled(true, false));
    }

    #[test]
    fn test_quality_to_percent_display_values() {
        assert_eq!(quality_to_percent_display(0.0), "0.0%");
        assert_eq!(quality_to_percent_display(0.5), "50.0%");
        assert_eq!(quality_to_percent_display(1.0), "100.0%");
    }

    #[test]
    fn stopped_state_message_with_quality() {
        let msg = stopped_state_message(Some(0.85));
        assert!(msg.contains("85.0"));
    }

    #[test]
    fn stopped_state_message_without_quality() {
        assert_eq!(stopped_state_message(None), "Capture complete!");
    }

    #[test]
    fn test_render_ui_idle() {
        let mut window = HumanEntropyWindow::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                window.render_ui(ui);
            });
        });
    }

    #[test]
    fn test_render_ui_recording() {
        let mut window = HumanEntropyWindow::new();
        window.modality = EntropyModality::Narrative;
        window.start_capture();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                window.render_ui(ui);
            });
        });
    }

    #[test]
    fn test_render_ui_stopped() {
        let mut window = HumanEntropyWindow::new();
        window.modality = EntropyModality::Narrative;
        window.start_capture();
        window.stop_capture();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                window.render_ui(ui);
            });
        });
    }

    #[test]
    fn test_render_ui_processing() {
        let mut window = HumanEntropyWindow::new();
        window.state = CaptureWindowState::Processing;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                window.render_ui(ui);
            });
        });
    }

    #[test]
    fn test_show_window() {
        let mut window = HumanEntropyWindow::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let open = window.show(ctx);
            assert!(open);
        });
    }
}
