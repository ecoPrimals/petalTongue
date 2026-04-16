// SPDX-License-Identifier: AGPL-3.0-or-later
//! System Dashboard - Panel rendering
//!
//! Compact sidebar, full panel, proprioception, and sensory status UI.

use crate::accessibility::ColorPalette;
use crate::audio::AudioSystemV2;
use crate::live_data::request_live_updates;
use crate::sensors::UiSensorRegistry;
use egui::Ui;
use petal_tongue_core::RenderingAwareness;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use super::state::SystemDashboard;

const BYTES_PER_GB: f64 = 1_073_741_824.0;

#[must_use]
fn memory_percent(used: u64, total: u64) -> f64 {
    if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

#[must_use]
fn format_memory_gb(used: u64, total: u64) -> String {
    format!(
        "Used: {:.1} / {:.1} GB",
        used as f64 / BYTES_PER_GB,
        total as f64 / BYTES_PER_GB
    )
}

impl SystemDashboard {
    /// Render compact dashboard (for sidebar)
    pub fn render_compact(
        &mut self,
        ui: &mut Ui,
        palette: &ColorPalette,
        font_scale: f32,
        audio_system: Option<&AudioSystemV2>,
    ) {
        self.refresh(audio_system);

        ui.group(|ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("📊 System")
                    .size(14.0 * font_scale)
                    .strong()
                    .color(palette.text),
            );

            ui.add_space(5.0);

            // CPU
            self.cpu_metric.render(ui);
            self.render_mini_sparkline(ui, &self.cpu_history, palette);

            ui.add_space(3.0);

            // Memory
            self.memory_metric.render(ui);
            self.render_mini_sparkline(ui, &self.mem_history, palette);
        });

        // Request continuous updates
        request_live_updates(ui.ctx());
    }

    /// Render full dashboard (for right panel)
    pub fn render_full(
        &mut self,
        ui: &mut Ui,
        palette: &ColorPalette,
        font_scale: f32,
        audio_system: Option<&AudioSystemV2>,
    ) {
        self.refresh(audio_system);

        ui.heading(
            egui::RichText::new("📊 Live System Metrics")
                .size(18.0 * font_scale)
                .color(palette.text),
        );

        ui.add_space(10.0);

        // CPU Section
        ui.group(|ui| {
            self.cpu_metric.render_large(ui);
            ui.add_space(5.0);

            let cpu_usage = self.stats.cpu_usage();

            // Progress bar
            ui.add(
                egui::ProgressBar::new(cpu_usage / 100.0)
                    .text(format!("{cpu_usage:.1}%"))
                    .fill(if cpu_usage > 90.0 {
                        palette.error
                    } else if cpu_usage > 70.0 {
                        palette.warning
                    } else {
                        palette.healthy
                    }),
            );

            ui.label(format!("Cores: {}", self.stats.cpu_count()));
            self.render_mini_sparkline(ui, &self.cpu_history, palette);
        });

        ui.add_space(10.0);

        // Memory Section
        ui.group(|ui| {
            self.memory_metric.render_large(ui);
            ui.add_space(5.0);

            let used = self.stats.used_memory();
            let total = self.stats.total_memory();
            let percent = memory_percent(used, total);

            ui.label(format_memory_gb(used, total));

            ui.add(
                egui::ProgressBar::new(percent as f32 / 100.0)
                    .text(format!("{percent:.1}%"))
                    .fill(if percent > 90.0 {
                        palette.error
                    } else if percent > 70.0 {
                        palette.warning
                    } else {
                        palette.healthy
                    }),
            );

            self.render_mini_sparkline(ui, &self.mem_history, palette);
        });

        // Request continuous updates
        request_live_updates(ui.ctx());
    }

    /// Render mini sparkline
    fn render_mini_sparkline(&self, ui: &mut Ui, data: &VecDeque<f32>, palette: &ColorPalette) {
        use egui::{Pos2, Stroke};

        if data.is_empty() {
            return;
        }

        let height = 20.0;
        let (response, painter) = ui.allocate_painter(
            egui::vec2(ui.available_width(), height),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        let width = rect.width();

        // Background
        painter.rect_filled(rect, 2.0, palette.background_alt);

        // Calculate points
        let point_count = data.len();
        if point_count < 2 {
            return;
        }

        let x_step = width / (point_count - 1) as f32;
        let mut points = Vec::with_capacity(point_count);

        for (i, &value) in data.iter().enumerate() {
            let x = (i as f32).mul_add(x_step, rect.min.x);
            let y_normalized = value / 100.0; // Assume 0-100% range
            let y = rect.max.y - (y_normalized * height);
            points.push(Pos2::new(x, y));
        }

        // Draw line
        let color = if let Some(&last) = data.back() {
            if last > 90.0 {
                palette.error
            } else if last > 70.0 {
                palette.warning
            } else {
                palette.healthy
            }
        } else {
            palette.accent
        };

        painter.add(egui::Shape::line(points, Stroke::new(1.5, color)));

        // Draw current value indicator
        if let Some(&last_value) = data.back() {
            let last_x = rect.max.x;
            let last_y = (last_value / 100.0).mul_add(-height, rect.max.y);
            painter.circle_filled(Pos2::new(last_x, last_y), 2.0, color);
        }
    }
}

