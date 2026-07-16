// SPDX-License-Identifier: AGPL-3.0-or-later

/// Mouse button identifier for click events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button (primary click)
    Left,
    /// Right mouse button (secondary click)
    Right,
    /// Middle mouse button (scroll wheel click)
    Middle,
    /// Other button with raw identifier
    Other(u8),
}

/// Key identifier for keyboard events (layout-agnostic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// Printable character key
    Char(char),
    /// Named key (e.g. "Space", "`CapsLock`") for non-printable keys
    Named(String),
    /// Escape key
    Escape,
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Arrow up
    Up,
    /// Arrow down
    Down,
    /// Arrow left
    Left,
    /// Arrow right
    Right,
    /// Function key (F1 = F(1), etc.)
    F(u8),
    /// Unknown or unmapped key
    Unknown,
}

/// Keyboard modifier state (Ctrl, Alt, Shift, Meta/Cmd).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    /// Control key held
    pub ctrl: bool,
    /// Alt/Option key held
    pub alt: bool,
    /// Shift key held
    pub shift: bool,
    /// Meta/Windows/Cmd key held
    pub meta: bool,
}

impl Modifiers {
    /// No modifiers pressed.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    /// Ctrl modifier only.
    #[must_use]
    pub const fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_none_all_false() {
        let m = Modifiers::none();
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.shift);
        assert!(!m.meta);
    }

    #[test]
    fn modifiers_ctrl_only_ctrl_true() {
        let m = Modifiers::ctrl();
        assert!(m.ctrl);
        assert!(!m.alt);
        assert!(!m.shift);
        assert!(!m.meta);
    }

    #[test]
    fn key_debug_format() {
        assert_eq!(format!("{:?}", Key::Escape), "Escape");
        assert_eq!(format!("{:?}", Key::Char('a')), "Char('a')");
        assert_eq!(format!("{:?}", Key::F(5)), "F(5)");
    }

    #[test]
    fn mouse_button_debug_format() {
        assert_eq!(format!("{:?}", MouseButton::Left), "Left");
        assert_eq!(format!("{:?}", MouseButton::Other(4)), "Other(4)");
    }
}
