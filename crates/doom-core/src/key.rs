// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom key codes (mapped from egui).

/// Doom key codes (mapped from egui).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoomKey {
    Up,
    Down,
    Left,
    Right,
    StrafeLeft,
    StrafeRight,
    Fire,
    Use,
    Run,
    Weapon1,
    Weapon2,
    Weapon3,
    Weapon4,
    Weapon5,
    Weapon6,
    Weapon7,
    Enter,
    Escape,
    Map,
}

impl DoomKey {
    /// Convert to Doom's internal keycode.
    #[must_use]
    pub fn to_doom_code(self) -> i32 {
        match self {
            Self::Up => 0xAE,
            Self::Down => 0xAF,
            Self::Left => 0xAC,
            Self::Right => 0xAD,
            Self::Fire => 0x9D,
            Self::Use => i32::from(b' '),
            Self::Run => 0x9E,
            Self::Weapon1 => i32::from(b'1'),
            Self::Weapon2 => i32::from(b'2'),
            Self::Weapon3 => i32::from(b'3'),
            Self::Weapon4 => i32::from(b'4'),
            Self::Weapon5 => i32::from(b'5'),
            Self::Weapon6 => i32::from(b'6'),
            Self::Weapon7 => i32::from(b'7'),
            Self::StrafeLeft => i32::from(b','),
            Self::StrafeRight => i32::from(b'.'),
            Self::Enter => 13,
            Self::Escape => 27,
            Self::Map => i32::from(b'\t'),
        }
    }
}
