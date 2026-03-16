// SPDX-License-Identifier: AGPL-3.0-or-later
//! Accessibility Settings Panel UI
//!
//! Panel for users to customize accessibility settings

use super::accessibility::{AccessibilitySettings, ColorPalette, ColorScheme, FontSize};
use egui::{Context, Ui, Window};

// ============================================================================
// Pure formatting functions (testable, no egui)
// ============================================================================

/// Format font size multiplier for display
#[must_use]
pub(crate) fn format_font_multiplier_display(multiplier: f32) -> String {
    format!("  → Current multiplier: {multiplier:.1}x")
}

/// Accessibility settings panel
#[derive(Default)]
pub struct AccessibilityPanel {
    /// Settings
    pub settings: AccessibilitySettings,
    /// Show panel
    pub show: bool,
}

impl AccessibilityPanel {
    /// Show the accessibility panel
    pub fn show(&mut self, ctx: &Context) {
        if !self.show {
            return;
        }

        Window::new("♿ Accessibility Settings")
            .default_width(500.0)
            .collapsible(true)
            .show(ctx, |ui| {
                self.render(ui);
            });
    }

    /// Render the panel contents
    fn render(&mut self, ui: &mut Ui) {
        ui.heading("Universal Accessibility");
        ui.label("petalTongue works for EVERYONE");
        ui.separator();

        // Color Scheme Section
        ui.heading("🎨 Color Scheme");
        ui.label("Choose colors that work best for you:");

        egui::ComboBox::from_label("Color Scheme")
            .selected_text(self.settings.color_scheme.name())
            .show_ui(ui, |ui| {
                for scheme in ColorScheme::all() {
                    ui.selectable_value(&mut self.settings.color_scheme, *scheme, scheme.name());
                }
            });

        // Show color preview
        ui.horizontal(|ui| {
            ui.label("Preview:");
            let palette = ColorPalette::from_scheme(self.settings.color_scheme);

            let size = egui::vec2(30.0, 20.0);
            ui.allocate_ui_with_layout(
                egui::vec2(400.0, 30.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    Self::color_square(ui, palette.healthy, "Healthy", size);
                    Self::color_square(ui, palette.warning, "Warning", size);
                    Self::color_square(ui, palette.error, "Error", size);
                    Self::color_square(ui, palette.accent, "Accent", size);
                },
            );
        });

        ui.add_space(10.0);

        // High Contrast Toggle
        ui.checkbox(&mut self.settings.high_contrast, "High Contrast Mode");
        ui.label("  → WCAG AAA compliant for low vision");

        ui.add_space(10.0);
        ui.separator();

        // Font Size Section
        ui.heading("🔤 Font Size");
        ui.label("Adjust text size for readability:");

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.settings.font_size, FontSize::Small, "Small");
            ui.selectable_value(&mut self.settings.font_size, FontSize::Medium, "Medium");
            ui.selectable_value(&mut self.settings.font_size, FontSize::Large, "Large");
            ui.selectable_value(
                &mut self.settings.font_size,
                FontSize::ExtraLarge,
                "X-Large",
            );
        });

        ui.label(format_font_multiplier_display(
            self.settings.font_size.multiplier(),
        ));

        ui.add_space(10.0);
        ui.separator();

        // Audio Section
        ui.heading("🔊 Audio");
        ui.label("Audio feedback and sonification:");

        ui.checkbox(
            &mut self.settings.audio_enabled,
            "Enable Audio Sonification",
        );
        ui.label("  → Convert visual data to sound (for blind users)");

        if self.settings.audio_enabled {
            ui.add_space(5.0);
            ui.label("Volume:");
            ui.add(
                egui::Slider::new(&mut self.settings.audio_volume, 0.0..=1.0)
                    .text("Volume")
                    .clamping(egui::SliderClamping::Always),
            );
        }

        ui.add_space(5.0);
        ui.add_enabled_ui(false, |ui| {
            ui.checkbox(
                &mut self.settings.narration_enabled,
                "Enable Text-to-Speech Narration (not yet available)",
            );
        });
        ui.label("  → Speak UI elements and status updates");

        ui.add_space(10.0);
        ui.separator();

        // Input Methods Section
        ui.heading("⌨️ Input Methods");
        ui.label("Customize how you interact:");

        ui.checkbox(&mut self.settings.keyboard_only, "Keyboard-Only Mode");
        ui.label("  → Optimize for keyboard navigation (motor disabilities)");

        ui.add_enabled_ui(false, |ui| {
            ui.checkbox(
                &mut self.settings.screen_reader_mode,
                "Screen Reader Mode (not yet available)",
            );
        });
        ui.label("  → Additional announcements for NVDA/JAWS/VoiceOver");

        ui.add_space(10.0);
        ui.separator();

        // Motion Section
        ui.heading("🌀 Motion");
        ui.label("Reduce animations (vestibular disorders):");

        ui.checkbox(&mut self.settings.reduced_motion, "Reduced Motion");
        ui.label("  → Minimize animations and transitions");

        ui.add_space(10.0);
        ui.separator();

        // Action Buttons
        ui.horizontal(|ui| {
            if ui.button("✅ Apply Settings").clicked() {
                // Settings are already applied in real-time
                // This just closes the panel
                self.show = false;
            }

            if ui.button("🔄 Reset to Defaults").clicked() {
                self.settings = AccessibilitySettings::default();
            }

            if ui.button("❌ Close").clicked() {
                self.show = false;
            }
        });

        ui.add_space(10.0);

        // Info footer
        ui.separator();
        ui.label(
            egui::RichText::new("🌸 petalTongue: Accessible to EVERYONE")
                .size(12.0)
                .color(egui::Color32::GRAY),
        );
    }

    /// Render a color preview square
    fn color_square(ui: &mut Ui, color: egui::Color32, label: &str, size: egui::Vec2) {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
        ui.painter().rect_filled(rect, 2.0, color);

        response.on_hover_text(label);
    }

    /// Get the current color palette
    #[must_use]
    pub const fn get_palette(&self) -> ColorPalette {
        ColorPalette::from_scheme(self.settings.color_scheme)
    }

    /// Apply font size to text
    #[must_use]
    pub fn scale_font(&self, base_size: f32) -> f32 {
        base_size * self.settings.font_size.multiplier()
    }

    /// Toggle panel visibility
    pub const fn toggle(&mut self) {
        self.show = !self.show;
    }

    /// Close the panel
    pub const fn close(&mut self) {
        self.show = false;
    }

    /// Select color scheme by index (0-6)
    pub fn select_color_scheme_by_index(&mut self, index: usize) {
        let schemes = [
            ColorScheme::Default,
            ColorScheme::HighContrast,
            ColorScheme::Deuteranopia,
            ColorScheme::Protanopia,
            ColorScheme::Tritanopia,
            ColorScheme::Dark,
            ColorScheme::Light,
        ];

        if let Some(scheme) = schemes.get(index) {
            self.settings.color_scheme = *scheme;
            // Palette is regenerated on next get_palette() call
        }
    }

    /// Increase font size
    pub const fn increase_font_size(&mut self) {
        self.settings.font_size = match self.settings.font_size {
            FontSize::Small => FontSize::Medium,
            FontSize::Medium => FontSize::Large,
            FontSize::Large | FontSize::ExtraLarge => FontSize::ExtraLarge, // Already at max
        };
    }

    /// Decrease font size
    pub const fn decrease_font_size(&mut self) {
        self.settings.font_size = match self.settings.font_size {
            FontSize::Small | FontSize::Medium => FontSize::Small, // Already at min
            FontSize::Large => FontSize::Medium,
            FontSize::ExtraLarge => FontSize::Large,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_panel() {
        let panel = AccessibilityPanel::default();
        assert!(!panel.show);
        assert_eq!(panel.settings.color_scheme, ColorScheme::Default);
        assert_eq!(panel.settings.font_size, FontSize::Medium);
    }

    #[test]
    fn test_font_scaling() {
        let mut panel = AccessibilityPanel::default();
        let base_size = 16.0;

        panel.settings.font_size = FontSize::Small;
        assert_eq!(panel.scale_font(base_size), 13.6);

        panel.settings.font_size = FontSize::Large;
        assert_eq!(panel.scale_font(base_size), 20.8);
    }

    #[test]
    fn test_font_scaling_all_sizes() {
        let mut panel = AccessibilityPanel::default();
        let base = 10.0;

        panel.settings.font_size = FontSize::Small;
        assert!((panel.scale_font(base) - 8.5).abs() < 0.01);

        panel.settings.font_size = FontSize::Medium;
        assert_eq!(panel.scale_font(base), 10.0);

        panel.settings.font_size = FontSize::Large;
        assert_eq!(panel.scale_font(base), 13.0);

        panel.settings.font_size = FontSize::ExtraLarge;
        assert_eq!(panel.scale_font(base), 16.0);
    }

    #[test]
    fn test_toggle() {
        let mut panel = AccessibilityPanel::default();
        assert!(!panel.show);
        panel.toggle();
        assert!(panel.show);
        panel.toggle();
        assert!(!panel.show);
    }

    #[test]
    fn test_close() {
        let mut panel = AccessibilityPanel::default();
        panel.show = true;
        panel.close();
        assert!(!panel.show);
    }

    #[test]
    fn test_get_palette() {
        let panel = AccessibilityPanel::default();
        let palette = panel.get_palette();
        assert_ne!(palette.healthy, palette.error);
        assert_ne!(palette.background, palette.text);
    }

    #[test]
    fn test_select_color_scheme_by_index() {
        let mut panel = AccessibilityPanel::default();

        for (i, expected) in ColorScheme::all().iter().enumerate() {
            panel.select_color_scheme_by_index(i);
            assert_eq!(panel.settings.color_scheme, *expected);
        }
    }

    #[test]
    fn test_select_color_scheme_invalid_index() {
        let mut panel = AccessibilityPanel::default();
        let original = panel.settings.color_scheme;
        panel.select_color_scheme_by_index(99);
        // Invalid index should not panic; scheme may stay unchanged
        assert_eq!(panel.settings.color_scheme, original);
    }

    #[test]
    fn test_increase_font_size() {
        let mut panel = AccessibilityPanel::default();

        panel.settings.font_size = FontSize::Small;
        panel.increase_font_size();
        assert_eq!(panel.settings.font_size, FontSize::Medium);

        panel.increase_font_size();
        assert_eq!(panel.settings.font_size, FontSize::Large);

        panel.increase_font_size();
        assert_eq!(panel.settings.font_size, FontSize::ExtraLarge);

        panel.increase_font_size();
        assert_eq!(panel.settings.font_size, FontSize::ExtraLarge);
    }

    #[test]
    fn test_decrease_font_size() {
        let mut panel = AccessibilityPanel::default();

        panel.settings.font_size = FontSize::ExtraLarge;
        panel.decrease_font_size();
        assert_eq!(panel.settings.font_size, FontSize::Large);

        panel.decrease_font_size();
        assert_eq!(panel.settings.font_size, FontSize::Medium);

        panel.decrease_font_size();
        assert_eq!(panel.settings.font_size, FontSize::Small);

        panel.decrease_font_size();
        assert_eq!(panel.settings.font_size, FontSize::Small);
    }

    #[test]
    fn test_get_palette_reflects_scheme_change() {
        let mut panel = AccessibilityPanel::default();
        panel.select_color_scheme_by_index(1);
        let palette = panel.get_palette();
        assert_eq!(panel.settings.color_scheme, ColorScheme::HighContrast);
        assert_ne!(palette.healthy, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn test_high_contrast_palette_wcag() {
        let mut panel = AccessibilityPanel::default();
        panel.settings.color_scheme = ColorScheme::HighContrast;
        let palette = panel.get_palette();
        // High contrast: black background, white text - WCAG AAA
        assert_eq!(palette.background, egui::Color32::from_rgb(0, 0, 0));
        assert_eq!(palette.text, egui::Color32::from_rgb(255, 255, 255));
    }

    #[test]
    fn test_format_font_multiplier_display() {
        assert_eq!(
            format_font_multiplier_display(1.0),
            "  → Current multiplier: 1.0x"
        );
        assert_eq!(
            format_font_multiplier_display(0.85),
            "  → Current multiplier: 0.9x"
        );
        assert_eq!(
            format_font_multiplier_display(1.6),
            "  → Current multiplier: 1.6x"
        );
    }
}
