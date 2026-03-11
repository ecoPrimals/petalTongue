// SPDX-License-Identifier: AGPL-3.0-only
//! Keyboard Shortcuts System
//!
//! Comprehensive keyboard navigation for accessibility

use egui::Key;

/// Modifiers state for key mapping (egui-free)
#[derive(Clone, Debug, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

/// Map a key press + modifiers to a shortcut action without egui dependency.
#[must_use]
pub fn map_key_to_action(
    key: Key,
    modifiers: &KeyModifiers,
    _show_help: bool,
) -> Option<ShortcutAction> {
    // Help toggle (Shift+/)
    if key == Key::Questionmark {
        return Some(ShortcutAction::ToggleHelp);
    }

    // ESC - Close panels
    if key == Key::Escape {
        return Some(ShortcutAction::CloseOverlays);
    }

    // F keys
    if key == Key::F1 {
        return Some(ShortcutAction::ToggleHelp);
    }

    // Ctrl+ shortcuts
    if modifiers.ctrl {
        if key == Key::A {
            return Some(ShortcutAction::ToggleAccessibility);
        }
        if key == Key::D {
            return Some(ShortcutAction::ToggleDashboard);
        }
        if key == Key::H {
            return Some(ShortcutAction::ToggleHelp);
        }
        if key == Key::T {
            return Some(ShortcutAction::FocusTools);
        }
        if key == Key::R {
            return Some(ShortcutAction::Refresh);
        }
        for (i, k) in [
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
            if key == *k {
                return Some(ShortcutAction::SelectColorScheme(i));
            }
        }
        if key == Key::Plus || key == Key::Equals {
            return Some(ShortcutAction::IncreaseFontSize);
        }
        if key == Key::Minus {
            return Some(ShortcutAction::DecreaseFontSize);
        }
    }

    None
}

/// Keyboard shortcut configuration
#[derive(Clone, Debug, Default)]
pub struct KeyboardShortcuts {
    /// Show help overlay
    pub show_help: bool,
}

impl KeyboardShortcuts {
    /// Handle keyboard input for the app
    pub fn handle_input(&mut self, ctx: &egui::Context) -> ShortcutAction {
        let modifiers = ctx.input(|i| crate::keyboard_shortcuts::KeyModifiers {
            ctrl: i.modifiers.ctrl,
            shift: i.modifiers.shift,
            alt: i.modifiers.alt,
        });

        let keys_to_check = [
            Key::Questionmark,
            Key::Escape,
            Key::F1,
            Key::A,
            Key::D,
            Key::H,
            Key::T,
            Key::R,
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Plus,
            Key::Equals,
            Key::Minus,
        ];

        for key in keys_to_check {
            if ctx.input(|i| i.key_pressed(key)) {
                if let Some(action) = map_key_to_action(key, &modifiers, self.show_help) {
                    if action == ShortcutAction::ToggleHelp {
                        self.show_help = !self.show_help;
                    }
                    return action;
                }
            }
        }

        ShortcutAction::None
    }

    /// Render help overlay
    pub fn render_help(
        &mut self,
        ctx: &egui::Context,
        palette: &crate::accessibility::ColorPalette,
    ) {
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
                ui.label(
                    egui::RichText::new("General")
                        .strong()
                        .color(palette.accent),
                );
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
                ui.label(
                    egui::RichText::new("Accessibility")
                        .strong()
                        .color(palette.accent),
                );
                Self::shortcut_row(ui, "Ctrl+1-7", "Select color scheme (1-7)", palette);
                Self::shortcut_row(ui, "Ctrl++", "Increase font size", palette);
                Self::shortcut_row(ui, "Ctrl+-", "Decrease font size", palette);

                ui.add_space(10.0);

                // Navigation
                ui.label(
                    egui::RichText::new("Navigation")
                        .strong()
                        .color(palette.accent),
                );
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
    fn shortcut_row(
        ui: &mut egui::Ui,
        keys: &str,
        description: &str,
        palette: &crate::accessibility::ColorPalette,
    ) {
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

    fn no_mods() -> KeyModifiers {
        KeyModifiers::default()
    }

    fn ctrl_only() -> KeyModifiers {
        KeyModifiers {
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

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

    #[test]
    fn map_questionmark_toggle_help() {
        assert_eq!(
            map_key_to_action(Key::Questionmark, &no_mods(), false),
            Some(ShortcutAction::ToggleHelp)
        );
    }

    #[test]
    fn map_escape_close_overlays() {
        assert_eq!(
            map_key_to_action(Key::Escape, &no_mods(), false),
            Some(ShortcutAction::CloseOverlays)
        );
    }

    #[test]
    fn map_f1_toggle_help() {
        assert_eq!(
            map_key_to_action(Key::F1, &no_mods(), false),
            Some(ShortcutAction::ToggleHelp)
        );
    }

    #[test]
    fn map_ctrl_a_toggle_accessibility() {
        assert_eq!(
            map_key_to_action(Key::A, &ctrl_only(), false),
            Some(ShortcutAction::ToggleAccessibility)
        );
    }

    #[test]
    fn map_ctrl_d_toggle_dashboard() {
        assert_eq!(
            map_key_to_action(Key::D, &ctrl_only(), false),
            Some(ShortcutAction::ToggleDashboard)
        );
    }

    #[test]
    fn map_ctrl_h_toggle_help() {
        assert_eq!(
            map_key_to_action(Key::H, &ctrl_only(), false),
            Some(ShortcutAction::ToggleHelp)
        );
    }

    #[test]
    fn map_ctrl_t_focus_tools() {
        assert_eq!(
            map_key_to_action(Key::T, &ctrl_only(), false),
            Some(ShortcutAction::FocusTools)
        );
    }

    #[test]
    fn map_ctrl_r_refresh() {
        assert_eq!(
            map_key_to_action(Key::R, &ctrl_only(), false),
            Some(ShortcutAction::Refresh)
        );
    }

    #[test]
    fn map_ctrl_num1_through_7_color_schemes() {
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
            assert_eq!(
                map_key_to_action(*key, &ctrl_only(), false),
                Some(ShortcutAction::SelectColorScheme(i))
            );
        }
    }

    #[test]
    fn map_ctrl_plus_increase_font() {
        assert_eq!(
            map_key_to_action(Key::Plus, &ctrl_only(), false),
            Some(ShortcutAction::IncreaseFontSize)
        );
    }

    #[test]
    fn map_ctrl_equals_increase_font() {
        assert_eq!(
            map_key_to_action(Key::Equals, &ctrl_only(), false),
            Some(ShortcutAction::IncreaseFontSize)
        );
    }

    #[test]
    fn map_ctrl_minus_decrease_font() {
        assert_eq!(
            map_key_to_action(Key::Minus, &ctrl_only(), false),
            Some(ShortcutAction::DecreaseFontSize)
        );
    }

    #[test]
    fn map_a_without_ctrl_returns_none() {
        assert_eq!(map_key_to_action(Key::A, &no_mods(), false), None);
    }

    #[test]
    fn map_num1_without_ctrl_returns_none() {
        assert_eq!(map_key_to_action(Key::Num1, &no_mods(), false), None);
    }

    #[test]
    fn map_unknown_key_returns_none() {
        assert_eq!(map_key_to_action(Key::Space, &no_mods(), false), None);
    }
}
