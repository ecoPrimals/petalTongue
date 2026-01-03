//! Keyboard Shortcuts System
//!
//! Comprehensive keyboard navigation for accessibility

use egui::Key;

/// Keyboard shortcut configuration
#[derive(Clone, Debug)]
pub struct KeyboardShortcuts {
    /// Show help overlay
    pub show_help: bool,
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self { show_help: false }
    }
}

impl KeyboardShortcuts {
    /// Handle keyboard input for the app
    pub fn handle_input(&mut self, ctx: &egui::Context) -> ShortcutAction {
        // Help toggle (Shift+/)
        if ctx.input(|i| i.key_pressed(Key::Questionmark)) {
            self.show_help = !self.show_help;
            return ShortcutAction::ToggleHelp;
        }

        // ESC - Close panels
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            return ShortcutAction::CloseOverlays;
        }

        // Accessibility shortcuts
        if ctx.input(|i| i.modifiers.ctrl) {
            // Ctrl+A - Accessibility panel
            if ctx.input(|i| i.key_pressed(Key::A)) {
                return ShortcutAction::ToggleAccessibility;
            }

            // Ctrl+D - Dashboard
            if ctx.input(|i| i.key_pressed(Key::D)) {
                return ShortcutAction::ToggleDashboard;
            }

            // Ctrl+H - Help
            if ctx.input(|i| i.key_pressed(Key::H)) {
                self.show_help = !self.show_help;
                return ShortcutAction::ToggleHelp;
            }

            // Ctrl+T - Tools menu
            if ctx.input(|i| i.key_pressed(Key::T)) {
                return ShortcutAction::FocusTools;
            }

            // Ctrl+R - Refresh
            if ctx.input(|i| i.key_pressed(Key::R)) {
                return ShortcutAction::Refresh;
            }

            // Ctrl+1-9 - Color schemes
            for (i, key) in [
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Num6,
                Key::Num7,
            ]
            .iter()
            .enumerate()
            {
                if ctx.input(|input| input.key_pressed(*key)) {
                    return ShortcutAction::SelectColorScheme(i);
                }
            }

            // Ctrl+Plus/Minus - Font size
            if ctx.input(|i| i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals)) {
                return ShortcutAction::IncreaseFontSize;
            }
            if ctx.input(|i| i.key_pressed(Key::Minus)) {
                return ShortcutAction::DecreaseFontSize;
            }
        }

        // F keys
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.show_help = !self.show_help;
            return ShortcutAction::ToggleHelp;
        }

        ShortcutAction::None
    }

    /// Render help overlay
    pub fn render_help(&mut self, ctx: &egui::Context, palette: &crate::accessibility::ColorPalette) {
        if !self.show_help {
            return;
        }

        egui::Window::new("⌨️ Keyboard Shortcuts")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(500.0);

                ui.heading("Keyboard Navigation");
                ui.label("petalTongue is fully keyboard accessible!");
                ui.separator();
                ui.add_space(5.0);

                // General shortcuts
                ui.label(egui::RichText::new("General").strong().color(palette.accent));
                Self::shortcut_row(ui, "?", "Show/hide this help", palette);
                Self::shortcut_row(ui, "F1", "Show/hide this help", palette);
                Self::shortcut_row(ui, "ESC", "Close overlays", palette);
                Self::shortcut_row(ui, "Ctrl+R", "Refresh data", palette);

                ui.add_space(10.0);

                // Panel shortcuts
                ui.label(egui::RichText::new("Panels").strong().color(palette.accent));
                Self::shortcut_row(ui, "Ctrl+A", "Toggle Accessibility", palette);
                Self::shortcut_row(ui, "Ctrl+D", "Toggle Dashboard", palette);
                Self::shortcut_row(ui, "Ctrl+T", "Focus Tools menu", palette);
                Self::shortcut_row(ui, "Ctrl+H", "Toggle Help", palette);

                ui.add_space(10.0);

                // Accessibility shortcuts
                ui.label(egui::RichText::new("Accessibility").strong().color(palette.accent));
                Self::shortcut_row(ui, "Ctrl+1-7", "Select color scheme (1-7)", palette);
                Self::shortcut_row(ui, "Ctrl++", "Increase font size", palette);
                Self::shortcut_row(ui, "Ctrl+-", "Decrease font size", palette);

                ui.add_space(10.0);

                // Navigation
                ui.label(egui::RichText::new("Navigation").strong().color(palette.accent));
                Self::shortcut_row(ui, "Tab", "Next element", palette);
                Self::shortcut_row(ui, "Shift+Tab", "Previous element", palette);
                Self::shortcut_row(ui, "Enter", "Activate", palette);
                Self::shortcut_row(ui, "Space", "Select", palette);

                ui.add_space(10.0);

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("✓ Close (ESC)").clicked() {
                        self.show_help = false;
                    }
                });
            });
    }

    /// Render a shortcut row
    fn shortcut_row(ui: &mut egui::Ui, keys: &str, description: &str, palette: &crate::accessibility::ColorPalette) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(keys)
                    .family(egui::FontFamily::Monospace)
                    .color(palette.healthy),
            );
            ui.label("→");
            ui.label(description);
        });
    }
}

/// Action resulting from keyboard shortcut
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutAction {
    /// No action
    None,
    /// Toggle help overlay
    ToggleHelp,
    /// Close all overlays
    CloseOverlays,
    /// Toggle accessibility panel
    ToggleAccessibility,
    /// Toggle dashboard
    ToggleDashboard,
    /// Focus tools menu
    FocusTools,
    /// Refresh data
    Refresh,
    /// Select color scheme by index
    SelectColorScheme(usize),
    /// Increase font size
    IncreaseFontSize,
    /// Decrease font size
    DecreaseFontSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_shortcuts_creation() {
        let shortcuts = KeyboardShortcuts::default();
        assert!(!shortcuts.show_help);
    }

    #[test]
    fn test_shortcut_actions() {
        assert_eq!(ShortcutAction::None, ShortcutAction::None);
        assert_ne!(ShortcutAction::ToggleHelp, ShortcutAction::None);
    }
}