impl SystemDashboard {
    /// Render SAME DAVE proprioception status (v1.1.0)
    pub fn render_proprioception_status(
        ui: &mut Ui,
        palette: &ColorPalette,
        font_scale: f32,
        proprioception: &mut crate::proprioception::ProprioceptionSystem,
    ) {
        use crate::proprioception::ProprioceptiveState;

        ui.group(|ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("🧠 SAME DAVE Proprioception")
                    .size(14.0 * font_scale)
                    .strong()
                    .color(palette.text),
            );

            ui.add_space(5.0);

            // Get current state
            let state: ProprioceptiveState = proprioception.assess();

            // Health percentage with color coding
            let health_pct = state.health * 100.0;
            let health_color = if health_pct > 80.0 {
                egui::Color32::from_rgb(0, 200, 83) // Green
            } else if health_pct > 50.0 {
                palette.warning // Yellow
            } else {
                palette.error // Red
            };

            ui.add(
                egui::ProgressBar::new(state.health)
                    .text(format!("Health: {health_pct:.0}%"))
                    .fill(health_color),
            );

            // Confidence percentage
            let conf_pct = state.confidence * 100.0;
            ui.add(
                egui::ProgressBar::new(state.confidence)
                    .text(format!("Confidence: {conf_pct:.0}%"))
                    .fill(if conf_pct > 70.0 {
                        egui::Color32::from_rgb(100, 149, 237) // Cornflower blue
                    } else {
                        palette.text_dim
                    }),
            );

            ui.add_space(3.0);

            // === v1.2.0: Frame Rate & Hang Detection ===

            // FPS display with color coding
            let fps_color = if state.frame_rate > 30.0 {
                egui::Color32::from_rgb(0, 200, 83) // Green
            } else if state.frame_rate > 15.0 {
                palette.warning // Yellow
            } else {
                palette.error // Red
            };

            ui.label(
                egui::RichText::new(format!(
                    "🎬 {:.1} FPS ({} frames)",
                    state.frame_rate, state.total_frames
                ))
                .size(10.0 * font_scale)
                .color(fps_color),
            );

            // Hang detection warning
            if state.is_hanging {
                ui.label(
                    egui::RichText::new(format!(
                        "⚠️  HANG: {}",
                        state.hang_reason.as_deref().unwrap_or("Unknown")
                    ))
                    .size(10.0 * font_scale)
                    .color(palette.error)
                    .strong(),
                );
            }

            // System status
            let motor_icon = if state.motor_functional { "✅" } else { "❌" };
            let sensory_icon = if state.sensory_functional {
                "✅"
            } else {
                "❌"
            };
            let loop_icon = if state.loop_complete { "✅" } else { "⏳" };

            ui.label(
                egui::RichText::new(format!(
                    "{motor_icon} Motor | {sensory_icon} Sensory | {loop_icon} Loop"
                ))
                .size(10.0 * font_scale)
                .color(palette.text_dim),
            );

            // Get diagnostic summary
            let output_status = proprioception.get_output_status();
            let input_status = proprioception.get_input_status();

            ui.label(
                egui::RichText::new(format!("📤 {output_status}"))
                    .size(9.0 * font_scale)
                    .color(palette.text_dim),
            );

            ui.label(
                egui::RichText::new(format!("📥 {input_status}"))
                    .size(9.0 * font_scale)
                    .color(palette.text_dim),
            );
        });
    }

    /// Render bidirectional sensory status (central nervous system)
    pub fn render_sensory_status(
        ui: &mut Ui,
        palette: &ColorPalette,
        font_scale: f32,
        rendering_awareness: &Arc<RwLock<RenderingAwareness>>,
        sensor_registry: &Arc<RwLock<UiSensorRegistry>>,
    ) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("🧠 Sensory Loop")
                    .size(14.0 * font_scale)
                    .strong()
                    .color(palette.text),
            );

            ui.add_space(5.0);

            // Get self-assessment
            if let Ok(awareness) = rendering_awareness.read() {
                let assessment = awareness.assess_self();
                let metrics = awareness.metrics();

                // Motor function
                let motor_icon = if assessment.can_render { "✅" } else { "❌" };
                ui.label(
                    egui::RichText::new(format!(
                        "{} Render: {}",
                        motor_icon, metrics.commands_sent
                    ))
                    .size(11.0 * font_scale)
                    .color(palette.text),
                );

                // Sensory function
                let sensory_icon = if assessment.can_sense { "✅" } else { "❌" };
                ui.label(
                    egui::RichText::new(format!(
                        "{} Sense: {}",
                        sensory_icon, metrics.frames_confirmed
                    ))
                    .size(11.0 * font_scale)
                    .color(palette.text),
                );

                // Bidirectional loop
                let loop_icon = if assessment.is_complete_loop {
                    "✅"
                } else {
                    "⏳"
                };
                ui.label(
                    egui::RichText::new(format!(
                        "{} Loop: {:.1}%",
                        loop_icon, assessment.confirmation_rate
                    ))
                    .size(11.0 * font_scale)
                    .color(if assessment.is_complete_loop {
                        egui::Color32::from_rgb(0, 200, 83) // Green
                    } else {
                        palette.warning
                    }),
                );

                // User visibility state
                let visibility_text = match assessment.user_visibility {
                    petal_tongue_core::VisibilityState::Confirmed => "👁️ Confirmed",
                    petal_tongue_core::VisibilityState::Probable => "👁️ Probable",
                    petal_tongue_core::VisibilityState::Uncertain => "👁️ Uncertain",
                    petal_tongue_core::VisibilityState::Unknown => "❓ Unknown",
                };
                ui.label(
                    egui::RichText::new(visibility_text)
                        .size(10.0 * font_scale)
                        .color(palette.text_dim),
                );

                // Health percentage
                let health_pct = assessment.health_percentage();
                ui.add(
                    egui::ProgressBar::new(health_pct / 100.0)
                        .text(format!("{health_pct:.0}% Healthy"))
                        .fill(if health_pct > 80.0 {
                            egui::Color32::from_rgb(0, 200, 83) // Green
                        } else if health_pct > 50.0 {
                            palette.warning
                        } else {
                            palette.error
                        }),
                );
            } else {
                ui.label(
                    egui::RichText::new("⚠️ Awareness unavailable")
                        .size(11.0 * font_scale)
                        .color(palette.warning),
                );
            }

            ui.add_space(5.0);

            // Discovered sensors
            if let Ok(reg) = sensor_registry.read() {
                let stats = reg.stats();
                ui.label(
                    egui::RichText::new(format!("🔍 Sensors: {}/{}", stats.active, stats.total))
                        .size(10.0 * font_scale)
                        .color(palette.text_dim),
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_percent_zero_total() {
        assert!((memory_percent(100, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn memory_percent_half() {
        assert!((memory_percent(512, 1024) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn memory_percent_full() {
        assert!((memory_percent(1024, 1024) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn format_memory_gb_display() {
        let used = 2 * 1_073_741_824;
        let total = 8 * 1_073_741_824;
        let s = format_memory_gb(used, total);
        assert!(s.contains("2.0"));
        assert!(s.contains("8.0"));
        assert!(s.starts_with("Used:"));
    }

    #[test]
    fn memory_percent_small_values() {
        assert!((memory_percent(1, 100) - 1.0).abs() < f64::EPSILON);
        assert!((memory_percent(0, 1000) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn format_memory_gb_zero() {
        let s = format_memory_gb(0, 0);
        assert!(s.contains("0.0"));
    }

    #[test]
    fn memory_percent_large_values() {
        let used = 8 * 1_073_741_824;
        let total = 16 * 1_073_741_824;
        assert!((memory_percent(used, total) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn format_memory_gb_fractional() {
        let used = 1_073_741_824 / 2;
        let total = 2 * 1_073_741_824;
        let s = format_memory_gb(used, total);
        assert!(s.contains("Used:"));
        assert!(s.contains("GB"));
    }

    #[test]
    fn memory_percent_near_full() {
        assert!((memory_percent(99, 100) - 99.0).abs() < f64::EPSILON);
    }

    /// Headless egui: `render_compact` runs without panic
    #[test]
    fn render_compact_headless() {
        use crate::accessibility::ColorScheme;

        let mut dashboard = SystemDashboard::default();
        let palette = ColorPalette::from_scheme(ColorScheme::Default);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                dashboard.render_compact(ui, &palette, 1.0, None);
            });
        });
    }

    /// Headless egui: `render_full` runs without panic
    #[test]
    fn render_full_headless() {
        use crate::accessibility::ColorScheme;

        let mut dashboard = SystemDashboard::default();
        let palette = ColorPalette::from_scheme(ColorScheme::Default);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                dashboard.render_full(ui, &palette, 1.0, None);
            });
        });
    }

    /// Headless egui: `render_proprioception_status` runs without panic
    #[test]
    fn render_proprioception_status_headless() {
        use crate::accessibility::ColorScheme;
        use crate::proprioception::ProprioceptionSystem;

        let mut proprioception = ProprioceptionSystem::new();
        let palette = ColorPalette::from_scheme(ColorScheme::Default);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                SystemDashboard::render_proprioception_status(
                    ui,
                    &palette,
                    1.0,
                    &mut proprioception,
                );
            });
        });
    }

    /// Headless egui: `render_sensory_status` runs without panic
    #[test]
    fn render_sensory_status_headless() {
        use crate::accessibility::ColorScheme;

        let palette = ColorPalette::from_scheme(ColorScheme::Default);
        let rendering_awareness = Arc::new(RwLock::new(RenderingAwareness::new()));
        let sensor_registry = Arc::new(RwLock::new(UiSensorRegistry::new()));

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                SystemDashboard::render_sensory_status(
                    ui,
                    &palette,
                    1.0,
                    &rendering_awareness,
                    &sensor_registry,
                );
            });
        });
    }
}
